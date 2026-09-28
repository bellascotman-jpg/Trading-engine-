use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::time::interval;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub mode: RunMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunMode {
    Observe,
    Paper,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEvent {
    pub received_at_ms: u128,
    pub source: String,
    pub kind: String,
    pub mint: Option<String>,
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let config = EngineConfig {
        // Real-money execution is intentionally not implemented in this stage.
        mode: RunMode::Observe,
    };

    info!(?config.mode, "Solana trading engine starting");
    warn!("LIVE execution is disabled in this development stage");

    let mut heartbeat = interval(Duration::from_secs(10));

    loop {
        heartbeat.tick().await;

        let event = MarketEvent {
            received_at_ms: now_ms(),
            source: "engine".to_string(),
            kind: "heartbeat".to_string(),
            mint: None,
        };

        info!(event = ?event, "engine healthy");
    }
}
