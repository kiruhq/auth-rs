use std::sync::Arc;

use super::id_generator::{IdGenerator, KsuidGenerator};

#[derive(Default)]
pub struct AdvancedConfig {
    pub database: DatabaseConfig,
}

pub struct DatabaseConfig {
    pub id_generator: Arc<dyn IdGenerator>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            id_generator: Arc::new(KsuidGenerator),
        }
    }
}
