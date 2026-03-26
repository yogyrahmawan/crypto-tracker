mod api;
mod config;
mod error;
mod models;
mod server;
mod state;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok(); // Load .env file if present
    let config = config::Config::from_env()?;

    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("crypto_tracker=info".parse()?)
        )
        .init();

    let cfg = config::Config::from_env()?;
    info!("Starting server on {}:{}", config.host, config.port);
   
    let state = state::AppState::new(config.clone());

    let fetcher_state = state.clone();
    tokio::spawn(async move {
        api::price_fetcher::run(fetcher_state).await;
    });

    info!("fetcher running... press control+c to stop");
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}
