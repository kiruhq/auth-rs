mod builder;
pub(crate) mod config;
pub(crate) mod token;
pub(crate) mod verification;

use std::sync::Arc;

use crate::auth::config::ModelName;

pub use builder::AuthBuilder;
use config::AuthConfig;

pub struct Auth<DB> {
    pub(crate) config: AuthConfig,
    pub(crate) database: Arc<DB>,
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
