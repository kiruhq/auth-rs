pub(crate) mod memory;

use crate::types::data::User;
use memory::MemoryStore;
use std::error::Error;

#[cfg(feature = "sqlx")]
pub mod sqlx;

pub(crate) enum StoreKind {
    #[cfg(feature = "sqlx")]
    Postgres(sqlx::SqlxStore<sqlx::Postgres>),
    Memory(MemoryStore),
}

impl Store for StoreKind {
    async fn create_user_with_credential_account(
        &self,
        input: CreateUserWithCredentialAccountInput,
    ) -> Result<CreateUserWithCredentialAccountResult, StoreError> {
        match self {
            StoreKind::Memory(store) => store.create_user_with_credential_account(input).await,
            StoreKind::Postgres(store) => store.create_user_with_credential_account(input).await,
        }
    }
}

pub(crate) trait Store: Send + Sync + 'static {
    async fn create_user_with_credential_account(
        &self,
        input: CreateUserWithCredentialAccountInput,
    ) -> Result<CreateUserWithCredentialAccountResult, StoreError>;
}

pub enum StoreError {
    DriverError(Box<dyn Error>),
}

pub struct CreateUserWithCredentialAccountInput {
    pub user_id: String,
    pub account_id: String,
    pub name: String,
    pub email: String,
    pub hashed_password: String,
}

pub enum CreateUserWithCredentialAccountResult {
    Created { user: User },
    EmailAlreadyExists,
}
