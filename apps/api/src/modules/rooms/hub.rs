//! Docs' handle on its salon rooms, with the calls the rest of the API makes.

use encre::ops::Op;
use salon::{Change, Due};
use uuid::Uuid;

use super::edits;
use crate::modules::documents::content::Prepared;
use crate::modules::documents::ops::MAX_OPS;
use crate::modules::documents::types::OpsAnswer;
use crate::state::AppState;

/// Every open page room. One replica only: two would split a room.
#[derive(Clone, Default)]
pub struct Hub(pub(super) salon::Hub<AppState>);

impl Hub {
    /// Agent operations by `author`, applied in one transaction inside the room of page `id` and
    /// persisted before the answer.
    pub async fn ops(&self, state: &AppState, id: Uuid, list: Vec<Op>, author: i64) -> tronc::Result<OpsAnswer> {
        if list.len() > MAX_OPS {
            return Err(tronc::Error::invalid(format!("at most {MAX_OPS} operations per request")));
        }
        let change =
            Change { due: Due::Flush, author: Some(author), edit: edits::apply(list), answer: edits::answered };
        self.0.write(state, id, change).await?
    }

    /// Replaces the whole page `id` by `prepared` (a revision restore), writes a revision and
    /// answers the new version.
    pub async fn replace(
        &self,
        state: &AppState,
        id: Uuid,
        prepared: Box<Prepared>,
        author: i64,
    ) -> tronc::Result<i64> {
        let answer = |page: &super::space::Page, _| page.version;
        let change = Change { due: Due::Snapshot, author: Some(author), edit: edits::replace(prepared), answer };
        self.0.write(state, id, change).await
    }

    /// Persists and unloads the rooms of `ids`, closing their members with 1001: they
    /// reconnect to a room that reads the document's new state (trashed, restored, deleted).
    pub async fn close(&self, ids: &[Uuid]) {
        self.0.close(ids, 1001, "server restarting").await;
    }

    /// Persists and unloads every room, for a graceful stop.
    pub async fn shutdown(&self) {
        self.0.shutdown(1001, "server restarting").await;
    }

    /// Closes, in every open room, the connections of `users` that lost access.
    pub async fn evict(&self, users: &[i64]) {
        self.0.evict(users).await;
    }
}
