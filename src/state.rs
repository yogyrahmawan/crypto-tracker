use crate::{config::Config, models::PriceSnapshot};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},    
};
use tokio::sync::broadcast;

pub type PriceTx = broadcast::Sender<String>;

#[derive(Clone)]
pub struct AppState {
    pub prices: Arc<RwLock<HashMap<String, PriceSnapshot>>>,
    pub config: Config,
    pub price_tx: PriceTx,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let (price_tx, _) = broadcast::channel(256);
        AppState {
            prices: Arc::new(RwLock::new(HashMap::new())),
            config,
            price_tx,
        }
    }

    pub fn get_prices(&self) -> HashMap<String, PriceSnapshot> {
        self.prices.read().unwrap().clone()
    }

    pub fn update_price(&self, snapshot: PriceSnapshot) {
        {
            let mut map = self.prices.write().unwrap();
            map.insert(snapshot.symbol.clone(), snapshot.clone());
        }

        let msg = crate::models::WsMessage::PriceUpdate { data: snapshot }.to_json();
        let _ = self.price_tx.send(msg);
    }
}