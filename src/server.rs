use crate::{config::Config, models::WsMessage, state::AppState};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tracing::{error, info, warn};
use warp::{ws::Websocket, Filter};

pub async fn run(state: AppState, cfg: Config) { 
    let state_filter = {
        let s = state.clone();
        warp::any().map(move || s.clone())
    }

    // GET: /ws 
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(state_filter.clone())
        .map(|ws: warp::ws::Ws, state| {
            ws.on_upgrade(move |socket| handle_connection(socket, state))
        });
    
    // GET /prices 
    let prices_route = warp::path("prices")
        .and(warp::get())
        .and(state_filter.clone())
        .map(|state| {
            let prices = state.get_prices();
            warp::reply::json(&prices)
        });
    
    // GET /health 
    let health_route = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({"status": "ok"})));
    
    let routes = ws_route.
        or(prices_route).
        or(health_route).
        with(warp::log("crypto_tracker::http"));
    
    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port)
    .parse().expect("Invalid host or port");
    
    info!("Websocket: ws://{}/ws", addr); 
    info!("REST prices: http://{}/prices", addr);
    info!("REST health: http://{}/health", addr);

    warp::serve(routes).run(addr).await;
}

async fn handle_client(ws: Websocket, state: AppState) {
    let (mut tx, mut rx) = ws.split();

    let mut broadcast_rx = state.price_tx.subscribe();

    let snapshot = WsMessage::Snapshot {
        prices: state.get_prices(),
    };

    if tx.send(warp::ws::Message::text(snapshot.to_json())).await.is_err() {
        return;
    }

    info!("Client connected");

    loop {
        tokio::select! {
            // Arm 1: broadcast channel has a new price update
            msg = broadcast_rx.recv() => {
                match msg {
                    Ok(json) => {
                        if tx.send(warp::ws::Message::text(json)).await.is_err() {
                            break; // client disconnected
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        // Client was too slow — resend full snapshot
                        warn!("Client lagged {} messages, resending snapshot", n);
                        let snap = WsMessage::Snapshot {
                            prices: state.get_prices()
                        };
                        if tx.send(warp::ws::Message::text(snap.to_json())).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }

            // Arm 2: client sent us a message
            msg = rx.next() => {
                match msg {
                    Some(Ok(m)) if m.is_ping() => {
                        // Respond to pings to keep connection alive
                        let _ = tx.send(warp::ws::Message::pong(m.into_bytes())).await;
                    }
                    Some(Ok(m)) if m.is_text() => {
                        // Future: handle subscribe/unsubscribe commands
                        info!("Client says: {:?}", m.to_str());
                    }
                    _ => break, // client closed connection or error
                }
            }
        }
    }

    info!("Client disconnected");
}