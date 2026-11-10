//! `GET /search`: full-text search over titles and text, inside the caller's scope only.

mod router;
mod service;

pub use router::routes;
pub use service::{Hit, search};
