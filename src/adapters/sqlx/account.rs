use crate::adapters::sqlx::{SqlxPostgresAdapter, SqlxPostgresTxnAdapter};
use crate::adapters::traits::account::{
    AccountStore, AccountTransactionStore, CreateAccount, CreateAccountError, GetAccountError,
};
use crate::core::entity::Account;

#[async_trait::async_trait]
impl AccountStore for SqlxPostgresAdapter {
    async fn get_account(
        &self,
        provider: &str,
        provider_account_id: &str,
    ) -> Result<Option<Account>, GetAccountError> {
        get_account(&self.conn, provider, provider_account_id).await
    }
}

#[async_trait::async_trait]
impl<'a> AccountTransactionStore for SqlxPostgresTxnAdapter<'a> {
    async fn create_account(
        &mut self,
        account: CreateAccount,
    ) -> Result<Account, CreateAccountError> {
        create_account(&mut *self.txn, account).await
    }

    async fn get_account(
        &mut self,
        provider: &str,
        provider_account_id: &str,
    ) -> Result<Option<Account>, GetAccountError> {
        get_account(&mut *self.txn, provider, provider_account_id).await
    }
}

async fn create_account<'e, E>(
    executor: E,
    input: CreateAccount,
) -> Result<Account, CreateAccountError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Account>(
        r#"
        INSERT INTO account
        (id, account_id, user_id, provider_id, password, created_at, updated_at)
        VALUES
        ($1, $2, $3, $4, $5, now(), now())
        RETURNING id, account_id, user_id, provider_id, password, created_at, updated_at
        "#,
    )
    .bind(input.id)
    .bind(input.account_id)
    .bind(input.user_id)
    .bind(input.provider_id)
    .bind(input.password)
    .fetch_one(executor)
    .await
    .map_err(|_| CreateAccountError::Stub)
}

async fn get_account<'e, E>(
    executor: E,
    provider: &str,
    provider_account_id: &str,
) -> Result<Option<Account>, GetAccountError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, Account>(
        r#"
        SELECT id, account_id, user_id, provider_id, password, created_at, updated_at
        FROM account
        WHERE provider_id = $1 AND account_id = $2
        "#,
    )
    .bind(provider)
    .bind(provider_account_id)
    .fetch_optional(executor)
    .await
    .map_err(|_| GetAccountError::Stub)
}
