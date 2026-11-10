use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use http::request::Parts;
use porte::mcpauth;
use tronc::mcp::{EXTRA_ACTOR, EXTRA_CLIENT_ID, TokenInfo, Verifier, VerifyError, VerifyFuture};

/// porte's MCP verifier behind tronc's trait: tronc never imports porte, so the app joins them.
pub struct PorteVerifier(pub Arc<mcpauth::Verifier>);

fn token_info(info: mcpauth::TokenInfo) -> TokenInfo {
    let seconds = u64::try_from(info.expiration.unix_timestamp()).unwrap_or(0);
    let extra = HashMap::from([(EXTRA_CLIENT_ID.to_owned(), info.client_id), (EXTRA_ACTOR.to_owned(), info.actor)]);
    TokenInfo {
        user_id: info.user_id,
        scopes: info.scopes,
        expires_at: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)),
        extra,
    }
}

impl Verifier for PorteVerifier {
    fn verify<'a>(&'a self, token: &'a str, _request: &'a Parts) -> VerifyFuture<'a> {
        Box::pin(async move {
            match self.0.verify(token).await {
                Ok(info) => Ok(token_info(info)),
                Err(mcpauth::VerifyError::InvalidToken) => Err(VerifyError::Invalid),
                Err(mcpauth::VerifyError::Unavailable) => Err(VerifyError::Unavailable),
            }
        })
    }
}
