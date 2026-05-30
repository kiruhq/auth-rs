use crate::types::data::User;

pub(crate) use sqlx::Postgres;

use super::{
    CreateUserWithCredentialAccountInput, CreateUserWithCredentialAccountResult, Store, StoreError,
};

pub struct SqlxStore<DB: sqlx::Database> {
    pool: sqlx::Pool<DB>,
}

impl SqlxStore<sqlx::Postgres> {
    pub fn postgres(pool: sqlx::PgPool) -> SqlxStore<sqlx::Postgres> {
        SqlxStore { pool }
    }
}

impl From<sqlx::Error> for StoreError {
    fn from(value: sqlx::Error) -> Self {
        Self::DriverError(Box::new(value))
    }
}

impl Store for SqlxStore<sqlx::Postgres> {
    async fn create_user_with_credential_account(
        &self,
        input: CreateUserWithCredentialAccountInput,
    ) -> Result<CreateUserWithCredentialAccountResult, StoreError> {
        let mut txn = self.pool.begin().await?;

        let exists: bool =
            sqlx::query_scalar(r#"SELECT EXISTS (SELECT 1 FROM "user" WHERE email = $1)"#)
                .bind(&input.email)
                .fetch_one(&mut *txn)
                .await?;

        if exists {
            return Ok(CreateUserWithCredentialAccountResult::EmailAlreadyExists);
        }

        let user: User = sqlx::query_as(
            r#"
INSERT INTO "user"
(id, name, email, email_verified, image, created_at, updated_at)
VALUES
($1, $2, $3, $4, null, now(), now())
RETURNING id, name, email, email_verified, image, created_at, updated_at"#,
        )
        .bind(&input.user_id)
        .bind(&input.name)
        .bind(&input.email)
        .bind(false)
        .fetch_one(&mut *txn)
        .await?;

        sqlx::query(
            r#"
INSERT INTO account
(id, account_id, user_id, provider_id, password, created_at, updated_at)
VALUES
($1, $2, $2, $3, $5, now(), now())
"#,
        )
        .bind(&input.account_id)
        .bind(&input.user_id)
        .bind("credential")
        .bind(&input.hashed_password)
        .execute(&mut *txn)
        .await?;

        txn.commit().await?;

        Ok(CreateUserWithCredentialAccountResult::Created { user })
    }
}
