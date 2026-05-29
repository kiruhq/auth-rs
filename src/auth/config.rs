#[derive(Default)]
pub struct AuthConfig {
    pub email_and_password: EmailAndPasswordConfig,
}

pub struct EmailAndPasswordConfig {
    pub enabled: bool,
    pub auto_sign_in: bool,
}

impl Default for EmailAndPasswordConfig {
    fn default() -> Self {
        EmailAndPasswordConfig {
            enabled: false,
            auto_sign_in: true,
        }
    }
}
