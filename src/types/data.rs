use chrono::{DateTime, Utc};

pub(crate) struct Account {
    id: String,
    user_id: String,
    account_id: String,
    provider_id: String,
    password: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

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
