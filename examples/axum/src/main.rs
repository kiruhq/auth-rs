use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    let pool = sqlx::PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost/test").await?

    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("server to bind to localhost 3000");

    axum::serve(listener, app)
        .await
        .expect("axum to serve the app");
}
