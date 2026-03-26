use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub coinapi_key: String, 
    pub host: String,
    pub port: u16,
    pub update_interval_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let coinapi_key = std::env::var("COINAPI_KEY")
            .context("Missing COINAPI_KEY environment variable")?;
        let host = std::env::var("HOST").unwrap_or_else(|_| "localhost".into());
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".into())
            .parse()
            .context("Invalid PORT environment variable")?;
        let update_interval_secs = std::env::var("UPDATE_INTERVAL_SECS")
            .unwrap_or_else(|_| "60".into())
            .parse()
            .context("Invalid UPDATE_INTERVAL_SECS environment variable")?;

        Ok(Config {
            coinapi_key,
            host,
            port,
            update_interval_secs,
        })
    }

}