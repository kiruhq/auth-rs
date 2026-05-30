mod builder;
pub(crate) mod config;

use std::sync::Arc;

use crate::auth::config::ModelName;

use builder::AuthBuilder;
use config::AuthConfig;

pub struct Auth<DB> {
    pub(crate) config: AuthConfig,
    pub(crate) database: Arc<DB>,
}

#[derive(Debug)]
pub enum AuthError {
    MissingStore,
}

impl Auth<()> {
    pub fn builder() -> AuthBuilder {
        AuthBuilder::default()
    }
}

impl<DB> Auth<DB> {
    pub(crate) fn generate_id(&self, model_name: ModelName) -> String {
        self.config
            .advanced
            .database
            .id_generator
            .generate(model_name)
    }
}
