extern crate reqwest;
extern crate reqwest_enum;
use reqwest_enum::jsonrpc::JsonRpcRequest;
use reqwest_enum::target::Target;
use serde_json::Value;
use std::collections::HashMap;

pub enum EthereumRPC {
    ChainId,
    GasPrice,
    BlockNumber,
    GetBalance(&'static str),
    SendRawTransaction(&'static str),
}

impl EthereumRPC {
    pub fn method_name(&self) -> &'static str {
        match self {
            EthereumRPC::ChainId => "eth_chainId",
            EthereumRPC::GasPrice => "eth_gasPrice",
            EthereumRPC::BlockNumber => "eth_blockNumber",
            EthereumRPC::GetBalance(_) => "eth_getBalance",
            EthereumRPC::SendRawTransaction(_) => "eth_sendRawTransaction",
        }
    }

    pub fn params(&self) -> Vec<Value> {
        match self {
            EthereumRPC::GetBalance(address) => vec![
                Value::String(address.to_string()),
                Value::String("latest".into()),
            ],
            EthereumRPC::SendRawTransaction(tx) => {
                vec![Value::String(tx.to_string())]
            }
            _ => vec![],
        }
    }
}

impl Target for EthereumRPC {
    fn base_url(&self) -> &'static str {
        "https://rpc.ankr.com"
    }

    fn method(&self) -> reqwest::Method {
        reqwest::Method::POST
    }

    fn path(&self) -> &'static str {
        "/eth"
    }

    fn query(&self) -> HashMap<&'static str, &'static str> {
        HashMap::default()
    }

    fn headers(&self) -> HashMap<&'static str, &'static str> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type", "application/json");
        headers
    }

    fn body(&self) -> reqwest::Body {
        let method = self.method_name();
        let params = self.params();
        let req = JsonRpcRequest::new(method, params);
        req.into()
    }
}
