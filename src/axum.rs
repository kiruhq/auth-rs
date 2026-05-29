mod handlers;
mod types;

use crate::Auth;
use axum::Router;
use axum::routing::post;
use std::sync::Arc;

#[derive(Clone)]
struct AuthState {
    inner: Arc<Auth>,
}

impl AuthState {
    pub(crate) fn auth(&self) -> &Auth {
        &self.inner
    }
}

pub fn router(auth: Auth) -> Router {
    let mut router: Router<AuthState> = Router::new();

    if auth.config.email_and_password.enabled {
        router = setup_email_password_routes(router);
    }

    router.with_state(AuthState {
        inner: Arc::new(auth),
    })
}

fn setup_email_password_routes(router: Router<AuthState>) -> Router<AuthState> {
    router
        .route("/sign-up/email", post(handlers::email::signup))
        .route("/sign-in/email", post(handlers::email::signin))
}
