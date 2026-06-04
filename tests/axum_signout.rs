#![cfg(all(feature = "axum", feature = "sqlx"))]

mod support;

use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test(flavor = "multi_thread")]
async fn signout_routes_revoke_the_current_session() -> support::TestResult<()> {
    let app = support::TestAuthApp::start().await?;

    for route in [SignOutRoute::DeleteSession, SignOutRoute::SignOut] {
        let token = signup_token(&app, route.email()).await?;
        assert_session_is_active(&app, &token).await?;

        let (status, body) = route.request(&app, &token).await?;
        assert_eq!(status, StatusCode::OK, "{}: {body}", route.name());
        assert_session_is_revoked(&app, &token).await?;
    }

    Ok(())
}

#[derive(Clone, Copy)]
enum SignOutRoute {
    DeleteSession,
    SignOut,
}

impl SignOutRoute {
    fn email(self) -> &'static str {
        match self {
            Self::DeleteSession => "delete-session@example.com",
            Self::SignOut => "sign-out@example.com",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::DeleteSession => "DELETE /session",
            Self::SignOut => "POST /sign-out",
        }
    }

    async fn request(
        self,
        app: &support::TestAuthApp,
        token: &str,
    ) -> support::TestResult<(StatusCode, String)> {
        match self {
            Self::DeleteSession => app.delete_session(token).await,
            Self::SignOut => app.signout(token).await,
        }
    }
}

async fn signup_token(app: &support::TestAuthApp, email: &str) -> support::TestResult<String> {
    let body = app
        .signup_email_ok("Sign Out User", email, "correct horse battery staple")
        .await?;

    Ok(body["token"]
        .as_str()
        .expect("signup should return a session token")
        .to_owned())
}

async fn assert_session_is_active(
    app: &support::TestAuthApp,
    token: &str,
) -> support::TestResult<()> {
    let (status, body) = app.session(token).await?;
    assert_eq!(status, StatusCode::OK, "{body}");

    let body: Value = serde_json::from_str(&body)?;
    assert!(body["session"]["id"].as_str().is_some());
    assert!(body["user"]["id"].as_str().is_some());

    Ok(())
}

async fn assert_session_is_revoked(
    app: &support::TestAuthApp,
    token: &str,
) -> support::TestResult<()> {
    let (status, _body) = app.session(token).await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    Ok(())
}
