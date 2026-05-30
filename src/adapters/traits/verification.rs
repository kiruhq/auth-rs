use crate::core::entity::Verification;

pub(crate) struct CreateVerification {
    email: String,
    password_hash: String,
}

enum CreateVerificationError {
    Stub,
}

enum GetVerificationError {
    Stub,
}

#[async_trait::async_trait]
pub(crate) trait VerificationStore: Send {
    async fn create_pending_signup(
        &mut self,
        params: CreateVerification,
    ) -> Result<Verification, CreateVerificationError>;

    async fn get_pending_signup_by_token_hash(
        &mut self,
        hash: &str,
    ) -> Result<Verification, GetVerificationError>;
}
