use crate::events::{EventKind, EventSource, MarketEvent, now_unix_ms};
use crate::flow::FlowAggregator;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use thiserror::Error;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

#[derive(Debug, Error)]
pub enum PumpPortalError {
    #[error("websocket connection failed: {0}")]
    Connect(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("subscription serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
}

fn parse_event(value: &Value) -> Option<MarketEvent> {
    let mint = value.get("mint").and_then(Value::as_str).map(str::to_owned);
    let tx_type = value.get("txType").and_then(Value::as_str).unwrap_or_default();

    let kind = match tx_type {
        "create" => EventKind::TokenCreated,
        "buy" | "sell" => EventKind::Trade,
        _ => return None,
    };

    let is_buy = match tx_type {
        "buy" => Some(true),
        "sell" => Some(false),
        _ => None,
    };

    Some(MarketEvent {
        event_id: format!("pumpportal-{}-{}", now_unix_ms(), mint.as_deref().unwrap_or("unknown")),
        observed_at_unix_ms: now_unix_ms(),
        slot: value.get("slot").and_then(Value::as_u64),
        signature: value.get("signature").and_then(Value::as_str).map(str::to_owned),
        mint,
        symbol: value.get("symbol").and_then(Value::as_str).map(str::to_owned),
        source: EventSource::PumpFun,
        kind,
        sol_amount: value.get("solAmount").and_then(Value::as_f64),
        token_amount: value.get("tokenAmount").and_then(Value::as_f64),
        is_buy,
        trader: value.get("traderPublicKey").and_then(Value::as_str).map(str::to_owned),
    })
}

pub async fn observe_new_tokens(url: &str) -> Result<(), PumpPortalError> {
    let mut flow = FlowAggregator::default();

    loop {
        match connect_async(url).await {
            Ok((stream, _)) => {
                info!("PumpPortal WebSocket connected");
                let (mut write, mut read) = stream.split();

                write.send(Message::Text(json!({"method":"subscribeNewToken"}).to_string().into())).await?;
                write.send(Message::Text(json!({"method":"subscribeTokenTrade","keys":[]}).to_string().into())).await?;
                info!("PumpPortal token subscriptions established");

                while let Some(message) = read.next().await {
                    match message? {
                        Message::Text(text) => {
                            if let Ok(value) = serde_json::from_str::<Value>(&text) {
                                if let Some(event) = parse_event(&value) {
                                    if matches!(event.kind, EventKind::Trade) {
                                        flow.record(&event);
                                        let snapshot = flow.snapshot();
                                        info!(
                                            mint = ?event.mint,
                                            buy_sol = snapshot.buy_sol,
                                            sell_sol = snapshot.sell_sol,
                                            buy_count = snapshot.buy_count,
                                            sell_count = snapshot.sell_count,
                                            unique_buyers = snapshot.unique_buyers,
                                            unique_sellers = snapshot.unique_sellers,
                                            "rolling one-minute trade flow updated"
                                        );
                                    } else {
                                        info!(mint = ?event.mint, "new token observed");
                                    }
                                }
                            } else {
                                warn!(payload=%text, "unparseable PumpPortal message");
                            }
                        }
                        Message::Ping(payload) => write.send(Message::Pong(payload)).await?,
                        Message::Close(_) => break,
                        _ => {}
                    }
                }
                warn!("PumpPortal connection closed; reconnecting");
            }
            Err(e) => error!(%e, "PumpPortal connection failed; retrying"),
        }

        sleep(Duration::from_secs(2)).await;
    }
}
