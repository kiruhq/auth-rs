use super::traits::{
    account::AccountStore, pending::PendingSignupStore, session::SessionStore, user::UserStore,
    verification::VerificationStore,
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
    UserStore + AccountStore + PendingSignupStore + VerificationStore + SessionStore + Send + Sync
{
    async fn commit(self) -> Result<(), AdapterError>;
    async fn rollback(self) -> Result<(), AdapterError>;
}
