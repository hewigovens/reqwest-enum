extern crate reqwest_enum;
use ethereum_rpc::EthereumRPC;
use reqwest_enum::jsonrpc::JsonRpcResult;
use reqwest_enum::provider::{JsonRpcProviderType, Provider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<EthereumRPC>::default();

    let targets = vec![EthereumRPC::ChainId, EthereumRPC::GasPrice];
    let results: Vec<JsonRpcResult<String>> = provider.batch(targets).await?;
    for result in results {
        match result {
            JsonRpcResult::Value(response) => {
                println!("{}", response.result);
            }
            JsonRpcResult::Error(error) => {
                println!("{}", error.error);
            }
        }
    }
    Ok(())
}
