use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Coin API Response
#[derive(Debug, Deserialize)]
pub struct CoinApiRate {
    pub asset_id_base: String,
    pub asset_id_quote: String,
    pub rate: f64,
    pub time: Option<String>,
}

// internal price record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSnapshot {
    pub symbol: String,
    pub price_usd: f64,
    pub updated_at: String,
}

// websocket message types 
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    Snapshot {
        prices: HashMap<String, PriceSnapshot>,
    },
    PriceUpdate {
        data: PriceSnapshot,
    },
    Error {
        message: String,
    },
}

impl WsMessage {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|e| format!("{{\"type\":\"error\",\"message\":\"{e}\"}}"))
    }
}

