use crate::core::entity::PendingSignup;

pub struct CreatePendingSignup {
    pub id: String,
    pub user_id: String,
    pub account_id: String,
    pub email: String,
    pub name: String,
    pub image: Option<String>,
    pub password_hash: String,
}

pub enum CreatePendingSignupError {
    Stub,
}

pub enum GetPendingSignupError {
    Stub,
}

#[async_trait::async_trait]
pub(crate) trait PendingSignupStore: Send + Sync {
    async fn get_pending_signup_by_id(
        &self,
        id: &str,
    ) -> Result<Option<PendingSignup>, GetPendingSignupError>;
}

#[async_trait::async_trait]
pub(crate) trait PendingSignupTransactionStore: Send {
    async fn create_pending_signup(
        &mut self,
        params: CreatePendingSignup,
    ) -> Result<PendingSignup, CreatePendingSignupError>;

    async fn get_pending_signup_by_id(
        &mut self,
        id: &str,
    ) -> Result<Option<PendingSignup>, GetPendingSignupError>;
}
