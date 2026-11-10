//! Documents: the tree of pages per scope, their content, trash and revisions.

pub mod access;
pub mod content;
mod handlers;
pub mod ops;
pub mod purge;
pub mod revisions;
mod router;
pub mod service;
pub mod trash;
pub mod tree;
pub mod types;
pub mod write;

pub use router::routes;
