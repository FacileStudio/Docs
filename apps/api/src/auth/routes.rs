use axum::Router;
use axum::extract::State;
use axum::routing::get;
use porte::Identity;
use serde::Serialize;
use tronc::json::Json;

use super::Kits;
use crate::state::AppState;

#[derive(Serialize, Default)]
struct SessionUser {
    email: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    avatar_url: String,
}

#[derive(Serialize)]
struct SessionAnswer {
    user: Option<SessionUser>,
}

async fn account(state: &AppState, identity: &Identity) -> SessionUser {
    let row: Option<(String, String, String)> = sqlx::query_as(
        "SELECT u.email, u.name, COALESCE(NULLIF(u.avatar_url, ''), (SELECT m.picture FROM space_members m
             JOIN porte_identities i ON i.subject = m.subject AND i.provider = $2
             WHERE i.user_id = u.id AND m.picture <> '' LIMIT 1), '')
         FROM porte_users u WHERE u.id = $1",
    )
    .bind(identity.user_id)
    .bind(&*state.issuer)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();
    match row {
        Some((email, name, avatar_url)) => SessionUser { email, name, avatar_url },
        None => SessionUser { email: identity.email.clone(), name: identity.name.clone(), ..SessionUser::default() },
    }
}

async fn me(State(state): State<AppState>, identity: Identity) -> Json<SessionUser> {
    Json(account(&state, &identity).await)
}

async fn session(State(state): State<AppState>, identity: Option<Identity>) -> Json<SessionAnswer> {
    let user = match identity {
        Some(identity) => Some(account(&state, &identity).await),
        None => None,
    };
    Json(SessionAnswer { user })
}

/// porte's routes, `GET /auth/me` for a CLI checking its token, and `GET /auth/session`, which
/// answers 200 with `{"user": null}` for an anonymous browser.
pub fn routes(kits: &Kits) -> Router<AppState> {
    Router::new()
        .merge(kits.sessions.routes())
        .merge(kits.oidc.routes())
        .route("/auth/me", get(me).route_layer(kits.sessions.require_auth()))
        .route("/auth/session", get(session).route_layer(kits.sessions.optional()))
}
