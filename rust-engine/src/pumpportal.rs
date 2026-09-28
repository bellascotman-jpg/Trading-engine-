use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use thiserror::Error;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{info, warn};

#[derive(Debug, Error)]
pub enum PumpPortalError {
    #[error("websocket connection failed: {0}")]
    Connect(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("subscription serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
}

pub async fn observe_new_tokens(url: &str) -> Result<(), PumpPortalError> {
    let (stream, _) = connect_async(url).await?;
    let (mut write, mut read) = stream.split();

    write
        .send(Message::Text(
            json!({ "method": "subscribeNewToken" }).to_string().into(),
        ))
        .await?;

    info!("PumpPortal new-token subscription established");

    while let Some(message) = read.next().await {
        match message? {
            Message::Text(text) => {
                if let Ok(value) = serde_json::from_str::<Value>(&text) {
                    let mint = value.get("mint").and_then(Value::as_str).unwrap_or("");
                    let name = value.get("name").and_then(Value::as_str).unwrap_or("");
                    if !mint.is_empty() {
                        info!(mint, name, event = "token_created", "new token observed");
                    } else {
                        info!(event = "pumpportal_message", payload = %text, "PumpPortal message");
                    }
                } else {
                    warn!(payload = %text, "unparseable PumpPortal message");
                }
            }
            Message::Ping(payload) => {
                write.send(Message::Pong(payload)).await?;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    Ok(())
}
