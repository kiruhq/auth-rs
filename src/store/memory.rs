use crate::store::{
    CreateUserWithCredentialAccountInput, CreateUserWithCredentialAccountResult, Store, StoreError,
};
use crate::types::data::User;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::RwLock;

pub(crate) struct MemoryStore {
    users: RwLock<HashMap<String, User>>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        MemoryStore {
            users: RwLock::new(HashMap::new()),
        }
    }
}

impl Store for MemoryStore {
    async fn create_user_with_credential_account(
        &self,
        input: CreateUserWithCredentialAccountInput,
    ) -> Result<CreateUserWithCredentialAccountResult, StoreError> {
        let lock = self.users.write().expect("memory store mutex poisoned");

        let exists: bool = lock.contains_key(&input.email);
        if exists {
            return Ok(CreateUserWithCredentialAccountResult::EmailAlreadyExists);
        }

        let now = Utc::now();

        let user = User::new(
            input.user_id,
            input.name,
            input.email,
            false,
            None,
            now,
            now,
        );

        Ok(CreateUserWithCredentialAccountResult::Created { user })
    }
}
