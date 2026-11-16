//! Docs' side of the rooms: how a page loads, saves, and who may stay in its room.

use std::time::Duration;

use salon::{Caller, Due, Fault, Kind, Policy, Saved};
use tokio::time::Instant;
use tronc::Code;
use uuid::Uuid;

use super::live::alive;
use super::load::{Loaded, load};
use super::persist::persist;
use super::space::Page;
use crate::modules::documents::access::{self, Access};
use crate::state::AppState;

/// How long after the last change a room persists.
const DEBOUNCE: Duration = Duration::from_secs(2);

fn code(kind: Kind) -> Code {
    match kind {
        Kind::NotFound => Code::NotFound,
        Kind::Invalid => Code::InvalidArgument,
        Kind::Failed => Code::FailedPrecondition,
        Kind::Unavailable => Code::Unavailable,
        Kind::Internal => Code::Internal,
    }
}

impl salon::Host for AppState {
    type Space = Page;
    type Error = tronc::Error;

    fn rooms(&self) -> (&salon::Hub<Self>, Policy) {
        let policy = Policy {
            flush: DEBOUNCE,
            debounce: true,
            retry: DEBOUNCE,
            snapshot_every: usize::MAX,
            snapshot_bytes: usize::MAX,
            snapshot_after: None,
            idle: Duration::from_secs(60),
            recheck: self.socket_recheck,
            blocking: true,
            noun: "document",
        };
        (&self.hub.0, policy)
    }

    fn fault(fault: Fault) -> tronc::Error {
        tronc::Error::new(code(fault.kind), fault.message)
    }

    async fn open(&self, key: Uuid) -> Result<Page, Fault> {
        match load(&self.pool, key).await {
            Ok(Some(Loaded { doc, version, trashed })) => Ok(Page { doc, version, trashed }),
            Ok(None) => Err(Fault::new(Kind::NotFound, "document not found")),
            Err(error) => {
                tracing::error!(document = %key, error = %error, "rooms: failed to load a room");
                Err(Fault::new(Kind::Internal, "failed to load a room"))
            }
        }
    }

    async fn save(&self, key: Uuid, page: &mut Page, due: Due, author: Option<i64>) -> Saved {
        match persist(&self.pool, key, &page.doc, author, due == Due::Snapshot).await {
            Ok(version) => {
                page.version = version;
                Saved::Stored
            }
            Err(error) if error.code == Code::NotFound => Saved::Gone,
            Err(error) => {
                tracing::error!(document = %key, error = %error.message, "rooms: persist failed, retrying");
                Saved::Retry
            }
        }
    }

    async fn alive(&self, caller: &Caller, opened: Instant) -> Option<(u16, String)> {
        alive(self, caller, opened).await
    }

    async fn lost(&self, key: Uuid, user: i64) -> bool {
        matches!(access::access(self, user, key).await, Ok(Access::None))
    }
}
