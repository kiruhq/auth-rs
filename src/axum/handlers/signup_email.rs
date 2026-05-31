use crate::Auth;
use crate::adapters::database::{DatabaseAdapter, DatabaseTransaction};
use crate::adapters::traits::pending::{CreatePendingSignup, PendingSignupStore};
use crate::adapters::traits::verification::{CreateVerification, VerificationStore};
use crate::adapters::traits::{
    account::{AccountStore, CreateAccount},
    user::{CreateUser, UserStore},
};
use crate::auth::config::{
    EmailAndPasswordConfig, ModelName, SendVerificationEmail, VerificationEmailUser,
};
use crate::auth::verification;
use crate::axum::AuthState;
use crate::core::entity::Verification;
use crate::types::payload::{EmailSignInBody, EmailSignUpBody};
use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use email_address::{EmailAddress, Options};

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
) -> StatusCode
where
    DB: DatabaseAdapter,
{
    let auth = state.auth();
    let config = &auth.config.email_and_password;

    // Check email/password config.
    // Reject if email_and_password.enabled is false or disable_sign_up is true.
    if !config.enabled || config.disable_sign_up {
        return StatusCode::NOT_FOUND;
    }

    // TODO - Handle this properly
    if validate_signup_input(config, &payload).is_err() {
        return StatusCode::BAD_REQUEST;
    }

    let Ok(normalized_email) = normalize_email(&payload.email) else {
        return StatusCode::BAD_REQUEST;
    };
    let callback_url = payload.callback_url.clone();

    let password = payload.password.clone();
    let hasher = config.password_hasher.clone();
    let Ok(Ok(hashed_password)) = tokio::task::spawn_blocking(move || hasher.hash(&password)).await
    else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let user_id = auth.generate_id(ModelName::User);
    let account_id = auth.generate_id(ModelName::Account);

    let Ok(mut txdb) = auth.database.begin_txn().await else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let Ok(result) = txdb.get_user_by_email(&normalized_email).await else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    if result.is_some() {
        return StatusCode::UNPROCESSABLE_ENTITY;
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

    // 9. Send verification email if configured.
    // Better Auth uses:
    // email_verification.send_on_sign_up ?? email_and_password.require_email_verification
    // If true, create a verification token, build /verify-email?token=... with
    // callbackURL, and call the configured send_verification_email callback.

    // 10. Decide whether to auto sign in.
    // Skip auto sign-in when auto_sign_in is false, or when the generic
    // duplicate-email response path is active. Return { token: null, user }.

    // 11. Otherwise create a session.
    // Create the session, set the session cookie, and return
    // { token: session.token, user }.
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
) -> StatusCode
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
        return StatusCode::BAD_REQUEST;
    };

    let verification_id = auth.generate_id(ModelName::Verification);
    let token = verification::generate_verification_token();

    let Ok(_verification) = txdb
        .create_verification(CreateVerification {
            id: verification_id,
            kind: ModelName::PendingSignup.to_string(),
            identifier: pending.id,
            expires_at: Utc::now(),
            token_hash: token.token_hash,
        })
        .await
    else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let Some(sender) = &auth.config.email_verification.send_verification_email else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    if txdb.commit().await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    let Ok(verification_url) = verification::create_verification_url(
        &auth.config.base_url,
        &auth.config.base_path,
        &token.encoded_token,
        input.callback_url.as_deref(),
    ) else {
        return StatusCode::INTERNAL_SERVER_ERROR;
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
            token: token.encoded_token,
        })
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

async fn signup_direct<DB, T>(auth: &Auth<DB>, mut txdb: T, input: SignupInput) -> StatusCode
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
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let create_account_input = CreateAccount {
        id: input.account_id,
        account_id: user.id.clone(),
        user_id: user.id.clone(),
        provider_id: "credential".to_string(),
        password: Some(input.hashed_password),
    };

    let Ok(_) = txdb.create_account(create_account_input).await else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let verification: Option<Verification>;
    let token = verification::generate_verification_token();

    if auth.config.email_verification.send_on_signup
        && let Some(sender) = &auth.config.email_verification.send_verification_email
    {
        let verification_id = auth.generate_id(ModelName::Verification);

        if let Ok(v) = txdb
            .create_verification(CreateVerification {
                id: verification_id,
                kind: ModelName::User.to_string(),
                identifier: user.id.to_string(),
                expires_at: Utc::now(),
                token_hash: token.token_hash,
            })
            .await
        {
            verification = Some(v);
        } else {
            verification = None;
        }
    } else {
        verification = None
    }

    if txdb.commit().await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if let Some(_) = verification
        && let Some(sender) = &auth.config.email_verification.send_verification_email
    {
        let Ok(verification_url) = verification::create_verification_url(
            &auth.config.base_url,
            &auth.config.base_path,
            &token.encoded_token,
            input.callback_url.as_deref(),
        ) else {
            return StatusCode::INTERNAL_SERVER_ERROR;
        };

        let _ = sender
            .send_verification_email(SendVerificationEmail {
                user: VerificationEmailUser {
                    id: user.id,
                    email: user.email,
                    name: user.name,
                    image: user.image,
                },
                url: verification_url,
                token: token.encoded_token,
            })
            .await;
    }

    StatusCode::OK
}
