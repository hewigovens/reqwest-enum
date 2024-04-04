use crate::{http::HTTPBody, target::Target};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "jsonrpc")]
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: &'static str,
    pub id: JsonRpcId,
    pub method: &'static str,
    pub params: Vec<Value>,
}

#[cfg(feature = "jsonrpc")]
impl JsonRpcRequest {
    pub fn new(method: &'static str, params: Vec<Value>, id: u64) -> Self {
        Self {
            jsonrpc: "2.0",
            id: JsonRpcId::Integer(id),
            method,
            params,
        }
    }
}

#[cfg(feature = "jsonrpc")]
impl From<JsonRpcRequest> for reqwest::Body {
    fn from(val: JsonRpcRequest) -> Self {
        serde_json::to_vec(&val).unwrap().into()
    }
}

#[cfg(feature = "jsonrpc")]
impl From<JsonRpcRequest> for HTTPBody {
    fn from(val: JsonRpcRequest) -> Self {
        HTTPBody::from(&val)
    }
}

#[cfg(feature = "jsonrpc")]
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub id: u64,
    pub jsonrpc: String,
    pub result: T,
}

#[cfg(feature = "jsonrpc")]
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcErrorResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub error: String,
}

#[cfg(feature = "jsonrpc")]
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

#[cfg(feature = "jsonrpc")]
impl std::error::Error for JsonRpcError {}

#[cfg(feature = "jsonrpc")]
impl From<reqwest::Error> for JsonRpcError {
    fn from(err: reqwest::Error) -> Self {
        JsonRpcError {
            code: -32603,
            message: format!("Internal error ({})", err),
        }
    }
}

#[cfg(feature = "jsonrpc")]
impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.message, self.code)
    }
}

#[cfg(feature = "jsonrpc")]
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcResult<T> {
    Value(JsonRpcResponse<T>),
    Error(JsonRpcErrorResponse),
}

#[cfg(feature = "jsonrpc")]
pub trait JsonRpcTarget: Target {
    fn method_name(&self) -> &'static str;
    fn params(&self) -> Vec<Value>;
}

#[cfg(feature = "jsonrpc")]
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcId {
    Integer(u64),
    String(String),
}
