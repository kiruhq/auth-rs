use chrono::{DateTime, Utc};

#[derive(Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub(crate) struct User {
    id: String,
    name: String,
    email: String,
    email_verified: bool,
    image: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub(crate) struct Account {
    id: String,
    account_id: String,
    user_id: String,
    provider_id: String,
    password: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

pub(crate) struct Session {}
