use serde::Serialize;
use tronc::json::Json;

/// The release this binary was built from, stamped by the Dockerfile through `APP_VERSION`.
pub const VERSION: &str = match option_env!("APP_VERSION") {
    Some(version) => version,
    None => "dev",
};

/// What `GET /version` answers.
#[derive(Serialize)]
pub struct VersionAnswer {
    version: &'static str,
    encre_schema: u32,
}

/// `GET /version`.
pub async fn handler() -> Json<VersionAnswer> {
    Json(VersionAnswer { version: VERSION, encre_schema: encre::SCHEMA_VERSION })
}
