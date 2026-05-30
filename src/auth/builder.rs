use super::config::{AuthConfig, EmailAndPasswordConfig};
use crate::adapters::database::DatabaseAdapter;
use crate::auth::Auth;
use std::sync::Arc;

#[derive(Default)]
pub struct NoAdapter;

#[derive(Default)]
pub struct AuthBuilder<DB = NoAdapter> {
    config: AuthConfig,
    database: DB,
}

impl<DB> AuthBuilder<DB> {
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

impl<DB> AuthBuilder<DB>
where
    DB: DatabaseAdapter,
{
    pub fn build(self) -> Auth<DB> {
        Auth {
            config: self.config,
            database: Arc::new(self.database),
        }
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

    pub fn auto_sign_in(mut self, enabled: bool) -> Self {
        self.config.auto_sign_in = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_password_builder() {
        let defaults = Auth::builder();
        let config = defaults.config.email_and_password;

        assert!(!config.enabled, "email_and_password should be disabled");
        assert!(config.auto_sign_in, "auto sign in should be enabled");
    }

    #[test]
    fn test_email_password_builder_overrides_config() {
        let builder =
            Auth::builder().email_and_password(|config| config.enabled(true).auto_sign_in(false));
        let config = builder.config.email_and_password;

        assert!(config.enabled, "email_and_password should be enabled");
        assert!(!config.auto_sign_in, "auto sign in should be disabled");
    }
}
