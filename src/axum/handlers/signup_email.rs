use crate::Auth;
use crate::adapters::database::{DatabaseAdapter, DatabaseTransaction};
use crate::adapters::traits::pending::CreatePendingSignup;
use crate::adapters::traits::session::CreateSession;
use crate::adapters::traits::verification::CreateVerification;
use crate::adapters::traits::{
    account::CreateAccount,
    user::{CreateUser, UserStore},
};
use crate::auth::config::{
    EmailAndPasswordConfig, ModelName, SendVerificationEmail, VerificationEmailUser,
};
use crate::auth::token;
use crate::auth::verification;
use crate::axum::AuthState;
use crate::types::data::{EmailSignUpResponse, User as ResponseUser};
use crate::types::payload::EmailSignUpBody;
use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use email_address::{EmailAddress, Options};

type SignUpResult = Result<Json<EmailSignUpResponse>, StatusCode>;

enum EmailSignUpValidationError {
    PasswordTooShort,
    PasswordTooLong,
    InvalidPassword,
    InvalidEmail,
}

fn validate_signup_input(
    config: &EmailAndPasswordConfig,
    payload: &EmailSignUpBody,
) -> Result<(), EmailSignUpValidationError> {
    if payload.password.is_empty() {
        return Err(EmailSignUpValidationError::InvalidPassword);
    }

    if payload.password.len() < config.min_password_length as usize {
        return Err(EmailSignUpValidationError::PasswordTooShort);
    }

    if payload.password.len() > config.max_password_length as usize {
        return Err(EmailSignUpValidationError::PasswordTooLong);
    }

    if !email_address::EmailAddress::is_valid(&payload.email) {
        return Err(EmailSignUpValidationError::InvalidEmail);
    }

    Ok(())
}

fn normalize_email(email: &str) -> Result<String, EmailSignUpValidationError> {
    let email = email.trim().to_lowercase();

    let options = Options::default()
        .without_display_text()
        .without_domain_literal();

    EmailAddress::parse_with_options(&email, options)
        .map_err(|_| EmailSignUpValidationError::InvalidEmail)?;

    Ok(email)
}

pub(crate) async fn signup<DB>(
    State(state): State<AuthState<DB>>,
    Json(payload): Json<EmailSignUpBody>,
) -> SignUpResult
where
    DB: DatabaseAdapter,
{
    let auth = state.auth();
    let config = &auth.config.email_and_password;

    // Check email/password config.
    // Reject if email_and_password.enabled is false or disable_sign_up is true.
    if !config.enabled || config.disable_sign_up {
        return Err(StatusCode::NOT_FOUND);
    }

    // TODO - Handle this properly
    if validate_signup_input(config, &payload).is_err() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let Ok(normalized_email) = normalize_email(&payload.email) else {
        return Err(StatusCode::BAD_REQUEST);
    };
    let callback_url = payload.callback_url.clone();

    let password = payload.password.clone();
    let hasher = config.password_hasher.clone();
    let Ok(Ok(hashed_password)) = tokio::task::spawn_blocking(move || hasher.hash(&password)).await
    else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let user_id = auth.generate_id(ModelName::User);
    let account_id = auth.generate_id(ModelName::Account);

    let Ok(mut txdb) = auth.database.begin_txn().await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Ok(result) = txdb.get_user_by_email(&normalized_email).await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    if result.is_some() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    };

    let input = SignupInput {
        account_id,
        user_id,
        hashed_password,
        normalized_email,
        user_name: payload.name,
        callback_url,
    };

    if config.require_email_verification {
        return signup_verification_required(auth, txdb, input).await;
    }

    signup_direct(auth, txdb, input).await
}

struct SignupInput {
    account_id: String,
    user_id: String,
    hashed_password: String,
    normalized_email: String,
    user_name: String,
    callback_url: Option<String>,
}

