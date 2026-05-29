#[derive(Default)]
pub struct AuthConfig {
    pub email_and_password: EmailAndPasswordConfig,
}

#[derive(Default)]
pub struct EmailAndPasswordConfig {
    pub enabled: bool,
}
