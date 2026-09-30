use crate::{events::{now_unix_ms, SecuritySnapshot}, rpc::SolanaRpc};
use base64::Engine;
use serde_json::Value;
const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEZ3nZ5Q7YhYV8Y4uK7V4g8s8J1";
fn authority_revoked(data: &[u8], offset: usize) -> Option<bool> { Some(u32::from_le_bytes(data.get(offset..offset+4)?.try_into().ok()?) == 0) }
fn decode_mint(value: &Value) -> Result<(bool,bool),String> {
    let owner=value.get("owner").and_then(Value::as_str).ok_or("missing token program")?;
    if owner!=TOKEN_PROGRAM && owner!=TOKEN_2022_PROGRAM { return Err("account is not an SPL token mint".into()); }
    let encoded=value.get("data").and_then(Value::as_array).and_then(|a|a.first()).and_then(Value::as_str).ok_or("missing base64 mint data")?;
    let data=base64::engine::general_purpose::STANDARD.decode(encoded).map_err(|e|e.to_string())?;
    if data.len()<82 { return Err("mint account data too short".into()); }
    Ok((authority_revoked(&data,0).ok_or("invalid mint authority option")?,authority_revoked(&data,46).ok_or("invalid freeze authority option")?))
}
async fn concentration(rpc:&SolanaRpc,mint:&str)->Option<f64>{
    let supply=rpc.get_token_supply(mint).await.ok()?; let total=supply["value"]["amount"].as_str()?.parse::<f64>().ok()?;
    if total<=0.0{return None;} let largest=rpc.get_token_largest_accounts(mint).await.ok()?; let accounts=largest["value"].as_array()?;
    let top: f64=accounts.iter().take(10).filter_map(|a|a["amount"].as_str()?.parse::<f64>().ok()).sum(); Some((top/total*100.0).min(100.0))
}
pub async fn inspect_mint(rpc:&SolanaRpc,mint:&str)->Result<SecuritySnapshot,String>{
    let response=rpc.get_multiple_accounts(&[mint.to_string()]).await.map_err(|e|e.to_string())?;
    let account=response.get("value").and_then(|v|v.get(0)).cloned().unwrap_or(Value::Null); if account.is_null(){return Err("mint account not found".into());}
    let (mint_revoked,freeze_revoked)=decode_mint(&account)?; let top=concentration(rpc,mint).await;
    Ok(SecuritySnapshot{mint:mint.to_owned(),mint_authority_revoked:Some(mint_revoked),freeze_authority_revoked:Some(freeze_revoked),top_10_non_bonding_pct:top,liquidity_usd:None,checked_at_unix_ms:now_unix_ms()})
}
