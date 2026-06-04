use super::{SqlxPostgresAdapter, SqlxPostgresTxnAdapter};
use crate::adapters::traits::session::{
    CreateSession, CreateSessionError, GetSessionError, SessionStore, SessionTransactionStore,
};
use crate::core::entity::Session;

#[async_trait::async_trait]
impl SessionStore for SqlxPostgresAdapter {
    async fn get_session_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<Session>, GetSessionError> {
        get_session_by_token_hash(&self.conn, token_hash).await
    }
}

#[async_trait::async_trait]
impl<'a> SessionTransactionStore for SqlxPostgresTxnAdapter<'a> {
    async fn create_session(
        &mut self,
        session: CreateSession,
    ) -> Result<Session, CreateSessionError> {
        create_session(&mut *self.txn, session).await
    }

    async fn get_session_by_token_hash(
        &mut self,
        token_hash: &str,
    ) -> Result<Option<Session>, GetSessionError> {
        get_session_by_token_hash(&mut *self.txn, token_hash).await
    }
}

async fn create_session<'e, E>(
    executor: E,
    input: CreateSession,
) -> Result<Session, CreateSessionError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Session>(
        r#"
        INSERT INTO "session"
        (id, user_id, token, expires_at, ip_address, user_agent, created_at, updated_at)
        VALUES
        ($1, $2, $3, $4, $5, $6, now(), now())
        RETURNING id, user_id, token AS token_hash, expires_at, ip_address, user_agent, created_at, updated_at
        "#,
    )
    .bind(input.id)
    .bind(input.user_id)
    .bind(input.token)
    .bind(input.expires_at)
    .bind(input.ip_address)
    .bind(input.user_agent)
    .fetch_one(executor)
    .await
    .map_err(|_| CreateSessionError::Stub)
}

async fn get_session_by_token_hash<'e, E>(
    executor: E,
    token_hash: &str,
) -> Result<Option<Session>, GetSessionError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Session>(
        r#"
        SELECT id, user_id, token AS token_hash, expires_at, ip_address, user_agent, created_at, updated_at
        FROM "session"
        WHERE token = $1
        "#,
    )
    .bind(token_hash)
    .fetch_optional(executor)
    .await
    .map_err(|_| GetSessionError::Stub)
}
