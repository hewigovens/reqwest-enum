#[cfg(test)]
mod ethereum_rpc_test {
    use ethereum_rpc::{BlockParameter, EthereumRPC};
    use reqwest_enum::jsonrpc::{JsonRpcResponse, JsonRpcResult};
    use reqwest_enum::provider::{JsonProviderType, Provider};

    const TEST_ADDRESS: &str = "0xee5f5c53ce2159fc6dd4b0571e86a4a390d04846";

    #[tokio::test]
    async fn test_chain_id() {
        let provider = Provider::<EthereumRPC>::default();
        let response: JsonRpcResponse<String> =
            provider.request_json(EthereumRPC::ChainId).await.unwrap();
        assert_eq!(response.result, "0x1");
    }

    #[tokio::test]
    async fn test_gas_price() {
        let provider = Provider::<EthereumRPC>::default();
        let response: JsonRpcResponse<String> =
            provider.request_json(EthereumRPC::GasPrice).await.unwrap();
        assert_ne!(response.result, "0x0");
    }

    #[tokio::test]
    async fn test_block_number() {
        let provider = Provider::<EthereumRPC>::default();
        let response: JsonRpcResponse<String> = provider
            .request_json(EthereumRPC::BlockNumber)
            .await
            .unwrap();
        assert_ne!(response.result, "0x0");
    }

    #[tokio::test]
    async fn test_get_balance() {
        let provider = Provider::<EthereumRPC>::default();

        let response: JsonRpcResponse<String> = provider
            .request_json(EthereumRPC::GetBalance(TEST_ADDRESS))
            .await
            .unwrap();
        assert_ne!(response.result, "0x0");
    }

    #[tokio::test]
    async fn test_get_transaction_count() {
        let provider = Provider::<EthereumRPC>::default();
        let response: JsonRpcResponse<String> = provider
            .request_json(EthereumRPC::GetTransactionCount(
                TEST_ADDRESS,
                BlockParameter::Latest,
            ))
            .await
            .unwrap();
        assert_eq!(response.result, "0x2");
    }

    #[tokio::test]
    async fn test_syncing() {
        let provider = Provider::<EthereumRPC>::default();
        let response: JsonRpcResult<bool> =
            provider.request_json(EthereumRPC::Syncing).await.unwrap();

        assert!(matches!(response, JsonRpcResult::Value(JsonRpcResponse { id: _, jsonrpc: _, result: _ })));
    }

    #[tokio::test]
    async fn test_blob_base_fee() {
        let provider = Provider::<EthereumRPC>::default();
        let result: JsonRpcResult<String> = provider
            .request_json(EthereumRPC::BlobBaseFee)
            .await
            .expect("request error");
        
        match result {
            JsonRpcResult::Value(JsonRpcResponse { id: _, jsonrpc: _, result }) => {
                let base_fee = u128::from_str_radix(&result.replace("0x", ""), 16).unwrap();

                assert!(base_fee > 0);
            }
            _ => {
                panic!("expected JsonRpcResult::Value, got: {:?}", result);
            }
        }
    }
}
