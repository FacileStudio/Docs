use tokio::time::Instant;

use super::command::{Command, Member, Out};
use super::room::{Conn, Room};
use super::sync::{self, departure};
use crate::modules::documents::access::{self, Access};

impl Room {
    /// Handles one command; `false` ends the room.
    pub(super) async fn handle(&mut self, command: Command) -> bool {
        match command {
            Command::Join(member, reply) => {
                self.join(member);
                _ = reply.send(Ok(Ok(())));
            }
            Command::Message(conn, frame) => self.message(conn, &frame),
            Command::Leave(conn) => self.leave(conn).await,
            Command::Ops(ops, user, reply) => _ = reply.send(Ok(self.ops(&ops, user).await)),
            Command::Replace(prepared, user, reply) => _ = reply.send(Ok(self.replace(&prepared, user).await)),
            Command::Evict(users) => self.evict(&users).await,
            Command::Shutdown(done) => {
                self.stop().await;
                _ = done.send(());
                return false;
            }
        }
        true
    }

    fn join(&mut self, member: Member) {
        let editor = member.editor && !self.trashed;
        let conn = Conn { outbox: member.outbox, user_id: member.user_id, editor, clients: Default::default() };
        self.conns.insert(member.conn, conn);
        self.empty_since = None;
        for frame in sync::opening(&self.core) {
            self.send(member.conn, Out::Binary(frame));
        }
    }

    fn message(&mut self, conn: u64, frame: &[u8]) {
        let Some(member) = self.conns.get_mut(&conn) else { return };
        let (user_id, editor) = (member.user_id, member.editor);
        let effects = sync::handle(&mut self.core, frame, editor, &mut member.clients);
        self.apply_effects(conn, user_id, effects);
    }

    async fn leave(&mut self, conn: u64) {
        self.drop_conn(conn, None);
        if self.conns.is_empty() {
            self.save(true).await;
        }
    }

    /// Removes a member, tells the others its cursors left, and closes it with `close`.
    pub(super) fn drop_conn(&mut self, conn: u64, close: Option<Out>) {
        let Some(member) = self.conns.remove(&conn) else { return };
        if let Some(close) = close {
            _ = member.outbox.try_send(close);
        }
        if let Some(gone) = departure(&mut self.core.awareness, &member.clients) {
            self.broadcast(None, &gone);
        }
        if self.conns.is_empty() {
            self.empty_since = Some(Instant::now());
        }
    }

    /// Sends `frame` to every member but `except`.
    pub(super) fn broadcast(&mut self, except: Option<u64>, frame: &[u8]) {
        let targets: Vec<u64> = self.conns.keys().copied().filter(|c| Some(*c) != except).collect();
        for conn in targets {
            self.send(conn, Out::Binary(frame.to_vec()));
        }
    }

    /// Marks the document changed by `author`; it persists [`super::room::DEBOUNCE`] later.
    pub(super) fn touch(&mut self, author: Option<i64>) {
        self.dirty = Some((Instant::now(), author.or(self.dirty.and_then(|(_, a)| a))));
        self.unrevised = true;
    }

    async fn evict(&mut self, users: &[i64]) {
        let mut lost = Vec::new();
        for user in users.iter().filter(|u| self.conns.values().any(|c| c.user_id == **u)) {
            if matches!(access::access(&self.state, *user, self.id).await, Ok(Access::None)) {
                lost.push(*user);
            }
        }
        let gone: Vec<u64> = self.conns.iter().filter(|(_, c)| lost.contains(&c.user_id)).map(|(id, _)| *id).collect();
        for conn in gone {
            self.drop_conn(conn, Some(Out::Close(1008, "access revoked".to_owned())));
        }
    }
}
