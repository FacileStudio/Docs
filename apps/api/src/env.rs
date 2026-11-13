use std::path::PathBuf;

use tronc::env::{Core, EnvError, Vars};

use crate::modules::attachments::sniff::FILE_MAX_BYTES;
use crate::modules::documents::markdown::MARKDOWN_MAX_BYTES;

/// Everything the binary reads from its environment.
#[derive(Debug, Clone)]
pub struct Config {
    /// The chassis variables: port, database, proxies, Journal.
    pub core: Core,
    /// Registre sign-in. `sso_only` is always true.
    pub porte: porte::Config,
    /// `PORTE_TOKEN_KEY`: seals the stored OIDC tokens.
    pub token_key: String,
    /// `DATA_DIR`: where attachments live.
    pub data_dir: PathBuf,
    /// `MCP_RESOURCE`: turns `/mcp` on.
    pub mcp_resource: String,
    /// `MCP_STATE_KEY`: the HMAC key of the confirmation flow.
    pub mcp_state_key: String,
    /// `MARKDOWN_MAX_BYTES`: the largest markdown body one request may ask the API to parse.
    pub markdown_max: usize,
    /// `ATTACHMENT_MAX_BYTES`: the largest attachment that is not an image or a PDF.
    pub attachment_max: usize,
    /// `BULLE_URL` and `BULLE_CLIENT_ID`: where the person's assistant lives.
    pub bulle: crate::modules::assistant::Bulle,
}

impl Config {
    /// Reads the process environment.
    pub fn load() -> Result<Self, EnvError> {
        Self::from_vars(&Vars::process())
    }

    /// Reads `vars`, refusing a production boot without Registre.
    pub fn from_vars(vars: &Vars) -> Result<Self, EnvError> {
        let core = Core::from_vars(vars, true)?;
        let text = |key: &str| vars.string(key, "").trim().to_owned();
        let porte = porte::Config { app_name: "Docs".to_owned(), sso_only: true, ..porte::Config::from_vars(vars)? };
        let config = Config {
            porte,
            token_key: text("PORTE_TOKEN_KEY"),
            data_dir: PathBuf::from(vars.string("DATA_DIR", "/data")),
            mcp_resource: text("MCP_RESOURCE"),
            mcp_state_key: text("MCP_STATE_KEY"),
            markdown_max: bytes(vars, "MARKDOWN_MAX_BYTES", MARKDOWN_MAX_BYTES)?,
            attachment_max: bytes(vars, "ATTACHMENT_MAX_BYTES", FILE_MAX_BYTES)?,
            bulle: crate::modules::assistant::Bulle::from_vars(vars),
            core,
        };
        config.check()?;
        Ok(config)
    }

    fn check(&self) -> Result<(), EnvError> {
        if self.core.is_production() && self.porte.issuer.is_empty() {
            return Err(EnvError("OIDC_ISSUER is required in production".to_owned()));
        }
        if self.core.is_production() && self.token_key.is_empty() {
            return Err(EnvError("PORTE_TOKEN_KEY is required in production".to_owned()));
        }
        if !self.mcp_resource.is_empty() && self.mcp_state_key.is_empty() {
            return Err(EnvError("MCP_STATE_KEY is required with MCP_RESOURCE".to_owned()));
        }
        Ok(())
    }
}

fn bytes(vars: &Vars, key: &str, default: usize) -> Result<usize, EnvError> {
    let raw = vars.int(key, default as i64)?;
    match usize::try_from(raw) {
        Ok(max) if max > 0 => Ok(max),
        _ => Err(EnvError(format!("{key} must be a positive number of bytes, not {raw}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(extra: &[(&str, &str)]) -> Vars {
        let mut pairs = vec![("DATABASE_URL", "postgres://x/y")];
        pairs.extend_from_slice(extra);
        Vars::from_pairs(pairs)
    }

    #[test]
    fn production_needs_the_issuer() {
        let error = Config::from_vars(&vars(&[("APP_ENV", "production")])).unwrap_err();
        assert!(error.0.contains("OIDC_ISSUER"), "{error}");
    }

    #[test]
    fn mcp_needs_a_state_key() {
        let error = Config::from_vars(&vars(&[("MCP_RESOURCE", "https://docs.test/mcp")])).unwrap_err();
        assert!(error.0.contains("MCP_STATE_KEY"), "{error}");
    }

    #[test]
    fn defaults_are_sso_only_with_slash_data_positive_caps_and_the_suite_bulle() {
        let config = Config::from_vars(&vars(&[])).unwrap();
        assert!(config.porte.sso_only);
        assert_eq!(config.data_dir, PathBuf::from("/data"));
        assert_eq!(config.markdown_max, 1 << 20);
        assert_eq!(config.attachment_max, 25 << 20);
        assert_eq!(
            (config.bulle.url.as_str(), config.bulle.audience.as_str()),
            ("https://bulle.facile.studio", "bulle")
        );
        let config = Config::from_vars(&vars(&[("MARKDOWN_MAX_BYTES", "4096")])).unwrap();
        assert_eq!(config.markdown_max, 4096);
        assert!(Config::from_vars(&vars(&[("MARKDOWN_MAX_BYTES", "0")])).is_err());
        assert!(Config::from_vars(&vars(&[("MARKDOWN_MAX_BYTES", "lots")])).is_err());
        let custom =
            [("ATTACHMENT_MAX_BYTES", "1024"), ("BULLE_URL", "https://bulle.test/"), ("BULLE_CLIENT_ID", "b2")];
        let config = Config::from_vars(&vars(&custom)).unwrap();
        assert_eq!((config.attachment_max, config.bulle.url.as_str()), (1024, "https://bulle.test"));
        assert_eq!(config.bulle.audience, "b2");
        assert!(Config::from_vars(&vars(&[("ATTACHMENT_MAX_BYTES", "-1")])).is_err());
    }
}
