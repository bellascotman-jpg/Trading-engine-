use crate::{config::RiskConfig, events::SecuritySnapshot, flow::FlowBook, market::MarketMetrics, portfolio::PaperPortfolio, strategy};
use std::collections::HashMap;

pub struct EngineState { pub flows: FlowBook, pub paper: PaperPortfolio, pub risks: RiskConfig, pub markets: HashMap<String, MarketMetrics> }
impl EngineState {
    pub fn new(risks: RiskConfig) -> Self { Self { flows: FlowBook::default(), paper: PaperPortfolio::default(), risks, markets: HashMap::new() } }
    pub fn evaluate(&mut self, mint: &str, security: &SecuritySnapshot) -> strategy::OpportunityScore {
        let flow = self.flows.snapshot(mint);
        let market = self.markets.get(mint).cloned().unwrap_or_default();
        strategy::score(security, &flow, &market)
    }
}
