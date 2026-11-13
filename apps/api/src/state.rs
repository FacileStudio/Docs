use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use porte::exchange::Exchanger;
use porte::session::Manager;
use tokio::sync::Semaphore;
use tronc::mcp::Confirmations;

use crate::modules::assistant::Bulle;
use crate::modules::embeds::Previews;
use crate::modules::rooms::Hub;
use sqlx::PgPool;

/// What every handler shares.
#[derive(Clone)]
pub struct AppState {
    /// The database.
    pub pool: PgPool,
    /// Sessions, for the WebSocket origin check.
    pub sessions: Manager,
    /// The Registre issuer: the `provider` of every identity whose subject is a space member.
    pub issuer: Arc<str>,
    /// Where attachments live.
    pub data_dir: Arc<PathBuf>,
    /// The open collaboration rooms.
    pub hub: Hub,
    /// The PDF compile slots.
    pub exports: Arc<Semaphore>,
    /// The largest markdown body one request may ask the API to parse.
    pub markdown_max: usize,
    /// The MCP confirmation flow, when `/mcp` is on.
    pub confirmations: Option<Confirmations>,
    /// How often an open socket re-checks its session.
    pub socket_recheck: Duration,
    /// The largest attachment that is not an image or a PDF.
    pub attachment_max: usize,
    /// Link previews for embeds.
    pub previews: Previews,
    /// Mints the person's tokens for other suite apps; `None` without Registre.
    pub exchanger: Option<Exchanger>,
    /// Where the person's assistant lives.
    pub bulle: Bulle,
}
