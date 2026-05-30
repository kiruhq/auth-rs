mod account;
mod pending;
mod user;
mod verification;

use crate::adapters::database::{AdapterError, DatabaseAdapter, DatabaseTransaction};

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
