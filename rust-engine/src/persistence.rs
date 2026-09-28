use crate::events::SecuritySnapshot;
use reqwest::Client;
use serde_json::{json,Value};
use thiserror::Error;

#[derive(Debug,Error)]
pub enum PersistenceError{#[error("http error: {0}")]Http(#[from]reqwest::Error),#[error("supabase returned {0}: {1}")]Status(u16,String)}

#[derive(Clone)]
pub struct SupabaseStore{base:String,key:String,client:Client}
impl SupabaseStore{
 pub fn from_env()->Option<Self>{Some(Self{base:std::env::var("SUPABASE_URL").ok()?,key:std::env::var("SUPABASE_SECRET_KEY").ok()?,client:Client::new()})}
 async fn post(&self,table:&str,body:Value)->Result<(),PersistenceError>{
  let r=self.client.post(format!("{}/rest/v1/{}",self.base,table)).header("apikey",&self.key).header("Authorization",format!("Bearer {}",self.key)).header("Content-Type","application/json").header("Prefer","return=minimal").json(&body).send().await?;
  if r.status().is_success(){Ok(())}else{Err(PersistenceError::Status(r.status().as_u16(),r.text().await.unwrap_or_default()))}
 }
 pub async fn token(&self,mint:&str,symbol:Option<&str>,created_ms:u64)->Result<(),PersistenceError>{
  self.post("tokens",json!({"mint":mint,"symbol":symbol,"created_at_chain":chrono::DateTime::<chrono::Utc>::from_timestamp_millis(created_ms as i64),"last_seen_at":chrono::Utc::now()})).await
 }
 pub async fn security(&self,s:&SecuritySnapshot)->Result<(),PersistenceError>{
  self.post("token_security",json!({"mint":s.mint,"mint_authority_revoked":s.mint_authority_revoked,"freeze_authority_revoked":s.freeze_authority_revoked,"observed_at":chrono::Utc::now(),"security_status":"partial","evidence":[{"type":"direct_observation","field":"mint_account"}]})).await
 }
 pub async fn trade(&self,e:&crate::events::MarketEvent)->Result<(),PersistenceError>{
  self.post("market_trades",json!({"mint":e.mint,"signature":e.signature,"slot":e.slot,"trader":e.trader,"side":match e.is_buy{Some(true)=>"buy",Some(false)=>"sell",None=>"unknown"},"amount_sol":e.sol_amount,"amount_tokens":e.token_amount,"observed_at":chrono::Utc::now()})).await
 }
 pub async fn health(&self,component:&str,status:&str,latency_ms:Option<f64>)->Result<(),PersistenceError>{
  let r=self.client.post(format!("{}/rest/v1/engine_health",self.base)).header("apikey",&self.key).header("Authorization",format!("Bearer {}",self.key)).header("Content-Type","application/json").header("Prefer","resolution=merge-duplicates,return=minimal").json(&json!({"component":component,"status":status,"latency_ms":latency_ms,"last_heartbeat":chrono::Utc::now()})).send().await?;
  if r.status().is_success(){Ok(())}else{Err(PersistenceError::Status(r.status().as_u16(),r.text().await.unwrap_or_default()))}
 }
}