use std::collections::{HashMap, HashSet};
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::Instant;
use uuid::Uuid;
use yrs::ClientID;
use yrs::sync::Awareness;

use super::command::{Command, Out};
use super::drain::{answer_gone, refuse_all};
use super::load::{Loaded, load};
use super::persist::persist;
use super::sync::{Core, Effect};
use crate::state::AppState;

/// How long after the last change a room persists.
pub const DEBOUNCE: Duration = Duration::from_secs(2);
/// How long an empty room stays loaded.
pub const IDLE: Duration = Duration::from_secs(60);

pub(super) struct Conn {
    pub(super) outbox: mpsc::Sender<Out>,
    pub(super) user_id: i64,
    pub(super) editor: bool,
    pub(super) clients: HashSet<ClientID>,
}

pub(super) struct Room {
    pub(super) state: AppState,
    pub(super) id: Uuid,
    pub(super) core: Core,
    pub(super) conns: HashMap<u64, Conn>,
    pub(super) version: i64,
    pub(super) trashed: bool,
    pub(super) dirty: Option<(Instant, Option<i64>)>,
    pub(super) unrevised: bool,
    pub(super) empty_since: Option<Instant>,
    pub(super) ended: bool,
}

/// The room task: loads the document, then serves commands until it has been empty for
/// [`IDLE`] or is shut down.
pub async fn run(state: AppState, id: Uuid, generation: u64, mut commands: mpsc::Receiver<Command>) {
    let loaded = match load(&state.pool, id).await {
        Ok(Some(loaded)) => Ok(loaded),
        Ok(None) => Err(tronc::Error::not_found("document not found")),
        Err(error) => Err(tronc::Error::internal("failed to load a room", error)),
    };
    let mut room = match loaded {
        Ok(Loaded { doc, version, trashed }) => {
            let core = Core { doc, awareness: Awareness::new(yrs::Doc::new()) };
            Room::new(state, id, core, version, trashed)
        }
        Err(error) => return refuse_all(&state, id, generation, commands, error).await,
    };
    loop {
        let deadline = room.deadline();
        tokio::select! {
            command = commands.recv() => match command {
                Some(command) => if !room.handle(command).await { break },
                None => break,
            },
            () = sleep_until(deadline) => if room.tick().await { break },
        }
    }
    room.state.hub.forget(id, generation);
    commands.close();
    while let Ok(command) = commands.try_recv() {
        answer_gone(command);
    }
}

async fn sleep_until(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline).await,
        None => std::future::pending().await,
    }
}

impl Room {
    fn new(state: AppState, id: Uuid, core: Core, version: i64, trashed: bool) -> Self {
        let (conns, dirty, empty_since) = (HashMap::new(), None, Some(Instant::now()));
        Room { state, id, core, conns, version, trashed, dirty, unrevised: false, empty_since, ended: false }
    }

    fn deadline(&self) -> Option<Instant> {
        if self.ended {
            return Some(Instant::now());
        }
        let save = self.dirty.map(|(at, _)| at + DEBOUNCE);
        let idle = self.empty_since.map(|at| at + IDLE);
        save.into_iter().chain(idle).min()
    }

    async fn tick(&mut self) -> bool {
        if self.dirty.is_some_and(|(at, _)| at + DEBOUNCE <= Instant::now()) {
            self.save(false).await;
        }
        self.ended || self.empty_since.is_some_and(|at| at + IDLE <= Instant::now()) && self.dirty.is_none()
    }

    pub(super) async fn save(&mut self, force: bool) {
        let Some((_, author)) = self.dirty else { return };
        match persist(&self.state.pool, self.id, &self.core.doc, author, force).await {
            Ok(version) => {
                self.version = version;
                self.dirty = None;
                self.unrevised &= !force;
            }
            Err(error) if error.code == tronc::Code::NotFound => self.vanish(),
            Err(error) => {
                tracing::error!(document = %self.id, error = %error.message, "rooms: persist failed, retrying");
                self.dirty = Some((Instant::now(), author));
            }
        }
    }

    pub(super) fn send(&mut self, conn: u64, out: Out) {
        let full = self.conns.get(&conn).is_some_and(|c| c.outbox.try_send(out).is_err());
        if full && let Some(conn) = self.conns.remove(&conn) {
            _ = conn.outbox.try_send(Out::Close(1013, "too slow".to_owned()));
        }
    }

    pub(super) fn apply_effects(&mut self, conn: u64, user_id: i64, effects: Vec<Effect>) {
        for effect in effects {
            match effect {
                Effect::Reply(bytes) => self.send(conn, Out::Binary(bytes)),
                Effect::Broadcast(bytes) => self.broadcast(Some(conn), &bytes),
                Effect::Changed => self.touch(Some(user_id)),
                Effect::Refuse(code, reason) => self.drop_conn(conn, Some(Out::Close(code, reason))),
            }
        }
    }
}
