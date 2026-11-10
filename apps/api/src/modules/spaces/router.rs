use axum::Router;
use axum::extract::State;
use axum::routing::get;
use porte::Identity;
use serde::Serialize;
use tronc::Result;
use tronc::json::Json;

use crate::extract::db;
use crate::modules::documents::access::member_spaces;
use crate::state::AppState;

#[derive(Serialize, sqlx::FromRow)]
struct Space {
    id: String,
    name: String,
    role: String,
}

#[derive(Serialize)]
struct Spaces {
    spaces: Vec<Space>,
}

async fn list(State(state): State<AppState>, who: Identity) -> Result<Json<Spaces>> {
    let spaces = sqlx::query_as::<_, Space>(concat!(
        "SELECT s.id, s.name, m.role FROM spaces s JOIN (",
        member_spaces!(),
        ") m ON m.space_id = s.id ORDER BY lower(s.name), s.id"
    ))
    .bind(who.user_id)
    .bind(&*state.issuer)
    .fetch_all(&state.pool)
    .await
    .map_err(db)?;
    Ok(Json(Spaces { spaces }))
}

/// `GET /spaces`, behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new().route("/spaces", get(list))
}
