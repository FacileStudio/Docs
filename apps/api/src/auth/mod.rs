//! porte wiring: sessions, Registre sign-in, directory sync, deletions and MCP auth.

mod deletions;
mod picture;
mod retire;
mod routes;
mod spaces;
mod store;

use std::sync::Arc;
use std::time::Duration;

use porte::directory::{self, Deletions, Syncer};
use porte::http_client::HttpClient;
use porte::oidc::{self, Kit};
use porte::session::{self, Manager};
use porte::{mcpauth, pg};
use sqlx::PgPool;

pub use deletions::{AppData, DeletionState};
pub use picture::Pictures;
pub use routes::routes;
pub use spaces::SpacesApply;
pub use store::SpaceStore;

use crate::env::Config;
use crate::modules::rooms::Hub;

/// Every auth kit the app builds once at boot.
#[derive(Clone)]
pub struct Kits {
    /// Sessions, cookies and the auth middleware.
    pub sessions: Manager,
    /// Registre sign-in routes.
    pub oidc: Kit,
    /// The directory loop, when Registre is configured.
    pub syncer: Option<Syncer>,
    /// The deletion loop and receiver, when Registre is configured.
    pub deletions: Option<Deletions>,
    /// The MCP token verifier, when `MCP_RESOURCE` is set.
    pub mcp: Option<Arc<mcpauth::Verifier>>,
}

type BootError = Box<dyn std::error::Error + Send + Sync>;

impl Kits {
    /// Builds the kits. A wrong issuer fails here, so the boot fails.
    pub async fn new(cfg: &Config, pool: &PgPool, hub: &Hub) -> Result<Kits, BootError> {
        let store = pg::Store::new(pool.clone()).with_token_sealer(pg::TokenSealer::parse(&cfg.token_key)?);
        let sessions = Manager::new(&cfg.porte, session::Deps { sessions: store.sessions(), now: None });
        let deps = oidc::Deps {
            users: Pictures::wrap(store.users()),
            identities: store.identities(),
            codes: store.login_codes(),
            sessions: sessions.clone(),
            config_extra: None,
            http: None,
            now: None,
        };
        let oidc = Kit::new(&cfg.porte, deps).await?;
        let mut kits = Kits { sessions, oidc, syncer: None, deletions: None, mcp: None };
        if cfg.porte.enabled() {
            kits.directory(cfg, pool, hub, store.clone());
        }
        if !cfg.mcp_resource.is_empty() {
            kits.mcp = Some(Arc::new(mcp_verifier(cfg, &store, &kits.oidc).await?));
        }
        Ok(kits)
    }

    fn directory(&mut self, cfg: &Config, pool: &PgPool, hub: &Hub, store: pg::Store) {
        let client = directory::Client {
            issuer: cfg.porte.issuer.clone(),
            client_id: cfg.porte.client_id.clone(),
            client_secret: cfg.porte.client_secret.clone(),
            http: HttpClient::new(),
        };
        let apply = Arc::new(SpacesApply::new(pool.clone(), hub.clone()));
        self.syncer = Some(Syncer::new(client.clone(), apply, Duration::ZERO));
        let data = Arc::new(AppData::new(pool.clone()));
        let state = Arc::new(DeletionState::new(pool.clone()));
        self.deletions = Some(Deletions::new(client, data, Some(Arc::new(store)), Some(state), Duration::ZERO));
    }
}

async fn mcp_verifier(cfg: &Config, store: &pg::Store, oidc: &Kit) -> Result<mcpauth::Verifier, BootError> {
    let config = mcpauth::Config {
        issuer: cfg.porte.issuer.clone(),
        resource: cfg.mcp_resource.clone(),
        scope_prefix: "docs".to_owned(),
        audience: cfg.porte.client_id.clone(),
        identities: store.identities(),
        provision: true,
        accounts: Some(oidc.clone()),
        http: HttpClient::new(),
    };
    Ok(mcpauth::Verifier::new(config).await?)
}
