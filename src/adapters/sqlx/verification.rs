use super::{SqlxPostgresAdapter, SqlxPostgresTxnAdapter};
use crate::adapters::traits::verification::{
    CreateVerification, CreateVerificationError, GetVerificationError, VerificationStore,
};
use crate::core::entity::Verification;

#[async_trait::async_trait]
impl VerificationStore for SqlxPostgresAdapter {
    async fn create_verification(
        &mut self,
        params: CreateVerification,
    ) -> Result<Verification, CreateVerificationError> {
        create_verification(&self.conn, params).await
    }

    async fn get_verification_by_token_hash(
        &mut self,
        hash: &str,
    ) -> Result<Option<Verification>, GetVerificationError> {
        get_verification_by_token_hash(&self.conn, hash).await
    }
}

#[async_trait::async_trait]
impl<'a> VerificationStore for SqlxPostgresTxnAdapter<'a> {
    async fn create_verification(
        &mut self,
        params: CreateVerification,
    ) -> Result<Verification, CreateVerificationError> {
        create_verification(&mut *self.txn, params).await
    }

    async fn get_verification_by_token_hash(
        &mut self,
        hash: &str,
    ) -> Result<Option<Verification>, GetVerificationError> {
        get_verification_by_token_hash(&mut *self.txn, hash).await
    }
}

async fn create_verification<'e, E>(
    executor: E,
    input: CreateVerification,
) -> Result<Verification, CreateVerificationError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Verification>(
        r#"
        INSERT INTO verification
        (id, kind, identifier, token_hash, expires_at, created_at, updated_at)
        VALUES
        ($1, $2, $3, $4, $5, now(), now())
        RETURNING id, kind, identifier, token_hash, expires_at, created_at, updated_at
        "#,
    )
    .bind(input.id)
    .bind(input.kind)
    .bind(input.identifier)
    .bind(input.token_hash)
    .bind(input.expires_at)
    .fetch_one(executor)
    .await
    .map_err(|_| CreateVerificationError::Stub)
}

async fn get_verification_by_token_hash<'e, E>(
    executor: E,
    hash: &str,
) -> Result<Option<Verification>, GetVerificationError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Verification>(
        r#"
        SELECT id, kind, identifier, token_hash, expires_at, created_at, updated_at
        FROM verification
        WHERE token_hash = $1
        "#,
    )
    .bind(hash)
    .fetch_optional(executor)
    .await
    .map_err(|_| GetVerificationError::Stub)
}
