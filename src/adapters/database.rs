use super::traits::{
    account::{AccountStore, AccountTransactionStore},
    pending::{PendingSignupStore, PendingSignupTransactionStore},
    session::{SessionStore, SessionTransactionStore},
    user::{UserStore, UserTransactionStore},
    verification::{VerificationStore, VerificationTransactionStore},
};

pub enum AdapterError {
    SqlxError(),
}

#[async_trait::async_trait]
pub trait DatabaseAdapter:
    UserStore
    + AccountStore
    + PendingSignupStore
    + VerificationStore
    + SessionStore
    + Send
    + Sync
    + 'static
{
    type Transaction<'a>: DatabaseTransaction + 'a
    where
        Self: 'a;

    async fn begin_txn(&self) -> Result<Self::Transaction<'_>, AdapterError>;
}

#[async_trait::async_trait]
pub trait DatabaseTransaction:
    UserTransactionStore
    + AccountTransactionStore
    + PendingSignupTransactionStore
    + VerificationTransactionStore
    + SessionTransactionStore
    + Send
    + Sync
{
    async fn commit(self) -> Result<(), AdapterError>;
    async fn rollback(self) -> Result<(), AdapterError>;
}
