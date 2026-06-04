#![cfg(all(feature = "axum", feature = "sqlx"))]

mod support;

use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test(flavor = "multi_thread")]
async fn email_signin_returns_user_and_session_for_existing_account() -> support::TestResult<()> {
    let app = support::TestAuthApp::start().await?;
    let password = "correct horse battery staple";

    app.signup_email_ok("Sign In User", "signin@example.com", password)
        .await?;

    let (status, body) = app.signin_email("signin@example.com", password).await?;
    assert_eq!(status, StatusCode::OK, "{body}");

    let body: Value = serde_json::from_str(&body)?;
    assert!(body["token"].as_str().is_some());
    assert_eq!(body["user"]["email"], "signin@example.com");

    Ok(())
}
