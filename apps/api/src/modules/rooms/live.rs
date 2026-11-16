//! Whether an open socket may stay: its session row still stands.

use std::time::Duration;

use salon::Caller;
use tokio::time::Instant;

use crate::state::AppState;

/// How long a socket opened with a bearer JWT (no session row) lives before it must reconnect.
pub const MAX_BEARER_AGE: Duration = Duration::from_secs(15 * 60);

/// `None` keeps the socket; a code and reason close it: the bearer token aged out, or the
/// session ended.
pub async fn alive(state: &AppState, caller: &Caller, opened: Instant) -> Option<(u16, String)> {
    if caller.session_id == 0 {
        return (opened.elapsed() >= MAX_BEARER_AGE).then(|| (1001, "token age, reconnect".to_owned()));
    }
    let live: Result<bool, sqlx::Error> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM porte_sessions WHERE id = $1 AND user_id = $2 \
         AND (expires_at IS NULL OR expires_at > now()))",
    )
    .bind(caller.session_id)
    .bind(caller.who.user_id.unwrap_or_default())
    .fetch_one(&state.pool)
    .await;
    matches!(live, Ok(false)).then(|| (1008, "session ended".to_owned()))
}
