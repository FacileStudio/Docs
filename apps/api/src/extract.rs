//! Request helpers that answer the suite envelope instead of axum's plain-text rejections.

use axum::extract::{FromRequestParts, Query};
use http::request::Parts;
use serde::de::DeserializeOwned;
use uuid::Uuid;

/// Query parameters; a malformed query is `400 invalid_argument`.
pub struct Params<T>(pub T);

impl<T: DeserializeOwned, S: Send + Sync> FromRequestParts<S> for Params<T> {
    type Rejection = tronc::Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Query::<T>::from_request_parts(parts, state)
            .await
            .map(|Query(value)| Params(value))
            .map_err(|rejection| tronc::Error::invalid(rejection.body_text()))
    }
}

/// A path id. Anything that is not a UUID names nothing, so it is `404 not_found`.
pub fn id(raw: &str) -> tronc::Result<Uuid> {
    Uuid::parse_str(raw).map_err(|_| tronc::Error::not_found("not found"))
}

/// A database failure, logged with its cause and answered as `500 internal`.
pub fn db(error: sqlx::Error) -> tronc::Error {
    tronc::Error::internal("database error", error)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_malformed_id_is_not_found() {
        assert_eq!(super::id("nope").unwrap_err().code, tronc::Code::NotFound);
        assert!(super::id("6f1e2d3c-4b5a-4968-8776-655443322110").is_ok());
    }
}
