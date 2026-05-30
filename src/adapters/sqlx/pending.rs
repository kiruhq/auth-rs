use super::SqlxPostgresAdapter;
use crate::adapters::sqlx::SqlxPostgresTxnAdapter;
use crate::adapters::traits::pending::{
    CreatePendingSignup, CreatePendingSignupError, GetPendingSignupError, PendingSignupStore,
};
use crate::core::entity::PendingSignup;

#[async_trait::async_trait]
impl PendingSignupStore for SqlxPostgresAdapter {
    async fn create_pending_signup(
        &mut self,
        user: CreatePendingSignup,
    ) -> Result<PendingSignup, CreatePendingSignupError> {
        create_pending_signup(&self.conn, user).await
    }

    async fn get_pending_signup_by_id(
        &mut self,
        id: &str,
    ) -> Result<Option<PendingSignup>, GetPendingSignupError> {
        get_pending_signup_by_id(&self.conn, id).await
    }
}

#[async_trait::async_trait]
impl<'a> PendingSignupStore for SqlxPostgresTxnAdapter<'a> {
    async fn create_pending_signup(
        &mut self,
        user: CreatePendingSignup,
    ) -> Result<PendingSignup, CreatePendingSignupError> {
        create_pending_signup(&mut *self.txn, user).await
    }

    async fn get_pending_signup_by_id(
        &mut self,
        id: &str,
    ) -> Result<Option<PendingSignup>, GetPendingSignupError> {
        get_pending_signup_by_id(&mut *self.txn, id).await
    }
}

async fn create_pending_signup<'e, E>(
    executor: E,
    input: CreatePendingSignup,
) -> Result<PendingSignup, CreatePendingSignupError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, PendingSignup>(
        r#"
          INSERT INTO "pending_signup"
          (id, user_id, account_id, name, email, password_hash, image, created_at, updated_at)
          VALUES
          ($1, $2, $3, $4, $5, now(), now())
          RETURNING id, user_id, account_id, name, email, password_hash, image, created_at, updated_at
          "#,
    )
    .bind(input.id)
    .bind(input.user_id)
    .bind(input.account_id)
    .bind(input.name)
    .bind(input.email)
    .bind(input.password_hash)
    .bind(input.image)
    .fetch_one(executor)
    .await
    .map_err(|_| CreatePendingSignupError::Stub)
}

async fn get_pending_signup_by_id<'e, E>(
    executor: E,
    id: &str,
) -> Result<Option<PendingSignup>, GetPendingSignupError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, PendingSignup>(
        r#"
        SELECT id, user_id, account_id, name, email, password_hash, image, created_at, updated_at
        FROM pending_signup
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(|_| GetPendingSignupError::Stub)
}
