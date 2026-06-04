#![allow(dead_code)]

use auth_rs::Auth;
use axum::Router;
use base64::prelude::*;
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, postgres::PgPoolOptions};
use testcontainers_modules::{
    postgres,
    testcontainers::{ContainerAsync, runners::AsyncRunner},
};

pub type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct TestAuthApp {
    _container: ContainerAsync<postgres::Postgres>,
    pub pool: PgPool,
    pub client: Client,
    pub server_url: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
    pub id: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub image: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AccountRow {
    pub id: String,
    pub account_id: String,
    pub user_id: String,
    pub provider_id: String,
    pub password: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct SessionRow {
    pub user_id: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

impl TestAuthApp {
    pub async fn start() -> TestResult<Self> {
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

        Ok(Self {
            _container: container,
            pool,
            client: Client::new(),
            server_url,
        })
    }

    pub async fn signup_email(
        &self,
        name: &str,
        email: &str,
        password: &str,
    ) -> TestResult<(StatusCode, String)> {
        self.post_auth_json(
            "/sign-up/email",
            json!({
                "name": name,
                "email": email,
                "password": password,
                "image": null,
            }),
        )
        .await
    }

    pub async fn signup_email_ok(
        &self,
        name: &str,
        email: &str,
        password: &str,
    ) -> TestResult<Value> {
        let (status, body) = self.signup_email(name, email, password).await?;
        assert_eq!(status, StatusCode::OK, "{body}");
        Ok(serde_json::from_str(&body)?)
    }

    pub async fn signin_email(
        &self,
        email: &str,
        password: &str,
    ) -> TestResult<(StatusCode, String)> {
        self.post_auth_json(
            "/sign-in/email",
            json!({
                "email": email,
                "password": password,
            }),
        )
        .await
    }

    pub async fn session(&self, token: &str) -> TestResult<(StatusCode, String)> {
        let response = self
            .client
            .get(format!("{}/api/auth/session", self.server_url))
            .bearer_auth(token)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        Ok((status, body))
    }

    pub async fn delete_session(&self, token: &str) -> TestResult<(StatusCode, String)> {
        let response = self
            .client
            .delete(format!("{}/api/auth/session", self.server_url))
            .bearer_auth(token)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        Ok((status, body))
    }

    pub async fn signout(&self, token: &str) -> TestResult<(StatusCode, String)> {
        let response = self
            .client
            .post(format!("{}/api/auth/sign-out", self.server_url))
            .bearer_auth(token)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        Ok((status, body))
    }

    pub async fn user_by_email(&self, email: &str) -> TestResult<UserRow> {
        Ok(sqlx::query_as(
            r#"
            SELECT id, name, email, email_verified, image
            FROM "user"
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?)
    }

    pub async fn account_by_user_id(&self, user_id: &str) -> TestResult<AccountRow> {
        Ok(sqlx::query_as(
            r#"
            SELECT id, account_id, user_id, provider_id, password
            FROM account
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?)
    }

    pub async fn session_by_token_hash(&self, token_hash: &str) -> TestResult<SessionRow> {
        Ok(sqlx::query_as(
            r#"
            SELECT user_id, token, expires_at
            FROM "session"
            WHERE token = $1
            "#,
        )
        .bind(token_hash)
        .fetch_one(&self.pool)
        .await?)
    }

    async fn post_auth_json(&self, path: &str, body: Value) -> TestResult<(StatusCode, String)> {
        let response = self
            .client
            .post(format!("{}/api/auth{}", self.server_url, path))
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        Ok((status, body))
    }
}

pub fn hash_session_token(token: &str) -> Result<String, base64::DecodeError> {
    let bytes = BASE64_URL_SAFE_NO_PAD.decode(token)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

async fn spawn_auth_server(pool: PgPool) -> TestResult<String> {
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
