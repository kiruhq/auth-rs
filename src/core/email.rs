use email_address::{EmailAddress, Options};

pub(crate) enum EmailValidationError {
    InvalidEmail,
}

pub(crate) fn normalize_email(email: &str) -> Result<String, EmailValidationError> {
    let email = email.trim().to_lowercase();

    let options = Options::default()
        .without_display_text()
        .without_domain_literal();

    EmailAddress::parse_with_options(&email, options)
        .map_err(|_| EmailValidationError::InvalidEmail)?;

    Ok(email)
}
