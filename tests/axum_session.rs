#![cfg(all(feature = "axum", feature = "sqlx"))]

mod support;

use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test(flavor = "multi_thread")]
async fn session_returns_current_user_and_session_for_bearer_token() -> support::TestResult<()> {
    let app = support::TestAuthApp::start().await?;

    let signup = app
        .signup_email_ok(
            "Session User",
            "session@example.com",
            "correct horse battery staple",
        )
        .await?;
    let token = signup["token"]
        .as_str()
        .expect("signup should return a session token");
    let user_id = signup["user"]["id"]
        .as_str()
        .expect("signup should return a user id");

    let (status, body) = app.session(token).await?;
    assert_eq!(status, StatusCode::OK, "{body}");

    let body: Value = serde_json::from_str(&body)?;
    assert_eq!(body["user"]["id"], user_id);
    assert_eq!(body["user"]["name"], "Session User");
    assert_eq!(body["user"]["email"], "session@example.com");
    assert_eq!(body["user"]["emailVerified"], false);

    assert!(
        body["session"]["id"]
            .as_str()
            .expect("session should have an id")
            .starts_with("session_")
    );
    assert_eq!(body["session"]["userId"], user_id);
    assert!(body["session"]["expiresAt"].as_str().is_some());
    assert!(body["session"]["ipAddress"].is_null());
    assert!(body["session"]["userAgent"].is_null());
    assert!(body["session"].get("token").is_none());

    Ok(())
}
