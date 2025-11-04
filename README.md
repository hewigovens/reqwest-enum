# reqwest-enum
[![crates.io](https://img.shields.io/crates/v/reqwest-enum.svg)](https://crates.io/crates/reqwest-enum)
[![CI](https://github.com/hewigovens/reqwest-enum/actions/workflows/ci.yml/badge.svg)](https://github.com/hewigovens/reqwest-enum/actions/workflows/ci.yml)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/hewigovens/reqwest-enum)

Type-safe and enum style API for Rust, some benefits:

1. It abstracts away repetitive boilerplate code like url formatting, query / header encoding and response deserialization.
2. Type-safe endpoints, readable like a spec, easy to add new or refactor existing endpoints.
3. Async by default and lightweight JSON-RPC support.

Features:

- [x] Type-safe and enum style HTTP API
- [x] JSON-RPC with batching support (default feature)
- [x] Optional middleware support via `reqwest-middleware` (using the `middleware` feature)
- [x] Flexible request customization via closures


## Installation

`cargo add reqwest-enum` or add it to your `Cargo.toml`:

```toml
[dependencies]
reqwest-enum = "0.4.0"
```

## Feature Flags

- `jsonrpc`: (Enabled by default) Provides support for JSON-RPC requests, including batching. Requires `futures`.
- `middleware`: Enables integration with `reqwest-middleware`, allowing you to use custom middleware with your requests. This changes the underlying `RequestBuilder` type used by the `Provider` to `reqwest_middleware::RequestBuilder`.

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
// Simplified trait definition for illustration. Refer to src/target.rs for the full definition.
pub trait Target {
    fn base_url(&self) -> Cow<'_, str>; // Can be dynamic
    fn method(&self) -> HTTPMethod;
    fn path(&self) -> String;
    fn query(&self) -> HashMap<String, String>; // Key/Value for query parameters
    fn headers(&self) -> HashMap<String, String>; // Key/Value for headers
    fn authentication(&self) -> Option<AuthMethod>; // Optional authentication
    fn body(&self) -> Result<HTTPBody, Error>; // Request body, can be fallible
    // Note: Timeout is now handled by the Provider or individual request builders, not directly in Target.
}
```

3. Create a provider and request:

```rust
let provider = Provider::<HttpBin>::default();
let response = provider.request(HttpBin::Get).await.unwrap();
assert_eq!(response.status(), 200);
```

The `Provider` offers powerful customization through closures passed to `Provider::new`:
- `EndpointFn`: `fn(target: &T) -> String`
  - Allows you to dynamically determine the complete request URL based on the `target` enum variant. This overrides the default behavior of combining `base_url()` and `path()`.
- `RequestBuilderFn`: `Box<dyn Fn(&ProviderRequestBuilder, &T) -> ProviderRequestBuilder + Send + Sync>`
  - Provides a way to modify the `ProviderRequestBuilder` after it has been initially constructed by the `Provider` but before the request is sent. This is useful for:
    - Adding or modifying headers.
    - Changing request parameters or body.
    - Any other final adjustments to the request, especially useful when interacting with middleware if the `middleware` feature is enabled.

The `ProviderRequestBuilder` type alias is used internally and in `RequestBuilderFn` to ensure type compatibility whether you are using `reqwest::RequestBuilder` (default) or `reqwest_middleware::RequestBuilder` (with the `middleware` feature).

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