async fn signup_verification_required<DB, T>(
    auth: &Auth<DB>,
    mut txdb: T,
    input: SignupInput,
) -> SignUpResult
where
    DB: DatabaseAdapter,
    T: DatabaseTransaction,
{
    let pending_id = auth.generate_id(ModelName::PendingSignup);

    let Ok(pending) = txdb
        .create_pending_signup(CreatePendingSignup {
            id: pending_id,
            account_id: input.account_id,
            user_id: input.user_id,
            password_hash: input.hashed_password,
            email: input.normalized_email,
            name: input.user_name,
            image: None,
        })
        .await
    else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let verification_id = auth.generate_id(ModelName::Verification);
    let token = token::generate_secret_token();
    let now = Utc::now();

    let Ok(_verification) = txdb
        .create_verification(CreateVerification {
            id: verification_id,
            kind: ModelName::PendingSignup.to_string(),
            identifier: pending.id,
            expires_at: now + auth.config.email_verification.expires_in,
            token_hash: token.token_hash,
        })
        .await
    else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Some(sender) = &auth.config.email_verification.send_verification_email else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    if txdb.commit().await.is_err() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let Ok(verification_url) = verification::create_verification_url(
        &auth.config.base_url,
        &auth.config.base_path,
        &token.token,
        input.callback_url.as_deref(),
    ) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    if sender
        .send_verification_email(SendVerificationEmail {
            user: VerificationEmailUser {
                id: pending.user_id,
                email: pending.email,
                name: pending.name,
                image: pending.image,
            },
            url: verification_url,
            token: token.token,
        })
        .await
        .is_err()
    {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(EmailSignUpResponse {
        token: None,
        user: None,
    }))
}

async fn signup_direct<DB, T>(auth: &Auth<DB>, mut txdb: T, input: SignupInput) -> SignUpResult
where
    DB: DatabaseAdapter,
    T: DatabaseTransaction,
{
    let user_input = CreateUser {
        id: input.user_id,
        name: input.user_name,
        email: input.normalized_email,
        image: None,
    };

    let Ok(user) = txdb.create_user(user_input).await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let create_account_input = CreateAccount {
        id: input.account_id,
        account_id: user.id.clone(),
        user_id: user.id.clone(),
        provider_id: "credential".to_string(),
        password: Some(input.hashed_password),
    };

    let Ok(_) = txdb.create_account(create_account_input).await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let now = Utc::now();
    let verification_token = if auth.config.email_verification.send_on_signup {
        Some(token::generate_secret_token())
    } else {
        None
    };

    if let Some(verification_token) = &verification_token {
        let verification_id = auth.generate_id(ModelName::Verification);

        let Ok(_) = txdb
            .create_verification(CreateVerification {
                id: verification_id,
                kind: ModelName::User.to_string(),
                identifier: user.id.to_string(),
                expires_at: now + auth.config.email_verification.expires_in,
                token_hash: verification_token.token_hash.clone(),
            })
            .await
        else {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };
    }

    let session_token = if auth.config.email_and_password.auto_sign_in {
        let session_token = token::generate_secret_token();
        let session_id = auth.generate_id(ModelName::Session);

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

        Some(session_token.token)
    } else {
        None
    };

    if txdb.commit().await.is_err() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if let Some(verification_token) = verification_token {
        let Some(sender) = &auth.config.email_verification.send_verification_email else {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };

        let Ok(verification_url) = verification::create_verification_url(
            &auth.config.base_url,
            &auth.config.base_path,
            &verification_token.token,
            input.callback_url.as_deref(),
        ) else {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };

        let _ = sender
            .send_verification_email(SendVerificationEmail {
                user: VerificationEmailUser {
                    id: user.id.clone(),
                    email: user.email.clone(),
                    name: user.name.clone(),
                    image: user.image.clone(),
                },
                url: verification_url,
                token: verification_token.token,
            })
            .await;
    }

    Ok(Json(EmailSignUpResponse {
        token: session_token,
        user: Some(ResponseUser::from(user)),
    }))
}
