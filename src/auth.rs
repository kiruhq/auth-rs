mod builder;
mod config;

use builder::AuthBuilder;
use config::AuthConfig;

#[derive(Default)]
pub struct Auth {
    pub(crate) config: AuthConfig,
}

#[derive(Debug)]
pub enum AuthError {}

impl Auth {
    pub fn new(config: AuthConfig) -> Self {
        Auth { config }
    }

    pub fn builder() -> AuthBuilder {
        AuthBuilder::default()
    }
}
