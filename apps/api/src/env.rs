use std::path::PathBuf;

use tronc::env::{Core, EnvError, Vars};

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
            markdown_max: markdown_max(vars)?,
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

fn markdown_max(vars: &Vars) -> Result<usize, EnvError> {
    let raw = vars.int("MARKDOWN_MAX_BYTES", MARKDOWN_MAX_BYTES as i64)?;
    match usize::try_from(raw) {
        Ok(max) if max > 0 => Ok(max),
        _ => Err(EnvError(format!("MARKDOWN_MAX_BYTES must be a positive number of bytes, not {raw}"))),
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
    fn defaults_are_sso_only_with_data_under_slash_data_and_a_positive_markdown_cap() {
        let config = Config::from_vars(&vars(&[])).unwrap();
        assert!(config.porte.sso_only);
        assert_eq!(config.data_dir, PathBuf::from("/data"));
        assert_eq!(config.markdown_max, 1 << 20);
        let config = Config::from_vars(&vars(&[("MARKDOWN_MAX_BYTES", "4096")])).unwrap();
        assert_eq!(config.markdown_max, 4096);
        assert!(Config::from_vars(&vars(&[("MARKDOWN_MAX_BYTES", "0")])).is_err());
        assert!(Config::from_vars(&vars(&[("MARKDOWN_MAX_BYTES", "lots")])).is_err());
    }
}
