use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use super::command::{Command, Gone};
use crate::state::AppState;

/// Answers a command that reached a room after it stopped: the hub retries on a new room.
pub fn answer_gone(command: Command) {
    match command {
        Command::Join(_, reply) => _ = reply.send(Err(Gone)),
        Command::Ops(_, _, reply) => _ = reply.send(Err(Gone)),
        Command::Replace(_, _, reply) => _ = reply.send(Err(Gone)),
        Command::Shutdown(done) => _ = done.send(()),
        Command::Message(..) | Command::Leave(_) | Command::Evict(_) => {}
    }
}

/// Answers every command with `error`: the document could not be loaded.
pub async fn refuse_all(
    state: &AppState,
    id: Uuid,
    generation: u64,
    mut commands: mpsc::Receiver<Command>,
    error: tronc::Error,
) {
    state.hub.forget(id, generation);
    commands.close();
    while let Some(command) = commands.recv().await {
        let failure = tronc::Error::new(error.code, error.message.clone());
        match command {
            Command::Join(_, reply) => _ = reply.send(Ok(Err(failure))),
            Command::Ops(_, _, reply) => _ = reply.send(Ok(Err(failure))),
            Command::Replace(_, _, reply) => _ = reply.send(Ok(Err(failure))),
            other => answer_gone(other),
        }
    }
}

/// Shuts the rooms behind `senders` down one by one, waiting for each to persist.
pub async fn stop(senders: Vec<mpsc::Sender<Command>>) {
    for sender in senders {
        let (done, wait) = oneshot::channel();
        if sender.send(Command::Shutdown(done)).await.is_ok() {
            _ = wait.await;
        }
    }
}
