use encre::ops::Op;
use tokio::sync::{mpsc, oneshot};

use crate::modules::documents::content::Prepared;
use crate::modules::documents::types::OpsAnswer;

/// A frame for one connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Out {
    /// A y-sync message.
    Binary(Vec<u8>),
    /// Close the socket with this code and reason.
    Close(u16, String),
}

/// The room unloaded while the command was in flight; the hub retries on a fresh room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gone;

/// A command answer: `Err(Gone)` asks for a retry, the inner result is the room's.
pub type Reply<T> = oneshot::Sender<Result<tronc::Result<T>, Gone>>;

/// What a connection told the room when it joined.
#[derive(Debug, Clone)]
pub struct Member {
    /// The connection.
    pub conn: u64,
    /// The porte user.
    pub user_id: i64,
    /// Whether its updates are applied; a viewer's are dropped.
    pub editor: bool,
    /// Where the room writes to it.
    pub outbox: mpsc::Sender<Out>,
}

/// One instruction to a room task.
#[derive(Debug)]
pub enum Command {
    /// A connection joins and receives the opening sync messages.
    Join(Member, Reply<()>),
    /// A binary frame from a connection.
    Message(u64, Vec<u8>),
    /// A connection left.
    Leave(u64),
    /// Agent operations, applied in one transaction and persisted before the answer.
    Ops(Vec<Op>, i64, Reply<OpsAnswer>),
    /// The whole document replaced (a revision restore), persisted before the answer.
    Replace(Box<Prepared>, i64, Reply<i64>),
    /// Close every connection of these users (they lost access).
    Evict(Vec<i64>),
    /// Persist now and unload.
    Shutdown(oneshot::Sender<()>),
}
