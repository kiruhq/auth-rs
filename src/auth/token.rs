use argon2::password_hash::rand_core::{OsRng, RngCore};
use base64::prelude::*;
use sha2::{Digest, Sha256};

pub(crate) struct SecretToken {
    pub(crate) token: String,
    pub(crate) token_hash: String,
}

pub(crate) fn generate_secret_token() -> SecretToken {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);

    let token = BASE64_URL_SAFE_NO_PAD.encode(bytes);
    let token_hash = hex::encode(Sha256::digest(bytes));

    SecretToken { token, token_hash }
}

pub(crate) enum TokenValidationError {
    InvalidEncoding,
}

pub(crate) fn hash_secret_token(token: &str) -> Result<String, TokenValidationError> {
    let bytes = BASE64_URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|_| TokenValidationError::InvalidEncoding)?;

    Ok(hex::encode(Sha256::digest(bytes)))
}
