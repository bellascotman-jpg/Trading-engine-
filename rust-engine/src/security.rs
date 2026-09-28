use crate::{events::{now_unix_ms, SecuritySnapshot}, rpc::SolanaRpc};
use base64::Engine;
use serde_json::Value;

const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEZ3nZ5Q7YhYV8Y4f5b7f2bYw5";

fn authority_revoked(data: &[u8], offset: usize) -> Option<bool> {
    let bytes = data.get(offset..offset + 4)?;
    let tag = u32::from_le_bytes(bytes.try_into().ok()?);
    Some(tag == 0)
}

fn decode_mint(value: &Value) -> Result<(bool, bool), String> {
    let owner = value.get("owner").and_then(Value::as_str).ok_or("missing token program")?;
    if owner != TOKEN_PROGRAM && owner != TOKEN_2022_PROGRAM { return Err("account is not an SPL token mint".into()); }
    let encoded = value.get("data").and_then(Value::as_array).and_then(|a| a.first()).and_then(Value::as_str).ok_or("missing base64 mint data")?;
    let data = base64::engine::general_purpose::STANDARD.decode(encoded).map_err(|e| e.to_string())?;
    if data.len() < 82 { return Err("mint account data too short".into()); }
    Ok((authority_revoked(&data, 0).ok_or("invalid mint authority option")?, authority_revoked(&data, 46).ok_or("invalid freeze authority option")?))
}

pub async fn inspect_mint(rpc: &SolanaRpc, mint: &str) -> Result<SecuritySnapshot, String> {
    let response = rpc.get_multiple_accounts(&[mint.to_string()]).await.map_err(|e| e.to_string())?;
    let account = response.get("value").and_then(|v| v.get(0)).cloned().unwrap_or(Value::Null);
    if account.is_null() { return Err("mint account not found".into()); }
    let (mint_revoked, freeze_revoked) = decode_mint(&account)?;
    Ok(SecuritySnapshot { mint: mint.to_owned(), mint_authority_revoked: Some(mint_revoked), freeze_authority_revoked: Some(freeze_revoked), top_10_non_bonding_pct: None, liquidity_usd: None, checked_at_unix_ms: now_unix_ms() })
}
