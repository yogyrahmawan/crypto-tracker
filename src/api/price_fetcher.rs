use crate::{error::AppError, models::PriceSnapshot, state::AppState};
use tracing::{error, info, warn};

const SYMBOLS: &[&str] = &["BTC", "ETH", "BNB", "SOL", "XRP",
    "ADA", "DOGE", "AVAX", "DOT", "MATIC"];

const COINAPI_URL: &str = "https://rest.coinapi.io/v1/exchangerate";

pub async fn run(state:AppState) {
    let interval = state.config.update_interval_secs; 

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("Failed to build HTTP client");
    
    info!("Price fetcher started - polling every {}s", interval);

    loop {
        fetch_all(&client, &state).await;
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
    }

    async fn fetch_all(client: &reqwest::Client, state: &AppState) {
        for &symbol in SYMBOLS {
            match fetch_price(client, state, symbol).await {
                Ok(snapshot) => {
                    info!("{}: ${:.2}", snapshot.symbol, snapshot.price_usd);
                    state.update_price(snapshot);
                }
                Err(AppError::RateLimit) => {
                    warn!("Rate limit exceeded. Backing off for 60 seconds...");
                    tokio::time::sleep(
                        tokio::time::Duration::from_secs(60)
                    ).await;
                    break;
                }
                Err(e) => {
                    error!("Error fetching price for {}: {}", symbol, e);
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await; // small delay between requests
        }
    }
}

async fn fetch_price(
    client: &reqwest::Client,
    state: &AppState,
    symbol: &str,
) -> Result<PriceSnapshot, AppError> {
    let url = format!("{}/{}{}", COINAPI_URL, symbol, "/USD");
    let resp = client
        .get(&url)
        .header("X-CoinAPI-Key", &std::env::var("COINAPI_KEY").unwrap_or_default())
        .send()
        .await?;

    if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(AppError::RateLimit);
    }

    let rate: crate::models::CoinApiRate = resp.error_for_status()?.json().await?;
 
    
    Ok(PriceSnapshot {
        symbol: symbol.to_string(),
        price_usd: rate.rate,
        updated_at: rate.time.unwrap_or_else(|| {
            chrono::Utc::now().to_rfc3339()
        }),
    })
}