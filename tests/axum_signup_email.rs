#![cfg(all(feature = "axum", feature = "sqlx"))]

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use auth_rs::Auth;
use axum::Router;
use base64::prelude::*;
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, postgres::PgPoolOptions};
use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};

#[derive(Debug, sqlx::FromRow)]
struct UserRow {
    id: String,
    name: String,
    email: String,
    email_verified: bool,
    image: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct AccountRow {
    id: String,
    account_id: String,
    user_id: String,
    provider_id: String,
    password: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct SessionRow {
    user_id: String,
    token: String,
    expires_at: DateTime<Utc>,
}

#[tokio::test(flavor = "multi_thread")]
async fn email_signup_creates_user_account_and_session() -> Result<(), Box<dyn std::error::Error>> {
    let container = postgres::Postgres::default()
        .with_db_name("auth_rs_test")
        .with_init_sql(auth_rs::schema::postgres_base_schema_sql().into_bytes())
        .start()
        .await?;

    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(5432).await?;
    let database_url = format!("postgres://postgres:postgres@{host}:{port}/auth_rs_test");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let server_url = spawn_auth_server(pool.clone()).await?;

    let password = "correct horse battery staple";
    let response = reqwest::Client::new()
        .post(format!("{server_url}/api/auth/sign-up/email"))
        .json(&json!({
            "name": "Test User",
            "email": " Test.User@Example.COM ",
            "password": password,
            "image": null,
        }))
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;
    assert_eq!(status, StatusCode::OK, "{body}");

    let body: Value = serde_json::from_str(&body)?;
    let token = body["token"]
        .as_str()
        .expect("signup should return a token");
    let response_user = &body["user"];
    assert_eq!(response_user["name"], "Test User");
    assert_eq!(response_user["email"], "test.user@example.com");
    assert_eq!(response_user["emailVerified"], false);
    assert!(response_user["image"].is_null());

    let user: UserRow = sqlx::query_as(
        r#"
        SELECT id, name, email, email_verified, image
        FROM "user"
        WHERE email = $1
        "#,
    )
    .bind("test.user@example.com")
    .fetch_one(&pool)
    .await?;

    assert_eq!(response_user["id"], user.id);
    assert_eq!(user.name, "Test User");
    assert_eq!(user.email, "test.user@example.com");
    assert!(!user.email_verified);
    assert!(user.image.is_none());

    let account: AccountRow = sqlx::query_as(
        r#"
        SELECT id, account_id, user_id, provider_id, password
        FROM account
        WHERE user_id = $1
        "#,
    )
    .bind(&user.id)
    .fetch_one(&pool)
    .await?;

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

    let expected_token_hash = hash_session_token(token)?;
    let session: SessionRow = sqlx::query_as(
        r#"
        SELECT user_id, token, expires_at
        FROM "session"
        WHERE token = $1
        "#,
    )
    .bind(&expected_token_hash)
    .fetch_one(&pool)
    .await?;

    assert_eq!(session.user_id, user.id);
    assert_eq!(session.token, expected_token_hash);
    assert!(session.expires_at > Utc::now());

    Ok(())
}

async fn spawn_auth_server(pool: PgPool) -> Result<String, Box<dyn std::error::Error>> {
    let auth = Auth::builder()
        .config(|config| {
            config.base_url = "http://127.0.0.1".to_owned();
        })
        .email_and_password(|config| {
            config.enabled = true;
            config.require_email_verification = false;
            config.auto_sign_in = true;
        })
        .sqlx(pool)
        .build()?;

    let app = Router::new().nest("/api/auth", auth_rs::axum::router(auth));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;

    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("test auth server should run");
    });

    Ok(format!("http://{address}"))
}

fn hash_session_token(token: &str) -> Result<String, base64::DecodeError> {
    let bytes = BASE64_URL_SAFE_NO_PAD.decode(token)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}
