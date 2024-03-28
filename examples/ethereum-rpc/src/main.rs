extern crate reqwest_enum;
mod ethereum_rpc;
use ethereum_rpc::EthereumRPC;
use reqwest_enum::jsonrpc::JsonRpcResponse;
use reqwest_enum::provider::{Provider, ProviderType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<EthereumRPC>::default();
    let mut json_resp: JsonRpcResponse<String> =
        provider.request(EthereumRPC::ChainId).await?.json().await?;
    println!("chainId: {}", json_resp.result);

    json_resp = provider
        .request(EthereumRPC::GasPrice)
        .await?
        .json()
        .await?;
    println!("gasPrice: {}", json_resp.result);

    json_resp = provider
        .request(EthereumRPC::BlockNumber)
        .await?
        .json()
        .await?;
    println!("blockNumber: {}", json_resp.result);

    json_resp = provider
        .request(EthereumRPC::GetBalance(
            "0xee5f5c53ce2159fc6dd4b0571e86a4a390d04846",
        ))
        .await?
        .json()
        .await?;

    println!("balance: {}", json_resp.result);
    Ok(())
}
