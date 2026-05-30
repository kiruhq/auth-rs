mod advanced;
mod id_generator;

use std::sync::Arc;

use argon2::PasswordHasher as Argon2PasswordHasherTrait;
use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordVerifier};

use advanced::AdvancedConfig;

#[derive(Default)]
pub struct AuthConfig {
    pub email_and_password: EmailAndPasswordConfig,
    pub advanced: AdvancedConfig,
}

pub struct EmailAndPasswordConfig {
    pub enabled: bool,
    pub disable_sign_up: bool,
    pub auto_sign_in: bool,
    pub min_password_length: u32,
    pub max_password_length: u32,
    pub password_hasher: Arc<dyn PasswordHasher>,
}

impl Default for EmailAndPasswordConfig {
    fn default() -> Self {
        EmailAndPasswordConfig {
            enabled: false,
            disable_sign_up: false,
            auto_sign_in: true,
            min_password_length: 8,
            max_password_length: 128,
            password_hasher: Arc::new(Argon2PasswordHasher {}),
        }
    }
}

pub enum PasswordHasherError {
    Argon2(argon2::password_hash::Error),
}

pub trait PasswordHasher: Send + Sync + 'static {
    fn hash(&self, password: &str) -> Result<String, PasswordHasherError>;
    fn verify(&self, password: &str, hash: &str) -> Result<bool, PasswordHasherError>;
}

pub struct Argon2PasswordHasher {}

impl PasswordHasher for Argon2PasswordHasher {
    fn hash(&self, password: &str) -> Result<String, PasswordHasherError> {
        let salt = SaltString::generate(&mut OsRng);
        let result = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(PasswordHasherError::Argon2)?
            .to_string();
        Ok(result)
    }

    fn verify(&self, password: &str, hash: &str) -> Result<bool, PasswordHasherError> {
        let parsed_hash = PasswordHash::new(hash).map_err(PasswordHasherError::Argon2)?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(PasswordHasherError::Argon2)?;

        Ok(true)
    }
}

pub enum ModelName<'a> {
    User,
    Account,
    Session,
    Verification,
    Custom(&'a str),
}

impl<'a> ToString for ModelName<'a> {
    fn to_string(&self) -> String {
        match self {
            Self::User => "user",
            Self::Account => "account",
            Self::Session => "session",
            Self::Verification => "verification",
            Self::Custom(x) => x,
        }
        .to_string()
    }
}
