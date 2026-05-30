use super::traits::{CreateAccount, CreateAccountError, CreateUser, CreateUserError};
use crate::adapters::database::{AdapterError, DatabaseAdapter, DatabaseTransaction};
use crate::adapters::traits::{AccountStore, GetAccountError, GetUserError, UserStore};
use crate::core::entity::{Account, User};

pub struct SqlxPostgresAdapter {
    conn: sqlx::PgPool,
}

#[async_trait::async_trait]
impl DatabaseAdapter for SqlxPostgresAdapter {
    type Transaction<'a> = SqlxPostgresTxnAdapter<'a>;

    async fn begin_txn(&self) -> Result<Self::Transaction<'_>, AdapterError> {
        let txn = self
            .conn
            .begin()
            .await
            .map_err(|_| AdapterError::SqlxError())?;

        Ok(SqlxPostgresTxnAdapter { txn })
    }
}

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
impl AccountStore for SqlxPostgresAdapter {
    async fn create_account(
        &mut self,
        account: CreateAccount,
    ) -> Result<Account, CreateAccountError> {
        create_account(&self.conn, account).await
    }

    async fn get_account(
        &mut self,
        provider: &str,
        provider_account_id: &str,
    ) -> Result<Option<Account>, GetAccountError> {
        get_account(&self.conn, provider, provider_account_id).await
    }
}

pub struct SqlxPostgresTxnAdapter<'a> {
    txn: sqlx::PgTransaction<'a>,
}

#[async_trait::async_trait]
impl<'a> DatabaseTransaction for SqlxPostgresTxnAdapter<'a> {
    async fn commit(self) -> Result<(), AdapterError> {
        self.txn
            .commit()
            .await
            .map_err(|_e| AdapterError::SqlxError())
    }

    async fn rollback(self) -> Result<(), AdapterError> {
        self.txn
            .rollback()
            .await
            .map_err(|_e| AdapterError::SqlxError())
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

#[async_trait::async_trait]
impl<'a> AccountStore for SqlxPostgresTxnAdapter<'a> {
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

async fn get_user_by_id<'e, E>(_executor: E, _id: &str) -> Result<Option<User>, GetUserError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    unimplemented!()
}

async fn get_user_by_email<'e, E>(_executor: E, _email: &str) -> Result<Option<User>, GetUserError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    unimplemented!()
}

async fn create_account<'e, E>(
    _executor: E,
    _input: CreateAccount,
) -> Result<Account, CreateAccountError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    unimplemented!()
}

async fn get_account<'e, E>(
    _executor: E,
    _provider: &str,
    _provider_account_id: &str,
) -> Result<Option<Account>, GetAccountError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    unimplemented!()
}
