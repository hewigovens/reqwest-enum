use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: &'static str,
    pub params: Vec<Value>,
}

impl JsonRpcRequest {
    pub fn new(method: &'static str, params: Vec<Value>) -> Self {
        Self {
            jsonrpc: "2.0",
            id: 1,
            method,
            params,
        }
    }
}

impl From<JsonRpcRequest> for reqwest::Body {
    fn from(val: JsonRpcRequest) -> Self {
        serde_json::to_vec(&val).unwrap().into()
    }
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub id: u64,
    pub jsonrpc: String,
    pub result: T,
}
