use std::sync::Arc;

use chrono::Duration;

pub struct EmailVerificationConfig {
    pub send_verification_email: Option<Arc<dyn VerificationEmailSender>>,
    pub send_on_signup: bool,
    pub expires_in: Duration,
}

impl Default for EmailVerificationConfig {
    fn default() -> Self {
        EmailVerificationConfig {
            send_verification_email: None,
            send_on_signup: false,
            expires_in: Duration::hours(24),
        }
    }
}

#[derive(Debug)]
pub struct SendVerificationEmailError;

pub struct SendVerificationEmail {
    pub user: VerificationEmailUser,
    pub url: String,
    pub token: String,
}

pub struct VerificationEmailUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub image: Option<String>,
}

#[async_trait::async_trait]
pub trait VerificationEmailSender: Send + Sync + 'static {
    async fn send_verification_email(
        &self,
        input: SendVerificationEmail,
    ) -> Result<(), SendVerificationEmailError>;
}
