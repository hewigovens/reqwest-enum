use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: &'static str,
    pub id: JsonRpcId,
    pub method: &'static str,
    pub params: Vec<Value>,
}

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

impl From<JsonRpcRequest> for reqwest::Body {
    fn from(val: JsonRpcRequest) -> Self {
        serde_json::to_vec(&val).unwrap().into()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub id: JsonRpcId,
    pub jsonrpc: String,
    pub result: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcErrorResponse {
    pub jsonrpc: String,
    pub id: JsonRpcId,
    pub error: JsonRpcError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

impl std::error::Error for JsonRpcError {}

impl From<reqwest::Error> for JsonRpcError {
    fn from(err: reqwest::Error) -> Self {
        JsonRpcError {
            code: -32603,
            message: format!("Internal error ({})", err),
        }
    }
}

impl From<crate::Error> for JsonRpcError {
    fn from(err: crate::Error) -> Self {
        match err {
            crate::Error::Reqwest(e) => {
                // Reuse the existing From<reqwest::Error> for JsonRpcError
                e.into()
            }
            #[cfg(feature = "middleware")]
            crate::Error::ReqwestMiddleware(e) => JsonRpcError {
                code: -32603, // Internal error
                message: format!("Middleware error: {}", e),
            },
            crate::Error::SerdeJson(e) => JsonRpcError {
                code: -32603, // Internal error (could also be Parse error -32700 depending on context)
                message: format!("Serialization/deserialization error: {}", e),
            },
        }
    }
}

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.message, self.code)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcResult<T> {
    Value(JsonRpcResponse<T>),
    Error(JsonRpcErrorResponse),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcId {
    Integer(u64),
    String(String),
}
