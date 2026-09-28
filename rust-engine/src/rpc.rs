use reqwest::Client;
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("RPC returned an error: {0}")]
    Rpc(String),
    #[error("RPC response was missing result")]
    MissingResult,
}

#[derive(Clone)]
pub struct SolanaRpc {
    client: Client,
    url: String,
}

impl SolanaRpc {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            url: url.into(),
        }
    }

    pub async fn call(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        let response = self.client.post(&self.url).json(&body).send().await?;
        let response = response.error_for_status()?;
        let value: Value = response.json().await?;

        if let Some(error) = value.get("error") {
            return Err(RpcError::Rpc(error.to_string()));
        }

        value.get("result").cloned().ok_or(RpcError::MissingResult)
    }

    pub async fn get_slot(&self) -> Result<u64, RpcError> {
        let result = self.call("getSlot", json!([{ "commitment": "confirmed" }])).await?;
        result.as_u64().ok_or(RpcError::MissingResult)
    }

    pub async fn get_latest_blockhash(&self) -> Result<String, RpcError> {
        let result = self
            .call("getLatestBlockhash", json!([{ "commitment": "confirmed" }]))
            .await?;

        result["value"]["blockhash"]
            .as_str()
            .map(str::to_owned)
            .ok_or(RpcError::MissingResult)
    }

    pub async fn get_balance(&self, address: &str) -> Result<u64, RpcError> {
        let result = self
            .call("getBalance", json!([address, { "commitment": "confirmed" }]))
            .await?;

        result["value"].as_u64().ok_or(RpcError::MissingResult)
    }
}
