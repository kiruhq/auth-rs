use crate::types::data::User;

pub(crate) trait Store {
    fn create_user_with_credential_account(
        &self,
        input: CreateUserWithCredentialAccountResult,
    ) -> Result<CreateUserWithCredentialAccountResult, StoreError>;
}

pub(crate) enum StoreError {}

pub(crate) enum CreateUserWithCredentialAccountResult {
    Created { user: User },
    EmailAlreadyExists,
}
