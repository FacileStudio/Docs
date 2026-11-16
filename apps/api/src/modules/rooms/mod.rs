//! Collaboration rooms: one in-memory Yjs document per open page, hosted by salon over y-sync.

mod edits;
mod host;
mod hub;
mod live;
mod load;
mod persist;
mod router;
mod space;

use std::time::Duration;

pub use hub::Hub;
pub use router::routes;

/// How often a socket re-checks its session in production: a logout, a back-channel logout or
/// an expiry closes it within this long.
pub const SOCKET_RECHECK: Duration = Duration::from_secs(30);
