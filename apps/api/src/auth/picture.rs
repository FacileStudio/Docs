//! Registre's `picture` claim as the account's avatar: porte-rs stores no avatars of its own.

use std::sync::Arc;

use porte::{Claims, StoreError, UserStore};

/// porte's user store, with the `https` `picture` claim copied into `avatar_url` on every sign-in.
pub struct Pictures {
    inner: Arc<dyn UserStore>,
}

impl Pictures {
    /// Wraps `inner`, the store porte would write to.
    pub fn wrap(inner: Arc<dyn UserStore>) -> Arc<dyn UserStore> {
        Arc::new(Pictures { inner })
    }
}

#[async_trait::async_trait]
impl UserStore for Pictures {
    async fn upsert_from_oidc(&self, claims: &Claims) -> Result<i64, StoreError> {
        if !claims.avatar_url.is_empty() || !claims.picture.starts_with("https://") {
            return self.inner.upsert_from_oidc(claims).await;
        }
        let pictured = Claims { avatar_url: claims.picture.clone(), ..claims.clone() };
        self.inner.upsert_from_oidc(&pictured).await
    }
}
