use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::core::entity;

pub(crate) struct Account {
    id: String,
    user_id: String,
    account_id: String,
    provider_id: String,
    password: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[serde(rename_all = "camelCase")]
pub(crate) struct User {
    id: String,
    name: String,
    email: String,
    email_verified: bool,
    image: Option<String>,
    #[serde(skip)]
    created_at: DateTime<Utc>,
    #[serde(skip)]
    updated_at: DateTime<Utc>,
}

impl User {
    pub(crate) fn new(
        id: String,
        name: String,
        email: String,
        email_verified: bool,
        image: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            email,
            email_verified,
            image,
            created_at,
            updated_at,
        }
    }
}

impl From<entity::User> for User {
    fn from(user: entity::User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            email_verified: user.email_verified,
            image: user.image,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
