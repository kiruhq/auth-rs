use auth_rs::Auth;
use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:55432/auth_rs_axum".to_owned());

    println!("connecting to db");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let auth = Auth::builder()
        .config(|config| {
            config.base_url = "http://localhost:3000".to_owned();
        })
        .email_and_password(|cfg| cfg.enabled = true)
        .sqlx(pool)
        .build()?;

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .nest("/api/auth", auth_rs::axum::router(auth));

    println!("starting app...");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("server to bind to localhost 3000");

    axum::serve(listener, app)
        .await
        .expect("axum to serve the app");

    Ok(())
}
