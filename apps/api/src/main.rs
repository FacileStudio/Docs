//! The `api` binary: the server, `healthcheck`, and `migrate`.
#![deny(unused, dead_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    if tronc::healthcheck::handle(std::env::args()) {
        return ExitCode::SUCCESS;
    }
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map(|runtime| runtime.block_on(api::server::run()))
        .unwrap_or(ExitCode::FAILURE)
}
