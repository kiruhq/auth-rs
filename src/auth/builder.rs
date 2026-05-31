use super::config::{AuthConfig, EmailAndPasswordConfig, EmailVerificationConfig};
use crate::adapters::database::DatabaseAdapter;
use crate::auth::Auth;
use crate::auth::verification;
use std::sync::Arc;

#[derive(Default)]
pub struct NoAdapter;

#[derive(Default)]
pub struct AuthBuilder<DB = NoAdapter> {
    config: AuthConfig,
    database: DB,
}

impl<DB> AuthBuilder<DB> {
    pub fn config<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut AuthConfig),
    {
        f(&mut self.config);
        self
    }

    pub fn email_and_password<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut EmailAndPasswordConfig),
    {
        f(&mut self.config.email_and_password);
        self
    }

    pub fn email_verification<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut EmailVerificationConfig),
    {
        f(&mut self.config.email_verification);
        self
    }
}

impl AuthBuilder<NoAdapter> {
    pub fn database<DB>(self, database: DB) -> AuthBuilder<DB>
    where
        DB: DatabaseAdapter,
    {
        AuthBuilder {
            config: self.config,
            database,
        }
    }
}

#[derive(Debug)]
pub enum AuthBuilderError {
    MissingSendEmailVerification,
    InvalidBaseUrl(String),
}

impl<DB> AuthBuilder<DB>
where
    DB: DatabaseAdapter,
{
    pub fn build(self) -> Result<Auth<DB>, AuthBuilderError> {
        validate_build_config(&self.config)?;

        Ok(Auth {
            config: self.config,
            database: Arc::new(self.database),
        })
    }
}

fn validate_build_config(config: &AuthConfig) -> Result<(), AuthBuilderError> {
    let sends_verification_email = config.email_and_password.require_email_verification
        || config.email_verification.send_on_signup;

    if !sends_verification_email {
        return Ok(());
    }

    if config.email_verification.send_verification_email.is_none() {
        return Err(AuthBuilderError::MissingSendEmailVerification);
    }

    verification::validate_verification_url_config(&config.base_url, &config.base_path)
        .map_err(|error| AuthBuilderError::InvalidBaseUrl(error.to_string()))
}

#[cfg(test)]
mod tests {
    use crate::auth::config::{
        SendVerificationEmail, SendVerificationEmailError, VerificationEmailSender,
    };

    use super::*;

    struct NoopVerificationEmailSender;

    #[async_trait::async_trait]
    impl VerificationEmailSender for NoopVerificationEmailSender {
        async fn send_verification_email(
            &self,
            _input: SendVerificationEmail,
        ) -> Result<(), SendVerificationEmailError> {
            Ok(())
        }
    }

    #[test]
    fn test_email_password_builder() {
        let defaults = Auth::builder();
        let config = defaults.config.email_and_password;

        assert!(!config.enabled, "email_and_password should be disabled");
        assert!(config.auto_sign_in, "auto sign in should be enabled");
    }

    #[test]
    fn test_email_password_builder_overrides_config() {
        let builder = Auth::builder().email_and_password(|config| {
            config.enabled = true;
            config.auto_sign_in = false;
        });
        let config = builder.config.email_and_password;

        assert!(config.enabled, "email_and_password should be enabled");
        assert!(!config.auto_sign_in, "auto sign in should be disabled");
    }

    #[test]
    fn validates_missing_verification_email_sender() {
        let mut config = AuthConfig::default();
        config.email_and_password.require_email_verification = true;

        assert!(matches!(
            validate_build_config(&config),
            Err(AuthBuilderError::MissingSendEmailVerification)
        ));
    }

    #[test]
    fn validates_verification_email_base_url() {
        let mut config = AuthConfig::default();
        config.base_url = "not-a-url".to_string();
        config.email_verification.send_on_signup = true;
        config.email_verification.send_verification_email =
            Some(Arc::new(NoopVerificationEmailSender));

        assert!(matches!(
            validate_build_config(&config),
            Err(AuthBuilderError::InvalidBaseUrl(_))
        ));
    }

    #[test]
    fn accepts_valid_verification_email_url_config() {
        let mut config = AuthConfig::default();
        config.base_url = "https://example.com".to_string();
        config.base_path = "/api/auth".to_string();
        config.email_verification.send_on_signup = true;
        config.email_verification.send_verification_email =
            Some(Arc::new(NoopVerificationEmailSender));

        assert!(validate_build_config(&config).is_ok());
    }
}
