use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingMode {
    Observe,
    Paper,
    Simulation,
    Live,
}

impl OperatingMode {
    pub fn from_env() -> Self {
        match env::var("TRADING_MODE")
            .unwrap_or_else(|_| "observe".to_string())
            .to_lowercase()
            .as_str()
        {
            "paper" => Self::Paper,
            "simulation" | "simulate" => Self::Simulation,
            "live" => Self::Live,
            _ => Self::Observe,
        }
    }

    pub fn allows_broadcast(self) -> bool {
        matches!(self, Self::Live)
    }
}

#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub max_position_sol: f64,
    pub max_open_positions: usize,
    pub max_daily_loss_sol: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub trailing_stop_pct: f64,
    pub max_slippage_bps: u64,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_position_sol: 0.25,
            max_open_positions: 2,
            max_daily_loss_sol: 1.0,
            stop_loss_pct: 20.0,
            take_profit_pct: 100.0,
            trailing_stop_pct: 15.0,
            max_slippage_bps: 1500,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub mode: OperatingMode,
    pub rpc_url: String,
    pub websocket_url: String,
    pub pumpportal_url: String,
    pub supabase_url: Option<String>,
    pub supabase_secret_key: Option<String>,
    pub risk: RiskConfig,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            mode: OperatingMode::from_env(),
            rpc_url: env::var("SOLANA_RPC_URL")
                .or_else(|_| env::var("HELIUS_RPC_URL"))
                .unwrap_or_else(|_| "https://api.mainnet.solana.com".to_string()),
            websocket_url: env::var("SOLANA_WS_URL")
                .or_else(|_| env::var("HELIUS_WS_URL"))
                .unwrap_or_else(|_| "wss://api.mainnet.solana.com".to_string()),
            pumpportal_url: env::var("PUMPPORTAL_WS_URL")
                .unwrap_or_else(|_| "wss://pumpportal.fun/api/data".to_string()),
            supabase_url: env::var("SUPABASE_URL").ok(),
            supabase_secret_key: env::var("SUPABASE_SECRET_KEY").ok(),
            risk: RiskConfig::default(),
        }
    }
}
