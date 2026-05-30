use crate::core::entity::Account;

pub(crate) struct CreateAccount {
    pub id: String,
    pub account_id: String,
    pub user_id: String,
    pub provider_id: String,
    pub password: Option<String>,
}

pub(crate) enum CreateAccountError {
    Stub,
}

pub(crate) enum GetAccountError {
    Stub,
}

#[async_trait::async_trait]
pub(crate) trait AccountStore: Send {
    async fn create_account(
        &mut self,
        account: CreateAccount,
    ) -> Result<Account, CreateAccountError>;

    async fn get_account(
        &mut self,
        provider: &str,
        provider_account_id: &str,
    ) -> Result<Option<Account>, GetAccountError>;
}
