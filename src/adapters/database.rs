use super::traits::{
    account::AccountStore, pending::PendingSignupStore, user::UserStore,
    verification::VerificationStore,
};

pub enum AdapterError {
    SqlxError(),
}

#[async_trait::async_trait]
pub trait DatabaseAdapter:
    UserStore + AccountStore + PendingSignupStore + VerificationStore + Send + Sync + 'static
{
    type Transaction<'a>: DatabaseTransaction + 'a
    where
        Self: 'a;

    async fn begin_txn(&self) -> Result<Self::Transaction<'_>, AdapterError>;
}

#[async_trait::async_trait]
pub trait DatabaseTransaction:
    UserStore + AccountStore + PendingSignupStore + VerificationStore + Send + Sync
{
    async fn commit(self) -> Result<(), AdapterError>;
    async fn rollback(self) -> Result<(), AdapterError>;
}
