use chrono::{DateTime, Utc};

use crate::core::entity::Session;

#[async_trait::async_trait]
pub(crate) trait SessionStore: Send + Sync {
    async fn get_session_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<Session>, GetSessionError>;

    async fn delete_session_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<Session>, DeleteSessionError>;
}

#[async_trait::async_trait]
pub(crate) trait SessionTransactionStore: Send {
    async fn create_session(
        &mut self,
        session: CreateSession,
    ) -> Result<Session, CreateSessionError>;

    async fn get_session_by_token_hash(
        &mut self,
        token_hash: &str,
    ) -> Result<Option<Session>, GetSessionError>;
}

pub(crate) struct CreateSession {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

pub(crate) enum CreateSessionError {
    Stub,
}

pub(crate) enum GetSessionError {
    Stub,
}

pub(crate) enum DeleteSessionError {
    Stub,
}
