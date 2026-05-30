use crate::adapters::database::DatabaseAdapter;
use crate::adapters::traits::{AccountStore, CreateAccount, CreateUser, UserStore};
use crate::auth::config::{EmailAndPasswordConfig, ModelName};
use crate::axum::AuthState;
use crate::types::payload::{EmailSignInBody, EmailSignUpBody};
use argon2::password_hash;
use axum::{Json, extract::State, http::StatusCode};
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

    // 5. Normalize email.
    // Better Auth lowercases the email before lookup and storage.
    let Ok(normalized_email) = normalize_email(&payload.email) else {
        return StatusCode::BAD_REQUEST;
    };

    let password = payload.password.clone();
    let hasher = config.password_hasher.clone();
    let Ok(Ok(hashed_password)) = tokio::task::spawn_blocking(move || hasher.hash(&password)).await
    else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    // 2. Start a database transaction.
    // Everything from user lookup through account/session creation should be
    // atomic so a user row is not left without its credential account row.
    //
    // 6. Look up an existing user by normalized email.

    // 7. If the user already exists.
    // If require_email_verification is true or auto_sign_in is false, return a
    // generic success response to avoid email enumeration:
    // - hash the submitted password anyway to reduce timing differences
    // - optionally call on_existing_user_sign_up
    // - return { token: null, user: synthetic_user }
    // Otherwise return 422 user already exists.
    // 8. If the user does not exist.
    // Hash the password before creating the user so hashing failures happen
    // before any database writes. Then create:
    // - user row with email_verified = false
    // - account row with provider_id = "credential"
    // - account_id = user.id
    // - user_id = user.id
    // - password = hash

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

    let input = CreateUser {
        id: user_id.clone(),
        name: payload.name,
        email: normalized_email,
        image: None,
    };

    let Ok(_) = txdb.create_user(input).await else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let input = CreateAccount {
        id: account_id,
        account_id: user_id.clone(),
        user_id,
        provider_id: "credential".to_string(),
        password: Some(hashed_password),
    };

    let Ok(_) = txdb.create_account(input).await else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

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
    StatusCode::OK
}

pub(crate) async fn signin<DB>(
    State(_auth): State<AuthState<DB>>,
    Json(_payload): Json<EmailSignInBody>,
) -> StatusCode
where
    DB: DatabaseAdapter,
{
    StatusCode::OK
}
