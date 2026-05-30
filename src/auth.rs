mod builder;
pub(crate) mod config;

use crate::store::StoreKind;

use builder::AuthBuilder;
use config::AuthConfig;

pub struct Auth {
    pub(crate) config: AuthConfig,
    pub(crate) store: StoreKind,
}

#[derive(Debug)]
pub enum AuthError {
    MissingStore,
}

impl Auth {
    pub fn builder() -> AuthBuilder {
        AuthBuilder::default()
    }
}
