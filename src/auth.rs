#[derive(Default)]
pub struct Auth {
    email_password: EmailPasswordConfig,
}

#[derive(Default)]
struct EmailPasswordConfig {
    enabled: bool,
}

impl Auth {
    pub fn new() -> Self {
        Auth {
            email_password: EmailPasswordConfig::default(),
        }
    }

    pub fn email_and_password(mut self, enabled: bool) -> Self {
        self.email_password.enabled = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::Auth;

    #[test]
    fn test_email_password() {
        let auth = Auth::new();
        assert!(!auth.email_password.enabled);

        let auth = auth.email_and_password(true);
        assert!(auth.email_password.enabled);
    }
}
