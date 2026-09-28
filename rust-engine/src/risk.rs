use crate::config::RiskConfig;
use crate::events::{Decision, RiskDecision, SecuritySnapshot};
use crate::flow::FlowSnapshot;

pub struct RiskEngine {
    pub config: RiskConfig,
}

impl RiskEngine {
    pub fn evaluate(&self, security: &SecuritySnapshot, buy_sol: f64, sell_sol: f64) -> RiskDecision {
        let mut reasons = Vec::new();

        if security.mint_authority_revoked != Some(true) {
            reasons.push("mint authority is not confirmed revoked".to_string());
        }
        if security.freeze_authority_revoked != Some(true) {
            reasons.push("freeze authority is not confirmed revoked".to_string());
        }
        if security.top_10_non_bonding_pct.unwrap_or(100.0) > 20.0 {
            reasons.push("top-10 non-bonding concentration exceeds configured threshold".to_string());
        }
        if security.liquidity_usd.unwrap_or(0.0) <= 0.0 {
            reasons.push("liquidity is missing or zero".to_string());
        }

        let flow_ratio = if sell_sol > 0.0 { buy_sol / sell_sol } else { f64::INFINITY };
        if flow_ratio < 2.0 {
            reasons.push("buy/sell SOL-flow ratio is below the initial 2:1 threshold".to_string());
        }

        let decision = if reasons.is_empty() { Decision::Pass } else { Decision::Reject };
        RiskDecision { decision, reasons, score: None }
    }
}

#[derive(Debug, Default)]
pub struct CircuitBreaker {
    pub halted: bool,
    pub daily_loss_sol: f64,
    pub open_positions: usize,
}

impl CircuitBreaker {
    pub fn can_open(&self, cfg: &RiskConfig) -> bool {
        !self.halted
            && self.daily_loss_sol < cfg.max_daily_loss_sol
            && self.open_positions < cfg.max_open_positions
    }

    pub fn record_loss(&mut self, loss_sol: f64, cfg: &RiskConfig) {
        self.daily_loss_sol += loss_sol.max(0.0);
        if self.daily_loss_sol >= cfg.max_daily_loss_sol {
            self.halted = true;
        }
    }
}


impl RiskEngine {
    pub fn evaluate_flow(&self, flow: &FlowSnapshot) -> RiskDecision {
        let mut reasons = Vec::new();

        if flow.buy_count + flow.sell_count == 0 {
            reasons.push("no one-minute trade flow observed".to_string());
        }
        if flow.buy_sol <= 0.0 {
            reasons.push("no buy-side SOL flow observed".to_string());
        }
        if flow.sell_sol > 0.0 && flow.buy_sol / flow.sell_sol < 2.0 {
            reasons.push("buy/sell SOL-flow ratio is below the configured 2:1 gate".to_string());
        }
        if flow.unique_buyers < 2 {
            reasons.push("buyer breadth is too low for the current strategy".to_string());
        }

        let ratio = if flow.sell_sol > 0.0 { flow.buy_sol / flow.sell_sol } else { f64::INFINITY };
        let breadth = flow.unique_buyers as f64 / (flow.unique_sellers.max(1) as f64);
        let score = (ratio.min(5.0) / 5.0 * 60.0 + breadth.min(3.0) / 3.0 * 40.0).clamp(0.0, 100.0);

        let decision = if reasons.is_empty() { Decision::Pass } else { Decision::Watch };
        RiskDecision { decision, reasons, score: Some(score) }
    }
}
