use std::fmt;
use url::{ParseError, Url};

pub(crate) fn create_verification_url(
    base_url: &str,
    base_path: &str,
    token: &str,
    callback_url: Option<&str>,
) -> Result<String, CreateVerificationUrlError> {
    let mut url = verification_url_base(base_url, base_path)?;

    url.query_pairs_mut().append_pair("token", token);

    if let Some(callback_url) = callback_url {
        url.query_pairs_mut()
            .append_pair("callbackURL", callback_url);
    }

    Ok(url.to_string())
}

pub(crate) fn validate_verification_url_config(
    base_url: &str,
    base_path: &str,
) -> Result<(), CreateVerificationUrlError> {
    verification_url_base(base_url, base_path).map(|_| ())
}

#[derive(Debug)]
pub(crate) enum CreateVerificationUrlError {
    InvalidBaseUrl(ParseError),
    UnsupportedBaseUrlScheme,
    MissingBaseUrlHost,
    BaseUrlCannotBeABase,
    BaseUrlIncludesQueryOrFragment,
    InvalidBasePath,
}

impl fmt::Display for CreateVerificationUrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBaseUrl(error) => write!(f, "invalid base_url: {error}"),
            Self::UnsupportedBaseUrlScheme => write!(f, "base_url must use http or https"),
            Self::MissingBaseUrlHost => write!(f, "base_url must include a host"),
            Self::BaseUrlCannotBeABase => write!(f, "base_url cannot be used as a base URL"),
            Self::BaseUrlIncludesQueryOrFragment => {
                write!(f, "base_url must not include a query string or fragment")
            }
            Self::InvalidBasePath => {
                write!(
                    f,
                    "base_path must be a relative path without a query string or fragment"
                )
            }
        }
    }
}

fn verification_url_base(
    base_url: &str,
    base_path: &str,
) -> Result<Url, CreateVerificationUrlError> {
    validate_base_path(base_path)?;

    let mut url = Url::parse(base_url).map_err(CreateVerificationUrlError::InvalidBaseUrl)?;

    if !matches!(url.scheme(), "http" | "https") {
        return Err(CreateVerificationUrlError::UnsupportedBaseUrlScheme);
    }

    if url.host_str().is_none() {
        return Err(CreateVerificationUrlError::MissingBaseUrlHost);
    }

    if url.cannot_be_a_base() {
        return Err(CreateVerificationUrlError::BaseUrlCannotBeABase);
    }

    if url.query().is_some() || url.fragment().is_some() {
        return Err(CreateVerificationUrlError::BaseUrlIncludesQueryOrFragment);
    }

    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| CreateVerificationUrlError::BaseUrlCannotBeABase)?;

        for segment in base_path
            .trim_matches('/')
            .split('/')
            .filter(|segment| !segment.is_empty())
        {
            segments.push(segment);
        }

        segments.push("verify-email");
    }

    Ok(url)
}

fn validate_base_path(base_path: &str) -> Result<(), CreateVerificationUrlError> {
    if base_path.trim() != base_path {
        return Err(CreateVerificationUrlError::InvalidBasePath);
    }

    if base_path.contains('?')
        || base_path.contains('#')
        || base_path.contains("://")
        || base_path.starts_with("//")
    {
        return Err(CreateVerificationUrlError::InvalidBasePath);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_verification_url_joins_base_path() {
        let url =
            create_verification_url("https://example.com/", "/api/auth/", "token", None).unwrap();

        assert_eq!(url, "https://example.com/api/auth/verify-email?token=token");
    }

    #[test]
    fn create_verification_url_includes_callback_url() {
        let url = create_verification_url(
            "https://example.com",
            "api/auth",
            "token",
            Some("https://app.example.com/welcome?plan=pro"),
        )
        .unwrap();

        assert_eq!(
            url,
            "https://example.com/api/auth/verify-email?token=token&callbackURL=https%3A%2F%2Fapp.example.com%2Fwelcome%3Fplan%3Dpro"
        );
    }

    #[test]
    fn rejects_invalid_base_url() {
        assert!(matches!(
            validate_verification_url_config("not-a-url", "/api/auth"),
            Err(CreateVerificationUrlError::InvalidBaseUrl(_))
        ));
    }

    #[test]
    fn rejects_base_path_with_query() {
        assert!(matches!(
            validate_verification_url_config("https://example.com", "/api/auth?x=1"),
            Err(CreateVerificationUrlError::InvalidBasePath)
        ));
    }
}
