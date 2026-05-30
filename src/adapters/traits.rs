use crate::core::entity::{Account, User};

pub(crate) enum CreateUserError {
    Stub,
}

pub(crate) enum GetUserError {
    Stub,
}

pub(crate) struct CreateUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub image: Option<String>,
}

#[async_trait::async_trait]
pub(crate) trait UserStore: Send {
    async fn create_user(&mut self, user: CreateUser) -> Result<User, CreateUserError>;
    async fn get_user_by_id(&mut self, id: &str) -> Result<Option<User>, GetUserError>;
    async fn get_user_by_email(&mut self, email: &str) -> Result<Option<User>, GetUserError>;
}

pub(crate) struct CreateAccount {
    pub id: String,
    pub account_id: String,
    pub user_id: String,
    pub provider_id: String,
    pub password: Option<String>,
}

pub(crate) enum CreateAccountError {}

pub(crate) enum GetAccountError {}

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
