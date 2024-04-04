extern crate reqwest_enum;
use ethereum_rpc::{BlockParameter, EthereumRPC};
use reqwest_enum::jsonrpc::JsonRpcResult;
use reqwest_enum::provider::{JsonRpcProviderType, Provider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<EthereumRPC>::default();

    let targets = vec![
        EthereumRPC::ChainId,
        EthereumRPC::GasPrice,
        EthereumRPC::BlockNumber,
        EthereumRPC::GetBalance("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
        EthereumRPC::GetCode(
            "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
            BlockParameter::Latest,
        ),
    ];
    let results: Vec<JsonRpcResult<String>> = provider.batch_chunk_by(targets, 2).await?;
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
