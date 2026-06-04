mod handlers;
mod types;

use crate::Auth;
use crate::adapters::database::DatabaseAdapter;
use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;

struct AuthState<DB> {
    inner: Arc<Auth<DB>>,
}

impl<DB> AuthState<DB> {
    pub(crate) fn auth(&self) -> &Auth<DB> {
        &self.inner
    }
}

impl<DB> Clone for AuthState<DB> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

pub fn router<DB>(auth: Auth<DB>) -> Router
where
    DB: DatabaseAdapter,
{
    let mut router: Router<AuthState<DB>> = Router::new();

    if auth.config.email_and_password.enabled {
        router = setup_email_password_routes(router);
    }

    router.with_state(AuthState {
        inner: Arc::new(auth),
    })
}

fn setup_email_password_routes<DB>(router: Router<AuthState<DB>>) -> Router<AuthState<DB>>
where
    DB: DatabaseAdapter,
{
    router
        .route("/sign-up/email", post(handlers::signup_email::signup))
        .route("/sign-in/email", post(handlers::signin_email::signin))
        .route("/session", get(handlers::session::session))
}
