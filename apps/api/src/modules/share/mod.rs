//! Share links: public, read-only views of a page and, optionally, its subtree.

pub mod public;
mod router;
pub mod service;

pub use router::{public_routes, routes};
