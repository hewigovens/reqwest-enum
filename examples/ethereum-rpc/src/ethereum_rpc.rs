use reqwest_enum::http::HTTPBody;
use reqwest_enum::jsonrpc::JsonRpcRequest;
use reqwest_enum::{
    http::{AuthMethod, HTTPMethod},
    target::{JsonRpcTarget, Target},
};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::{collections::HashMap, borrow::Cow};

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
    // hexadecimal block number
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

fn u64_to_value(val: &u64) -> serde_json::Value {
    Value::Number(Number::from(*val))
}

pub enum EthereumRPC {
    BlockNumber,
    BlobBaseFee,
    ChainId,
    Call(TransactionObject, BlockParameter),
    EstimateGas(TransactionObject),
    // blockCount, newestBlock, rewardPercentiles
    FeeHistory(u64, BlockParameter, Vec<u64>),
    GasPrice,
    // addrees
    GetBalance(&'static str),
    GetBlockByNumber(BlockParameter, bool),
    GetCode(&'static str, BlockParameter),
    // address, blockNumber
    GetTransactionCount(&'static str, BlockParameter),
    SendRawTransaction(&'static str),
    Syncing,
    // filter id
    UninstallFilter(&'static str),
}

impl JsonRpcTarget for EthereumRPC {
    fn method_name(&self) -> &'static str {
        match self {
            EthereumRPC::Syncing => "eth_syncing",
            EthereumRPC::ChainId => "eth_chainId",
            EthereumRPC::GasPrice => "eth_gasPrice",
            EthereumRPC::BlockNumber => "eth_blockNumber",
            EthereumRPC::GetBalance(_) => "eth_getBalance",
            EthereumRPC::SendRawTransaction(_) => "eth_sendRawTransaction",
            EthereumRPC::GetBlockByNumber(_, _) => "eth_getBlockByNumber",
            EthereumRPC::GetTransactionCount(_, _) => "eth_getTransactionCount",
            EthereumRPC::Call(_, _) => "eth_call",
            EthereumRPC::EstimateGas(_) => "eth_estimateGas",
            EthereumRPC::FeeHistory(_, _, _) => "eth_feeHistory",
            EthereumRPC::GetCode(_, _) => "eth_getCode",
            EthereumRPC::BlobBaseFee => "eth_blobBaseFee",
            EthereumRPC::UninstallFilter(_) => "eth_uninstallFilter",
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
                vec![block.into(), Value::Bool(full.to_owned())]
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
            EthereumRPC::FeeHistory(block_count, block, reward_percentiles) => {
                let mut params = vec![
                    u64_to_value(block_count),
                    block.into(),
                    Value::Array(reward_percentiles.iter().map(u64_to_value).collect()),
                ];
                params.push(Value::Bool(false));
                params
            }
            EthereumRPC::GetCode(address, block) => {
                vec![Value::String(address.to_string()), block.into()]
            }
            EthereumRPC::UninstallFilter(filter_id) => {
                vec![Value::String(filter_id.to_string())]
            }
            EthereumRPC::ChainId
            | EthereumRPC::GasPrice
            | EthereumRPC::BlockNumber
            | EthereumRPC::Syncing
            | EthereumRPC::BlobBaseFee => vec![],
        }
    }
}

impl Target for EthereumRPC {
    fn base_url(&self) -> Cow<'_, str> {
        Cow::Borrowed("https://ethereum-rpc.publicnode.com")
    }

    fn method(&self) -> HTTPMethod {
        HTTPMethod::POST
    }

    fn path(&self) -> String {
        "/".into()
    }

    fn query(&self) -> HashMap<String, String> {
        HashMap::default()
    }

    fn headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }

    fn authentication(&self) -> Option<AuthMethod> {
        None
    }

    fn body(&self) -> Result<HTTPBody, reqwest_enum::Error> {
        let method = self.method_name();
        let params = self.params();
        let req = JsonRpcRequest::new(method, params, 1);
        Ok(HTTPBody::from(&req)?)
    }

}
