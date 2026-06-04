use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;

use crate::adapters::database::{DatabaseAdapter, DatabaseTransaction};
use crate::adapters::traits::session::{CreateSession, SessionTransactionStore};
use crate::auth::config::ModelName;
use crate::auth::token;
use crate::axum::AuthState;
use crate::core::email::normalize_email;
use crate::types::data::User as ResponseUser;
use crate::types::payload::{EmailSignInBody, EmailSignInResponse};

pub(crate) type SignInResult = Result<Json<EmailSignInResponse>, StatusCode>;

pub(crate) async fn signin<DB>(
    State(state): State<AuthState<DB>>,
    Json(payload): Json<EmailSignInBody>,
) -> SignInResult
where
    DB: DatabaseAdapter,
{
    let auth = state.auth();
    let config = &auth.config.email_and_password;
    if !config.enabled {
        return Err(StatusCode::NOT_FOUND);
    }

    let Ok(email) = normalize_email(&payload.email) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let Ok(user) = auth.database.get_user_by_email(&email).await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Some(user) = user else {
        return Ok(Json(EmailSignInResponse {
            token: None,
            user: None,
        }));
    };

    let Ok(account) = auth.database.get_account("credential", &user.id).await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Some(account) = account else {
        return Ok(Json(EmailSignInResponse {
            token: None,
            user: None,
        }));
    };

    let Some(password_hash) = account.password else {
        return Ok(Json(EmailSignInResponse {
            token: None,
            user: None,
        }));
    };

    let password = payload.password;
    let hasher = config.password_hasher.clone();

    let Ok(Ok(matches)) =
        tokio::task::spawn_blocking(move || hasher.verify(&password, &password_hash)).await
    else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    if !matches {
        return Ok(Json(EmailSignInResponse {
            token: None,
            user: None,
        }));
    }

    let now = Utc::now();
    let session_token = token::generate_secret_token();
    let session_id = auth.generate_id(ModelName::Session);

    let Ok(mut txdb) = auth.database.begin_txn().await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Ok(_) = txdb
        .create_session(CreateSession {
            id: session_id,
            user_id: user.id.to_string(),
            token: session_token.token_hash.clone(),
            expires_at: now + auth.config.session.expires_in,
            ip_address: None,
            user_agent: None,
        })
        .await
    else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    if txdb.commit().await.is_err() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(EmailSignInResponse {
        token: Some(session_token.token),
        user: Some(ResponseUser::from(user)),
    }))
}
