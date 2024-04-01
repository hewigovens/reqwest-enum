extern crate reqwest;
extern crate reqwest_enum;
use reqwest_enum::jsonrpc::{JsonRpcRequest, JsonRpcTarget};
use reqwest_enum::{
    http::{HTTPBody, HTTPMethod},
    target::Target,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct TransactionObject {
    pub from: String,
    pub to: String,
    pub gas: Option<String>,
    pub gas_price: Option<String>,
    pub value: Option<String>,
    pub data: Option<String>,
}

pub enum BlockParameter {
    Number(&'static str),
    Latest,
    Earliest,
    Pending,
    Safe,
    Finalized,
}

impl From<&BlockParameter> for &'static str {
    fn from(val: &BlockParameter) -> Self {
        match val {
            BlockParameter::Number(val) => val,
            BlockParameter::Latest => "latest",
            BlockParameter::Earliest => "earliest",
            BlockParameter::Pending => "pending",
            BlockParameter::Safe => "safe",
            BlockParameter::Finalized => "finalized",
        }
    }
}

impl From<&BlockParameter> for serde_json::Value {
    fn from(val: &BlockParameter) -> Self {
        let str: &str = val.into();
        serde_json::Value::String(str.to_string())
    }
}

pub enum EthereumRPC {
    ChainId,
    GasPrice,
    BlockNumber,
    GetBalance(&'static str),
    GetBlockByNumber(&'static str, bool),
    GetTransactionCount(&'static str, BlockParameter),
    Call(TransactionObject, BlockParameter),
    EstimateGas(TransactionObject),
    SendRawTransaction(&'static str),
}

impl JsonRpcTarget for EthereumRPC {
    fn method_name(&self) -> &'static str {
        match self {
            EthereumRPC::ChainId => "eth_chainId",
            EthereumRPC::GasPrice => "eth_gasPrice",
            EthereumRPC::BlockNumber => "eth_blockNumber",
            EthereumRPC::GetBalance(_) => "eth_getBalance",
            EthereumRPC::SendRawTransaction(_) => "eth_sendRawTransaction",
            EthereumRPC::GetBlockByNumber(_, _) => "eth_getBlockByNumber",
            EthereumRPC::GetTransactionCount(_, _) => "eth_getTransactionCount",
            EthereumRPC::Call(_, _) => "eth_call",
            EthereumRPC::EstimateGas(_) => "eth_estimateGas",
        }
    }

    fn params(&self) -> Vec<Value> {
        match self {
            EthereumRPC::GetBalance(address) => vec![
                Value::String(address.to_string()),
                (&BlockParameter::Latest).into(),
            ],
            EthereumRPC::SendRawTransaction(tx) => {
                vec![Value::String(tx.to_string())]
            }
            EthereumRPC::GetBlockByNumber(block, full) => {
                vec![
                    Value::String(block.to_string()),
                    Value::Bool(full.to_owned()),
                ]
            }
            EthereumRPC::GetTransactionCount(address, block) => {
                vec![Value::String(address.to_string()), block.into()]
            }
            EthereumRPC::Call(tx, block) => {
                let value = serde_json::to_value(tx).unwrap();
                vec![value, block.into()]
            }
            EthereumRPC::EstimateGas(tx) => {
                let value = serde_json::to_value(tx).unwrap();
                vec![value]
            }
            EthereumRPC::ChainId => vec![],
            EthereumRPC::GasPrice => vec![],
            EthereumRPC::BlockNumber => vec![],
        }
    }
}

impl Target for EthereumRPC {
    fn base_url(&self) -> &'static str {
        "https://rpc.ankr.com"
    }

    fn method(&self) -> HTTPMethod {
        HTTPMethod::POST
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

    fn body(&self) -> HTTPBody {
        let method = self.method_name();
        let params = self.params();
        let req = JsonRpcRequest::new(method, params, 1);
        req.into()
    }
}
