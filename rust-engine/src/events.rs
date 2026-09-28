use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_unix_ms() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEvent {
    pub event_id: String, pub observed_at_unix_ms: u64, pub slot: Option<u64>, pub signature: Option<String>,
    pub mint: Option<String>, pub symbol: Option<String>, pub source: EventSource, pub kind: EventKind,
    pub sol_amount: Option<f64>, pub token_amount: Option<f64>, pub is_buy: Option<bool>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSource { PumpFun, SolanaRpc, Yellowstone, Raydium, DexScreener, Unknown }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind { TokenCreated, Trade, PoolCreated, PoolMigrated, AccountUpdated, Unknown }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySnapshot {
    pub mint: String, pub mint_authority_revoked: Option<bool>, pub freeze_authority_revoked: Option<bool>,
    pub top_10_non_bonding_pct: Option<f64>, pub liquidity_usd: Option<f64>, pub checked_at_unix_ms: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Decision { Pass, Reject, Watch }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDecision { pub decision: Decision, pub reasons: Vec<String>, pub score: Option<f64> }
