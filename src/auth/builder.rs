use super::AuthError;
use super::config::{AuthConfig, EmailAndPasswordConfig};
use crate::auth::Auth;
use crate::store::StoreKind;
use crate::store::memory::MemoryStore;

#[derive(Default)]
pub struct AuthBuilder {
    config: AuthConfig,
    store: Option<StoreKind>,
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

    pub fn memory(mut self) -> Self {
        self.store = Some(StoreKind::Memory(MemoryStore::default()));
        self
    }

    pub fn build(self) -> Result<Auth, AuthError> {
        let store = self.store.ok_or(AuthError::MissingStore)?;
        Ok(Auth {
            config: self.config,
            store,
        })
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
        let defaults = Auth::builder().memory().build().expect("should build fine");
        let config = defaults.config.email_and_password;

        assert!(config.enabled, "email_and_password should be enabled");
        assert!(!config.auto_sign_in, "auto sign in should be disabled");

        let builder = Auth::builder()
            .email_and_password(|config| config.enabled(true).auto_sign_in(false))
            .memory();

        let auth = builder.build().expect("should build fine");

        let config = auth.config.email_and_password;

        assert!(config.enabled, "email_and_password should be enabled");
        assert!(!config.auto_sign_in, "auto sign in should be disabled")
    }
}
