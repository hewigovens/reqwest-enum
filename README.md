# reqwest-enum
[![CI](https://github.com/hewigovens/reqwest-enum/actions/workflows/ci.yml/badge.svg)](https://github.com/hewigovens/reqwest-enum/actions/workflows/ci.yml)

Typed enum HTTP API for [reqwest](https://github.com/seanmonstar/reqwest), inspired by [Moya](https://github.com/Moya/Moya).

Features:

- [x] Typed enum HTTP API
- [x] JSON-RPC
- [ ] ...


## Installation

`cargo add reqwest-enum` or add it to your `Cargo.toml`:

```toml
[dependencies]
reqwest-enum = "0.1.0"
```

## Example

### httpbin.org

1. Define endpoints for https://httbin.org as an enum:

```rust
pub enum HttpBin {
    Get,
    Post,
}
```

2. Implement `Target` for the enum:

```rust
pub trait Target {
    fn base_url(&self) -> &'static str;
    fn method(&self) -> HTTPMethod;
    fn path(&self) -> &'static str;
    fn query(&self) -> HashMap<&'static str, &'static str>;
    fn headers(&self) -> HashMap<&'static str, &'static str>;
    fn body(&self) -> HTTPBody;
}
```

3. Create a provider and request:

```rust
let provider = Provider::<HttpBin>::default();
let response = provider.request(HttpBin::Get).await.unwrap();
assert_eq!(response.status(), 200);
```

### JSON-RPC

Full example can be found in [examples/ethereum-rpc](examples/ethereum-rpc).

1. Define Ethereum JSON-RPC methods as an enum:

```rust
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
```

2. Implement `Target` and `JsonRpcTarger` for the enum:

```rust
pub trait JsonRpcTarget: Target {
    fn method_name(&self) -> &'static str;
    fn params(&self) -> Vec<Value>;
}
```

3. Create a provider and request:

```rust
let provider = Provider::<EthereumRPC>::default();
let response: JsonRpcResponse<String> =
    provider.request_json(EthereumRPC::ChainId).await.unwrap();
assert_eq!(response.result, "0x1");
```

## License

[MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE)
