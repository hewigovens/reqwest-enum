extern crate reqwest_enum;
mod ethereum_rpc;
use ethereum_rpc::EthereumRPC;
use reqwest_enum::jsonrpc::JsonRpcResponse;
use reqwest_enum::provider::{JsonProviderType, Provider, ProviderType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<EthereumRPC>::default();
    let mut response: JsonRpcResponse<String> = provider.request_json(EthereumRPC::ChainId).await?;
    println!("chainId: {}", response.result);

    response = provider.request_json(EthereumRPC::GasPrice).await?;
    println!("gasPrice: {}", response.result);

    response = provider.request_json(EthereumRPC::BlockNumber).await?;
    println!("blockNumber: {}", response.result);

    response = provider
        .request(EthereumRPC::GetBalance(
            "0xee5f5c53ce2159fc6dd4b0571e86a4a390d04846",
        ))
        .await?
        .json()
        .await?;

    println!("balance: {}", response.result);
    Ok(())
}
