# reqwest-enum
[![crates.io](https://img.shields.io/crates/v/reqwest-enum.svg)](https://crates.io/crates/reqwest-enum)
[![CI](https://github.com/hewigovens/reqwest-enum/actions/workflows/ci.yml/badge.svg)](https://github.com/hewigovens/reqwest-enum/actions/workflows/ci.yml)

Type-safe and enum style API for Rust, some benefits:

1. It abstracts away repetitive boilerplate code like url formatting, query / header encoding and response deserialization.
2. Type-safe endpoints, readable like a spec, easy to add new or refactor existing endpoints.
3. Async by default and lightweight JSON-RPC support.

Features:

- [x] Type-safe and enum style HTTP API
- [x] JSON-RPC with batching support
- [ ] ...


## Installation

`cargo add reqwest-enum` or add it to your `Cargo.toml`:

```toml
[dependencies]
reqwest-enum = "0.3.2"
```

## Example

### httpbin.org

1. Define endpoints for https://httbin.org as an enum:

```rust
pub enum HttpBin {
    Get,
    Post,
    Bearer,
}
```

2. Implement `Target` for the enum:

```rust
pub trait Target {
    fn base_url(&self) -> &'static str;
    fn method(&self) -> HTTPMethod;
    fn path(&self) -> String;
    fn query(&self) -> HashMap<&'static str, &'static str>;
    fn headers(&self) -> HashMap<&'static str, &'static str>;
    fn authentication(&self) -> Option<AuthMethod>;
    fn body(&self) -> HTTPBody;
}
```

3. Create a provider and request:

```rust
let provider = Provider::<HttpBin>::default();
let response = provider.request(HttpBin::Get).await.unwrap();
assert_eq!(response.status(), 200);
```

Provider also allows you to customize the request by providing a `EndpointFn` or `RequestBuilderFn` closure if default behavior is not sufficient:
1. Need to use different endpoint based on the target.
2. Need to insert custom headers or intercept the final request.

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

## Credits

- [Moya](https://github.com/Moya/Moya)
- [reqwest](https://github.com/seanmonstar/reqwest)
