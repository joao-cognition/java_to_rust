use rust_travel::{app, config::Config, db};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .init();

    let config = Config::from_env()?;
    tracing::info!(
        "connecting to {} as {} (pool max {})",
        config.database_url,
        config.user,
        db::MAX_CONNECTIONS
    );
    let pool = db::connect(&config).await?;

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port)).await?;
    tracing::info!("rust-travel listening on :{}/travel/api", config.port);
    axum::serve(listener, app(pool))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
