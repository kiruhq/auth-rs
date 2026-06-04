#![cfg(all(feature = "axum", feature = "sqlx"))]

mod support;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::Utc;

#[tokio::test(flavor = "multi_thread")]
async fn email_signup_creates_user_account_and_session() -> support::TestResult<()> {
    let app = support::TestAuthApp::start().await?;

    let password = "correct horse battery staple";
    let body = app
        .signup_email_ok("Test User", " Test.User@Example.COM ", password)
        .await?;
    let token = body["token"]
        .as_str()
        .expect("signup should return a token");
    let response_user = &body["user"];
    assert_eq!(response_user["name"], "Test User");
    assert_eq!(response_user["email"], "test.user@example.com");
    assert_eq!(response_user["emailVerified"], false);
    assert!(response_user["image"].is_null());

    let user = app.user_by_email("test.user@example.com").await?;

    assert_eq!(response_user["id"], user.id);
    assert_eq!(user.name, "Test User");
    assert_eq!(user.email, "test.user@example.com");
    assert!(!user.email_verified);
    assert!(user.image.is_none());

    let account = app.account_by_user_id(&user.id).await?;

    assert!(account.id.starts_with("account_"));
    assert_eq!(account.account_id, user.id);
    assert_eq!(account.user_id, user.id);
    assert_eq!(account.provider_id, "credential");

    let password_hash = account
        .password
        .expect("account should store password hash");
    assert_ne!(password_hash, password);
    let parsed_hash = PasswordHash::new(&password_hash)?;
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash)?;

    let expected_token_hash = support::hash_session_token(token)?;
    let session = app.session_by_token_hash(&expected_token_hash).await?;

    assert_eq!(session.user_id, user.id);
    assert_eq!(session.token, expected_token_hash);
    assert!(session.expires_at > Utc::now());

    Ok(())
}
