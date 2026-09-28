mod config;
mod events;
mod pumpportal;
mod rpc;
mod risk;

use config::AppConfig;
use rpc::SolanaRpc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,solana_trading_engine=debug".to_string()),
        )
        .init();

    let config = AppConfig::from_env();

    info!(mode = ?config.mode, "Solana trading engine starting");
    warn!("LIVE transaction broadcast remains disabled until explicit production readiness");

    let rpc = SolanaRpc::new(config.rpc_url.clone());

    match rpc.get_slot().await {
        Ok(slot) => info!(slot, "Solana RPC connection healthy"),
        Err(error) => error!(%error, "Solana RPC health check failed"),
    }

    match rpc.get_latest_blockhash().await {
        Ok(blockhash) => info!(blockhash = %blockhash, "latest confirmed blockhash received"),
        Err(error) => error!(%error, "latest blockhash check failed"),
    }

    let pump_url = config.pumpportal_url.clone();
    tokio::spawn(async move {
        if let Err(error) = pumpportal::observe_new_tokens(&pump_url).await {
            error!(%error, "PumpPortal observer stopped");
        }
    });

    let mut heartbeat = interval(Duration::from_secs(15));
    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                match rpc.get_slot().await {
                    Ok(slot) => info!(slot, "engine heartbeat"),
                    Err(error) => error!(%error, "RPC heartbeat failed"),
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("shutdown signal received");
                break;
            }
        }
    }
}
