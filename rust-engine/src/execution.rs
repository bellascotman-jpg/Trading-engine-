use crate::rpc::SolanaRpc;
use serde::{Deserialize,Serialize};
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SimulationResult{pub ok:bool,pub err:Option<String>,pub units_consumed:Option<u64>,pub fee_lamports:Option<u64>}
pub async fn simulate(rpc:&SolanaRpc,encoded:&str)->Result<SimulationResult,String>{let v=rpc.simulate_transaction(encoded).await.map_err(|e|e.to_string())?;let err=v.get("err").cloned().filter(|x|!x.is_null()).map(|x|x.to_string());Ok(SimulationResult{ok:err.is_none(),err,units_consumed:v.get("unitsConsumed").and_then(|x|x.as_u64()),fee_lamports:v.get("fee").and_then(|x|x.as_u64())})}
pub fn compute_limit(units:u64)->u32{((units as f64*1.10).ceil()as u32).clamp(1,1_400_000)}
pub fn priority_fee_lamports(cu:u32,price_micro:u64)->u64{((cu as u128*price_micro as u128+999_999)/1_000_000)as u64}