use crate::events::{MarketEvent, now_unix_ms};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, Default)]
pub struct FlowSnapshot {
    pub buy_sol: f64,
    pub sell_sol: f64,
    pub buy_count: u64,
    pub sell_count: u64,
    pub unique_buyers: usize,
    pub unique_sellers: usize,
}

#[derive(Debug)]
struct FlowEvent {
    ts_ms: u64,
    sol: f64,
    is_buy: bool,
    trader: Option<String>,
}

#[derive(Debug, Default)]
pub struct FlowAggregator {
    events: VecDeque<FlowEvent>,
}

impl FlowAggregator {
    pub fn record(&mut self, event: &MarketEvent) {
        let (Some(sol), Some(is_buy)) = (event.sol_amount, event.is_buy) else { return; };
        if !sol.is_finite() || sol < 0.0 { return; }
        self.events.push_back(FlowEvent {
            ts_ms: event.observed_at_unix_ms,
            sol,
            is_buy,
            trader: event.trader.clone(),
        });
        self.prune(event.observed_at_unix_ms);
    }

    pub fn snapshot(&mut self) -> FlowSnapshot {
        let now = now_unix_ms();
        self.prune(now);
        let mut out = FlowSnapshot::default();
        let mut buyers = HashSet::new();
        let mut sellers = HashSet::new();
        for e in &self.events {
            if e.is_buy {
                out.buy_sol += e.sol;
                out.buy_count += 1;
                if let Some(t) = &e.trader { buyers.insert(t); }
            } else {
                out.sell_sol += e.sol;
                out.sell_count += 1;
                if let Some(t) = &e.trader { sellers.insert(t); }
            }
        }
        out.unique_buyers = buyers.len();
        out.unique_sellers = sellers.len();
        out
    }

    fn prune(&mut self, now_ms: u64) {
        let cutoff = now_ms.saturating_sub(60_000);
        while self.events.front().is_some_and(|e| e.ts_ms < cutoff) {
            self.events.pop_front();
        }
    }
}
