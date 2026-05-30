use crate::core::entity::Verification;
use chrono::{DateTime, Utc};

pub(crate) struct CreateVerification {
    pub id: String,
    pub kind: String,
    pub identifier: String,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

pub(crate) enum CreateVerificationError {
    Stub,
}

pub(crate) enum GetVerificationError {
    Stub,
}

#[async_trait::async_trait]
pub(crate) trait VerificationStore: Send {
    async fn create_verification(
        &mut self,
        params: CreateVerification,
    ) -> Result<Verification, CreateVerificationError>;

    async fn get_verification_by_token_hash(
        &mut self,
        hash: &str,
    ) -> Result<Option<Verification>, GetVerificationError>;
}
