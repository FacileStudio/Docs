//! Operator subcommands, run in the container: `docker exec -i docs-api /api ops <id> -`.

use std::io::Read;

use encre::ops::Op;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::Kits;
use crate::env::Config;
use crate::modules::rooms::{Command, Hub};

type Error = Box<dyn std::error::Error + Send + Sync>;

const USAGE: &str = "usage: api ops <document-id> <ops.json | ->";

/// Runs an operator subcommand when `argv` names one, or answers `None` for the server.
pub async fn command(argv: &[String], config: &Config, pool: &PgPool) -> Option<Result<(), Error>> {
    match argv.get(1).map(String::as_str) {
        Some("ops") => Some(ops(&argv[2..], config, pool).await),
        _ => None,
    }
}

fn read_ops(path: &str) -> Result<Vec<Op>, Error> {
    let mut raw = Vec::new();
    match path {
        "-" => _ = std::io::stdin().read_to_end(&mut raw)?,
        path => raw = std::fs::read(path)?,
    }
    Ok(serde_json::from_slice(&raw)?)
}

/// `api ops <document-id> <ops.json | ->`: applies a JSON list of encre operations to one page
/// through its room, as `POST /documents/{id}/ops` does but with no author, and prints the
/// answer (`[{"op": "read"}]` reads the page). The room lives in this process: run it while
/// nobody has the page open, or a room the server holds would overwrite the change.
async fn ops(args: &[String], config: &Config, pool: &PgPool) -> Result<(), Error> {
    let [id, path] = args else { return Err(USAGE.into()) };
    let id: Uuid = id.parse().map_err(|_| USAGE)?;
    let list = read_ops(path)?;
    let hub = Hub::default();
    let kits = Kits::new(config, pool, &hub).await?;
    let state = crate::server::state(config, pool.clone(), &kits, hub.clone());
    let answer = hub.ask(&state, id, |reply| Command::Ops(list.clone(), 0, reply)).await;
    hub.shutdown().await;
    println!("{}", serde_json::to_string(&answer?)?);
    Ok(())
}
