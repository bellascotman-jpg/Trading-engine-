use crate::{dexscreener::EnrichedMarket, events::SecuritySnapshot, flow::FlowSnapshot};

#[derive(Debug, Clone, serde::Serialize)]
pub struct AuditResult { pub verdict: String, pub audit_score: f64, pub rug_confidence: f64, pub honeypot_confidence: f64, pub scam_confidence: f64, pub reasons: Vec<String>, pub evidence: Vec<String> }
#[derive(Debug, Clone, serde::Serialize)]
pub struct Scenario { pub multiple: f64, pub probability: f64, pub horizon: String, pub basis: Vec<String> }

pub fn audit(sec: &SecuritySnapshot, market: Option<&EnrichedMarket>, flow: &FlowSnapshot) -> AuditResult {
    let mut score: f64 = 0.0; let mut reasons = Vec::new(); let mut evidence = Vec::new();
    if sec.mint_authority_revoked == Some(true) { score += 20.0; evidence.push("Mint authority directly observed as revoked".into()); } else { reasons.push("Mint authority is not confirmed revoked".into()); }
    if sec.freeze_authority_revoked == Some(true) { score += 20.0; evidence.push("Freeze authority directly observed as revoked".into()); } else { reasons.push("Freeze authority is not confirmed revoked".into()); }
    match sec.top_10_non_bonding_pct { Some(v) if v <= 20.0 => { score += 20.0; evidence.push(format!("Top holder concentration observed at {:.2}%", v)); }, Some(v) => reasons.push(format!("Top holder concentration is {:.2}%", v)), None => reasons.push("Holder concentration is unavailable".into()) }
    match market { Some(m) => { if m.price_usd.is_some() { score += 15.0; } evidence.extend(m.evidence.clone()); }, None => reasons.push("No DEX market observation yet".into()) }
    let ratio = if flow.sell_sol > 0.0 { flow.buy_sol / flow.sell_sol } else { f64::INFINITY };
    if ratio >= 2.0 && flow.unique_buyers >= 2 { score += 25.0; evidence.push(format!("1m buy/sell SOL-flow ratio {:.2}", ratio)); } else { reasons.push(format!("1m flow gate not met; ratio {:.2}", ratio)); }
    let rug = (100.0 - score).clamp(0.0, 100.0);
    let scam = ((reasons.len() as f64 * 18.0) + if sec.top_10_non_bonding_pct.unwrap_or(0.0) > 40.0 { 30.0 } else { 0.0 }).clamp(0.0, 100.0);
    let honeypot = if market.is_none() { 80.0 } else { 55.0 };
    let verdict = if score >= 85.0 && reasons.is_empty() { "ENTRY_CANDIDATE" } else if score >= 55.0 { "WATCH" } else { "BLOCKED" };
    AuditResult { verdict: verdict.into(), audit_score: score.min(100.0), rug_confidence: rug, honeypot_confidence: honeypot, scam_confidence: scam, reasons, evidence }
}

pub fn scenarios(audit: &AuditResult, market: Option<&EnrichedMarket>, flow: &FlowSnapshot) -> Vec<Scenario> {
    if audit.verdict != "ENTRY_CANDIDATE" { return Vec::new(); }
    let flow_ratio = if flow.sell_sol > 0.0 { flow.buy_sol / flow.sell_sol } else { 5.0 };
    let momentum = (flow_ratio.min(5.0) / 5.0) * 0.35 + (audit.audit_score / 100.0) * 0.65;
    let liquidity_ok = market.is_some();
    let prior = [(2.0,0.55,"15-60m"),(5.0,0.22,"1-6h"),(10.0,0.10,"1-24h"),(25.0,0.035,"6-48h"),(50.0,0.01,"1-7d"),(100.0,0.002,"1-14d")];
    prior.into_iter().map(|(multiple,p,horizon)| Scenario { multiple, probability:(p*momentum).clamp(0.0,1.0), horizon:horizon.into(), basis:vec!["Evidence-gated scenario model; not a guarantee".into(), format!("Audit score {:.1}",audit.audit_score), format!("Flow ratio {:.2}",flow_ratio), format!("Market evidence available: {}",liquidity_ok)] }).collect()
}
