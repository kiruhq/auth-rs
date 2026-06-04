use axum::extract::State;
use axum::http::HeaderMap;
use axum::{
    Json,
    http::{StatusCode, header},
};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::adapters::database::DatabaseAdapter;
use crate::auth::token;
use crate::axum::AuthState;
use crate::core::entity::Session;
use crate::types::data::User as ResponseUser;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionResponse {
    session: SessionData,
    user: ResponseUser,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionData {
    id: String,
    user_id: String,
    expires_at: DateTime<Utc>,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

impl From<Session> for SessionData {
    fn from(session: Session) -> Self {
        Self {
            id: session.id,
            user_id: session.user_id,
            expires_at: session.expires_at,
            ip_address: session.ip_address,
            user_agent: session.user_agent,
        }
    }
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    value.strip_prefix("Bearer ").map(|x| x.to_string())
}

pub(crate) async fn session<DB>(
    State(state): State<AuthState<DB>>,
    headers: HeaderMap,
) -> Result<Json<SessionResponse>, StatusCode>
where
    DB: DatabaseAdapter,
{
    let Some(token) = bearer_token(&headers) else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // hash token -> lookup session -> check expiry -> get user
    let Ok(Ok(hashed_token)) =
        tokio::task::spawn_blocking(move || token::hash_secret_token(&token)).await
    else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Ok(Some(session)) = state
        .auth()
        .database
        .get_session_by_token_hash(&hashed_token)
        .await
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if session.expires_at <= Utc::now() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let Ok(Some(user)) = state.auth().database.get_user_by_id(&session.user_id).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    Ok(Json(SessionResponse {
        session: session.into(),
        user: user.into(),
    }))
}

pub(crate) async fn signout<DB>(
    State(state): State<AuthState<DB>>,
    headers: HeaderMap,
) -> StatusCode
where
    DB: DatabaseAdapter,
{
    let Some(token) = bearer_token(&headers) else {
        return StatusCode::OK;
    };

    let Ok(Ok(hashed_token)) =
        tokio::task::spawn_blocking(move || token::hash_secret_token(&token)).await
    else {
        return StatusCode::OK;
    };

    let _ = state
        .auth()
        .database
        .delete_session_by_token_hash(&hashed_token)
        .await;

    StatusCode::OK
}
