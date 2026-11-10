//! Attachments: content-addressed files on the data volume, served sandboxed.

pub mod files;
mod router;
pub mod service;
pub mod sniff;
pub mod sweep;

pub use router::routes;
