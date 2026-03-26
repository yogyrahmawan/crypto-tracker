use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("CoinAPI error: {0}")]
    ApiRequest(#[from] reqwest::Error),

    #[error("Rate limit exceeded")]
    RateLimit, 

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #error("Configuration error: {0}")]
    Config(String),
}