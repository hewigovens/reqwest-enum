use crate::http::HTTPBody;
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
    pub fn new(method: &'static str, params: Vec<Value>, id: u64) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
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

impl From<JsonRpcRequest> for HTTPBody {
    fn from(val: JsonRpcRequest) -> Self {
        HTTPBody::from(&val)
    }
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub id: u64,
    pub jsonrpc: String,
    pub result: T,
}
