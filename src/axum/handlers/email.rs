use crate::axum::AuthState;
use crate::types::payload::{EmailSignInBody, EmailSignUpBody};
use axum::{Json, extract::State, http::StatusCode};

pub(crate) async fn signup(
    State(_auth): State<AuthState>,
    Json(_payload): Json<EmailSignUpBody>,
) -> StatusCode {
    // 2. Start a database transaction.
    // Everything from user lookup through account/session creation should be
    // atomic so a user row is not left without its credential account row.

    // 3. Check email/password config.
    // Reject if email_and_password.enabled is false or disable_sign_up is true.

    // 4. Validate input.
    // Required: name, email, password. Validate email format, require password
    // to be present, and enforce configured min/max password length.

    // 5. Normalize email.
    // Better Auth lowercases the email before lookup and storage.

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

pub(crate) async fn signin(
    State(_auth): State<AuthState>,
    Json(_payload): Json<EmailSignInBody>,
) -> StatusCode {
    StatusCode::OK
}
