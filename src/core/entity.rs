use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub(crate) struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub image: Option<String>,
    #[serde(skip)]
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub(crate) struct Account {
    pub id: String,
    pub account_id: String,
    pub user_id: String,
    pub provider_id: String,
    pub password: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub(crate) struct PendingSignup {
    pub id: String,
    pub user_id: String,
    pub account_id: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub image: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub(crate) struct Verification {
    pub id: String,
    pub kind: String,
    pub identifier: String,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub(crate) struct Session {
    pub id: String,
    pub user_id: String,
    #[serde(skip)]
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    #[serde(skip)]
    pub created_at: DateTime<Utc>,
    #[serde(skip)]
    pub updated_at: DateTime<Utc>,
}
