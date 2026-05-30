use crate::core::entity::PendingSignup;

pub(crate) struct CreatePendingSignup {
    email: String,
    password_hash: String,
}

enum CreatePendingSignupError {
    Stub,
}

enum GetPendingSignupError {
    Stub,
}

#[async_trait::async_trait]
pub(crate) trait PendingSignupStore: Send {
    async fn create_pending_signup(
        &mut self,
        params: CreatePendingSignup,
    ) -> Result<PendingSignup, CreatePendingSignupError>;

    async fn get_pending_signup_by_id(
        &mut self,
        id: &str,
    ) -> Result<PendingSignup, GetPendingSignupError>;
}
