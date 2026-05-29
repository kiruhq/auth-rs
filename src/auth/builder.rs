use super::AuthError;
use super::config::{AuthConfig, EmailAndPasswordConfig};
use crate::auth::Auth;

#[derive(Default)]
pub struct AuthBuilder {
    config: AuthConfig,
}

impl AuthBuilder {
    pub fn email_and_password<F>(mut self, f: F) -> Self
    where
        F: FnOnce(EmailAndPasswordBuilder) -> EmailAndPasswordBuilder,
    {
        let builder = EmailAndPasswordBuilder {
            config: self.config.email_and_password,
        };

        self.config.email_and_password = f(builder).config;
        self
    }

    pub fn build(self) -> Result<Auth, AuthError> {
        Ok(Auth::new(self.config))
    }
}

#[derive(Default)]
pub struct EmailAndPasswordBuilder {
    config: EmailAndPasswordConfig,
}

impl EmailAndPasswordBuilder {
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_password_builder() {
        let builder = Auth::builder().email_and_password(|config| config.enabled(true));
        let auth = builder.build().expect("should build fine");

        assert!(auth.config.email_and_password.enabled)
    }
}
