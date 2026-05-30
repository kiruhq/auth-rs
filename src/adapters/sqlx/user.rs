use super::SqlxPostgresAdapter;
use crate::adapters::sqlx::SqlxPostgresTxnAdapter;
use crate::adapters::traits::user::{CreateUser, CreateUserError, GetUserError, UserStore};
use crate::core::entity::User;

#[async_trait::async_trait]
impl UserStore for SqlxPostgresAdapter {
    async fn create_user(&mut self, user: CreateUser) -> Result<User, CreateUserError> {
        create_user(&self.conn, user).await
    }

    async fn get_user_by_id(&mut self, id: &str) -> Result<Option<User>, GetUserError> {
        get_user_by_id(&self.conn, id).await
    }

    async fn get_user_by_email(&mut self, email: &str) -> Result<Option<User>, GetUserError> {
        get_user_by_email(&self.conn, email).await
    }
}

#[async_trait::async_trait]
impl<'a> UserStore for SqlxPostgresTxnAdapter<'a> {
    async fn create_user(&mut self, user: CreateUser) -> Result<User, CreateUserError> {
        create_user(&mut *self.txn, user).await
    }

    async fn get_user_by_id(&mut self, id: &str) -> Result<Option<User>, GetUserError> {
        get_user_by_id(&mut *self.txn, id).await
    }

    async fn get_user_by_email(&mut self, email: &str) -> Result<Option<User>, GetUserError> {
        get_user_by_email(&mut *self.txn, email).await
    }
}

async fn create_user<'e, E>(executor: E, input: CreateUser) -> Result<User, CreateUserError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, User>(
        r#"
          INSERT INTO "user"
          (id, name, email, email_verified, image, created_at, updated_at)
          VALUES
          ($1, $2, $3, $4, $5, now(), now())
          RETURNING id, name, email, email_verified, image, created_at, updated_at
          "#,
    )
    .bind(input.id)
    .bind(input.name)
    .bind(input.email)
    .bind(false)
    .bind(input.image)
    .fetch_one(executor)
    .await
    .map_err(|_| CreateUserError::Stub)
}

async fn get_user_by_id<'e, E>(executor: E, id: &str) -> Result<Option<User>, GetUserError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, name, email, email_verified, image, created_at, updated_at
        FROM "user"
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(|_| GetUserError::Stub)
}

async fn get_user_by_email<'e, E>(executor: E, email: &str) -> Result<Option<User>, GetUserError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, name, email, email_verified, image, created_at, updated_at
        FROM "user"
        WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(executor)
    .await
    .map_err(|_| GetUserError::Stub)
}
