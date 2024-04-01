extern crate reqwest_enum;
use ethereum_rpc::EthereumRPC;
use reqwest_enum::jsonrpc::JsonRpcResponse;
use reqwest_enum::provider::{JsonProviderType, Provider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<EthereumRPC>::default();
    let mut response: JsonRpcResponse<String> = provider.request_json(EthereumRPC::ChainId).await?;
    println!("chainId: {}", response.result);

    response = provider.request_json(EthereumRPC::GasPrice).await?;
    println!("gasPrice: {}", response.result);

    response = provider.request_json(EthereumRPC::BlockNumber).await?;
    println!("blockNumber: {}", response.result);
    Ok(())
}
