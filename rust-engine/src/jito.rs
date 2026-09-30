use reqwest::Client;
use serde_json::{json,Value};
use thiserror::Error;
#[derive(Debug,Error)] pub enum JitoError { #[error("http: {0}")] Http(#[from] reqwest::Error), #[error("jito rpc error: {0}")] Rpc(String), #[error("missing result")] MissingResult }
#[derive(Clone)] pub struct JitoClient { client:Client, endpoint:String }
impl JitoClient { pub fn new(endpoint:Option<String>)->Self{Self{client:Client::new(),endpoint:endpoint.unwrap_or_else(||"https://mainnet.block-engine.jito.wtf/api/v1".into())}}
 pub async fn send_bundle(&self,txs:&[String])->Result<String,JitoError>{if txs.is_empty()||txs.len()>5{return Err(JitoError::Rpc("bundle must contain 1-5 signed transactions".into()));}let r:Value=self.client.post(format!("{}/bundles",self.endpoint)).json(&json!({"jsonrpc":"2.0","id":1,"method":"sendBundle","params":[txs,{"encoding":"base64"}]})).send().await?.error_for_status()?.json().await?;if let Some(e)=r.get("error"){return Err(JitoError::Rpc(e.to_string()));}r.get("result").and_then(Value::as_str).map(str::to_owned).ok_or(JitoError::MissingResult)}
 pub async fn bundle_status(&self,id:&str)->Result<Value,JitoError>{let r:Value=self.client.post(format!("{}/getBundleStatuses",self.endpoint)).json(&json!({"jsonrpc":"2.0","id":1,"method":"getBundleStatuses","params":[[id]]})).send().await?.error_for_status()?.json().await?;if let Some(e)=r.get("error"){return Err(JitoError::Rpc(e.to_string()));}Ok(r.get("result").cloned().unwrap_or(Value::Null))}
 pub async fn tip_accounts(&self)->Result<Vec<String>,JitoError>{let r:Value=self.client.post(format!("{}/getTipAccounts",self.endpoint)).json(&json!({"jsonrpc":"2.0","id":1,"method":"getTipAccounts","params":[]})).send().await?.error_for_status()?.json().await?;if let Some(e)=r.get("error"){return Err(JitoError::Rpc(e.to_string()));}Ok(r.get("result").and_then(Value::as_array).map(|a|a.iter().filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default())}
}
