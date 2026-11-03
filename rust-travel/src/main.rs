use rust_travel::{app, AppState};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://travel:travel@localhost:5432/travel".into());
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(9977);
    let max: u32 = std::env::var("DB_POOL_SIZE").ok().and_then(|p| p.parse().ok()).unwrap_or(10);
    let pool = PgPoolOptions::new()
        .max_connections(max)
        .connect(&url)
        .await
        .expect("connect to PostgreSQL");
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.expect("bind");
    tracing::info!("rust-travel listening on http://localhost:{port}/travel/api");
    axum::serve(listener, app(AppState { pool }))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .expect("server");
}
