use crate::events::{EventKind,EventSource,MarketEvent,now_unix_ms};
use crate::{flow::FlowBook,persistence::SupabaseStore,rpc::SolanaRpc,security};
use futures_util::{SinkExt,StreamExt};
use serde_json::{json,Value};
use thiserror::Error;
use tokio::time::{sleep,Duration};
use tokio_tungstenite::{connect_async,tungstenite::Message};
use tracing::{error,info,warn};

#[derive(Debug,Error)]
pub enum PumpPortalError{#[error("websocket connection failed: {0}")]Connect(#[from]tokio_tungstenite::tungstenite::Error),#[error("subscription serialization failed: {0}")]Serialize(#[from]serde_json::Error)}

fn parse_event(value:&Value)->Option<MarketEvent>{
 let mint=value.get("mint").and_then(Value::as_str).map(str::to_owned);let tx=value.get("txType").and_then(Value::as_str).unwrap_or_default();
 let kind=match tx{"create"=>EventKind::TokenCreated,"buy"|"sell"=>EventKind::Trade,_=>return None};
 Some(MarketEvent{event_id:format!("pumpportal-{}-{}",now_unix_ms(),mint.as_deref().unwrap_or("unknown")),observed_at_unix_ms:now_unix_ms(),slot:value.get("slot").and_then(Value::as_u64),signature:value.get("signature").and_then(Value::as_str).map(str::to_owned),mint,symbol:value.get("symbol").and_then(Value::as_str).map(str::to_owned),source:EventSource::PumpFun,kind,sol_amount:value.get("solAmount").and_then(Value::as_f64),token_amount:value.get("tokenAmount").and_then(Value::as_f64),is_buy:match tx{"buy"=>Some(true),"sell"=>Some(false),_=>None},trader:value.get("traderPublicKey").and_then(Value::as_str).map(str::to_owned)})
}

pub async fn observe_new_tokens(url:&str,rpc:SolanaRpc,store:Option<SupabaseStore>)->Result<(),PumpPortalError>{
 let mut flow=FlowBook::default();
 loop{match connect_async(url).await{Ok((stream,_))=>{info!("PumpPortal connected");let(mut write,mut read)=stream.split();write.send(Message::Text(json!({"method":"subscribeNewToken"}).to_string().into())).await?;
  while let Some(message)=read.next().await{match message?{Message::Text(text)=>{if let Ok(value)=serde_json::from_str::<Value>(&text){if let Some(event)=parse_event(&value){match event.kind{
   EventKind::TokenCreated=>{if let Some(mint)=&event.mint{if let Some(db)=&store{let _=db.token(mint,event.symbol.as_deref(),event.observed_at_unix_ms).await;if let Ok(s)=security::inspect_mint(&rpc,mint).await{let _=db.security(&s).await;}}let sub=json!({"method":"subscribeTokenTrade","keys":[mint]}).to_string();if let Err(e)=write.send(Message::Text(sub.into())).await{warn!(%e,mint,"trade subscription failed");}info!(mint,symbol=?event.symbol,"new token persisted");}}
   EventKind::Trade=>{flow.record(&event);if let Some(db)=&store{let _=db.trade(&event).await;}if let Some(m)=event.mint.as_deref(){let s=flow.snapshot(m);info!(mint=m,buy_sol=s.buy_sol,sell_sol=s.sell_sol,buy_count=s.buy_count,sell_count=s.sell_count,unique_buyers=s.unique_buyers,unique_sellers=s.unique_sellers,"trade flow updated");}}
   _=>{}
 }}}}else{warn!(payload=%text,"unparseable PumpPortal message")}},Message::Ping(p)=>write.send(Message::Pong(p)).await?,Message::Close(_)=>break,_=>{}}}
 warn!("PumpPortal closed; reconnecting");},Err(e)=>error!(%e,"PumpPortal connection failed; retrying")};sleep(Duration::from_secs(2)).await;}
}