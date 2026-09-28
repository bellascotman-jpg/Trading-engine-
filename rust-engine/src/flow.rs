use crate::events::{now_unix_ms, MarketEvent};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct FlowSnapshot { pub buy_sol: f64, pub sell_sol: f64, pub buy_count: u64, pub sell_count: u64, pub unique_buyers: usize, pub unique_sellers: usize }
#[derive(Debug)]
struct Entry { ts: u64, sol: f64, buy: bool, trader: Option<String> }
#[derive(Debug, Default)]
pub struct FlowBook { m: HashMap<String, VecDeque<Entry>> }
impl FlowBook {
    pub fn record(&mut self, event: &MarketEvent) {
        let (Some(mint), Some(sol), Some(buy)) = (event.mint.clone(), event.sol_amount, event.is_buy) else { return; };
        if !sol.is_finite() || sol < 0.0 { return; }
        let queue = self.m.entry(mint).or_default();
        queue.push_back(Entry { ts: event.observed_at_unix_ms, sol, buy, trader: event.trader.clone() });
        Self::prune(queue, event.observed_at_unix_ms);
    }
    pub fn snapshot(&mut self, mint: &str) -> FlowSnapshot {
        let now = now_unix_ms(); let queue = self.m.entry(mint.to_string()).or_default(); Self::prune(queue, now);
        let mut snapshot = FlowSnapshot::default(); let (mut buyers, mut sellers) = (HashSet::new(), HashSet::new());
        for entry in queue.iter() {
            if entry.buy { snapshot.buy_sol += entry.sol; snapshot.buy_count += 1; if let Some(trader) = &entry.trader { buyers.insert(trader); } }
            else { snapshot.sell_sol += entry.sol; snapshot.sell_count += 1; if let Some(trader) = &entry.trader { sellers.insert(trader); } }
        }
        snapshot.unique_buyers = buyers.len(); snapshot.unique_sellers = sellers.len(); snapshot
    }
    fn prune(queue: &mut VecDeque<Entry>, now: u64) {
        let cutoff = now.saturating_sub(60_000);
        while queue.front().is_some_and(|entry| entry.ts < cutoff) { queue.pop_front(); }
    }
}
