mod config; mod events; mod flow; mod pumpportal; mod rpc; mod risk; mod security;
use config::AppConfig; use rpc::SolanaRpc; use std::time::Duration; use tokio::time::interval; use tracing::{error,info,warn};

#[tokio::main]
async fn main(){
 tracing_subscriber::fmt().with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_|"info,solana_trading_engine=debug".into())).init();
 let config=AppConfig::from_env(); info!(mode=?config.mode,"Solana trading engine starting"); warn!("LIVE transaction broadcast remains disabled");
 let rpc=SolanaRpc::new(config.rpc_url.clone());
 match rpc.get_slot().await{Ok(slot)=>info!(slot,"Solana RPC connection healthy"),Err(e)=>error!(%e,"Solana RPC health check failed")}
 match rpc.get_latest_blockhash().await{Ok(h)=>info!(blockhash=%h.blockhash,last_valid_block_height=h.last_valid_block_height,"blockhash check passed"),Err(e)=>error!(%e,"blockhash check failed")}
 let pump_url=config.pumpportal_url.clone(); tokio::spawn(async move{if let Err(e)=pumpportal::observe_new_tokens(&pump_url).await{error!(%e,"PumpPortal observer stopped")}});
 let mut heartbeat=interval(Duration::from_secs(15));
 loop{tokio::select!{_=heartbeat.tick()=>match rpc.get_slot().await{Ok(slot)=>info!(slot,mode=?config.mode,"engine heartbeat"),Err(e)=>error!(%e,"RPC heartbeat failed")},_=tokio::signal::ctrl_c()=>{info!("shutdown signal received");break}}}
}
