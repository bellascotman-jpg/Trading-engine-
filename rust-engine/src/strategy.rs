use crate::{events::{Decision, SecuritySnapshot}, flow::FlowSnapshot, market::MarketMetrics};

#[derive(Debug, Clone, serde::Serialize)]
pub struct OpportunityScore {
    pub score: f64,
    pub decision: Decision,
    pub reasons: Vec<String>,
    pub risk_gate: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ReturnScenario {
    pub multiple: f64,
    pub probability: f64,
    pub horizon: &'static str,
    pub evidence: Vec<String>,
}

pub fn score(s: &SecuritySnapshot, f: &FlowSnapshot, m: &MarketMetrics) -> OpportunityScore {
    let mut reasons = Vec::new();
    let mut score = 0.0;
    if s.mint_authority_revoked == Some(true) { score += 20.0; } else { reasons.push("mint authority not confirmed".into()); }
    if s.freeze_authority_revoked == Some(true) { score += 20.0; } else { reasons.push("freeze authority not confirmed".into()); }
    if s.top_10_non_bonding_pct.is_some_and(|v| v <= 20.0) { score += 20.0; } else { reasons.push("holder concentration unavailable or above threshold".into()); }
    if s.liquidity_usd.is_some_and(|v| v > 0.0) { score += 15.0; } else { reasons.push("liquidity unavailable".into()); }
    let ratio = if f.sell_sol > 0.0 { f.buy_sol / f.sell_sol } else { f64::INFINITY };
    if ratio >= 2.0 && f.unique_buyers >= 2 { score += 15.0; } else { reasons.push("flow gate not met".into()); }
    if m.price_sol.is_some() { score += 10.0; } else { reasons.push("price unavailable".into()); }
    let decision = if reasons.is_empty() { Decision::Pass } else { Decision::Watch };
    OpportunityScore {
        score: score.min(100.0),
        decision,
        risk_gate: if reasons.is_empty() { "PASS".into() } else { "BLOCKED/INCOMPLETE".into() },
        reasons,
    }
}

pub fn scenarios(s: &SecuritySnapshot, f: &FlowSnapshot, m: &MarketMetrics) -> Vec<ReturnScenario> {
    let base = score(s, f, m);
    if base.decision != Decision::Pass { return vec![]; }
    let momentum = (f.buy_sol - f.sell_sol).max(0.0);
    let activity = (f.buy_count + f.sell_count) as f64;
    let quality = ((base.score / 100.0)
        * (momentum / (momentum + 1.0))
        * (activity / (activity + 10.0)))
        .clamp(0.0, 1.0);
    vec![
        (2.0, 0.55, "short"),
        (5.0, 0.25, "short"),
        (10.0, 0.12, "medium"),
        (25.0, 0.04, "medium"),
        (50.0, 0.01, "long"),
        (100.0, 0.002, "long"),
    ]
    .into_iter()
    .map(|(multiple, prior, horizon)| ReturnScenario {
        multiple,
        probability: (prior * quality).clamp(0.0, 1.0),
        horizon,
        evidence: vec!["Calculated only from observed security and flow inputs; not a guarantee.".into()],
    })
    .collect()
}
