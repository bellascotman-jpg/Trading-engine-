use reqwest::Client;
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("HTTP request failed: {0}")] Http(#[from] reqwest::Error),
    #[error("RPC returned an error: {0}")] Rpc(String),
    #[error("RPC response was missing result")] MissingResult,
}
#[derive(Debug, Clone)]
pub struct LatestBlockhash { pub blockhash: String, pub last_valid_block_height: u64 }
#[derive(Clone)]
pub struct SolanaRpc { client: Client, url: String }
impl SolanaRpc {
    pub fn new(url: impl Into<String>) -> Self { Self { client: Client::new(), url: url.into() } }
    pub async fn call(&self, method:&str, params:Value)->Result<Value,RpcError>{
        let body=json!({"jsonrpc":"2.0","id":1,"method":method,"params":params});
        let v:Value=self.client.post(&self.url).json(&body).send().await?.error_for_status()?.json().await?;
        if let Some(e)=v.get("error"){return Err(RpcError::Rpc(e.to_string()));}
        v.get("result").cloned().ok_or(RpcError::MissingResult)
    }
    pub async fn get_slot(&self)->Result<u64,RpcError>{ self.call("getSlot",json!([{"commitment":"confirmed"}])).await?.as_u64().ok_or(RpcError::MissingResult) }
    pub async fn get_latest_blockhash(&self)->Result<LatestBlockhash,RpcError>{
        let v=self.call("getLatestBlockhash",json!([{"commitment":"confirmed"}])).await?;
        Ok(LatestBlockhash{blockhash:v["value"]["blockhash"].as_str().ok_or(RpcError::MissingResult)?.into(),last_valid_block_height:v["value"]["lastValidBlockHeight"].as_u64().ok_or(RpcError::MissingResult)?})
    }
    pub async fn get_balance(&self,address:&str)->Result<u64,RpcError>{self.call("getBalance",json!([address,{"commitment":"confirmed"}])).await?["value"].as_u64().ok_or(RpcError::MissingResult)}
    pub async fn get_multiple_accounts(&self,addresses:&[String])->Result<Value,RpcError>{self.call("getMultipleAccounts",json!([addresses,{"encoding":"base64","commitment":"confirmed"}])).await}
    pub async fn get_token_supply(&self,mint:&str)->Result<Value,RpcError>{self.call("getTokenSupply",json!([mint,{"commitment":"confirmed"}])).await}
    pub async fn get_token_largest_accounts(&self,mint:&str)->Result<Value,RpcError>{self.call("getTokenLargestAccounts",json!([mint,{"commitment":"confirmed"}])).await}
}
