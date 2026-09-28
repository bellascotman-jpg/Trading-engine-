use crate::events::SecuritySnapshot;
use crate::rpc::SolanaRpc;
use serde_json::Value;
use tracing::warn;

pub async fn inspect_mint(rpc: &SolanaRpc, mint: &str) -> Result<SecuritySnapshot, String> {
    let accounts = rpc.get_multiple_accounts(&[mint.to_string()]).await.map_err(|e| e.to_string())?;
    let account = accounts["value"].get(0).cloned().unwrap_or(Value::Null);
    if account.is_null() { return Err("mint account not found".into()); }

    let owner = account["owner"].as_str().unwrap_or("unknown");
    warn!(mint, owner, "mint account observed; authority decoder must confirm program layout");

    Ok(SecuritySnapshot {
        mint: mint.to_string(),
        mint_authority_revoked: None,
        freeze_authority_revoked: None,
        top_10_non_bonding_pct: None,
        liquidity_usd: None,
        checked_at_unix_ms: crate::events::now_unix_ms(),
    })
}
