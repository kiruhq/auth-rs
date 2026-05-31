use chrono::Duration;

pub struct SessionConfig {
    pub expires_in: Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            expires_in: Duration::days(7),
        }
    }
}
