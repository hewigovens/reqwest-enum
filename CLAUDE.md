# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# reqwest-enum Development Guide

## Commands
- Build: `cargo build`
- Lint: `just lint` or `cargo clippy -- -D warnings`
- Run all tests: `just test` or `cargo test --all --verbose`
- Run specific test: `cargo test --package <package> <test_name>`
- Run Ethereum RPC example: `just ethereum-rpc`
- Run Ethereum RPC tests: `just ethereum-rpc-test`

## Code Style
- Use 2021 edition Rust with static typing for all APIs
- Follow enum-based API design where targets are defined as enum variants
- Implement `Target` trait for HTTP endpoints, `JsonRpcTarget` for RPC methods
- Prefer static strings (`&'static str`) for constant values
- Use descriptive enum variants that match API endpoints
- Use proper error handling with Result types
- Keep implementation simple and focused on the target trait implementation
- Use workspace dependencies for version consistency
- Document public APIs with comments
- Format code with `cargo fmt`
- Pass clippy checks with no warnings

## Architecture Overview

### Core Trait System
The library is built around the `Target` trait pattern:
- **Target trait**: Defines HTTP endpoints with methods for URL, headers, body, auth
- **JsonRpcTarget trait**: Extends Target for JSON-RPC 2.0 method calls
- **Provider<T>**: Centralizes request execution with configurable behavior

### Key Components
- `reqwest-enum/src/target.rs`: Core trait definitions
- `reqwest-enum/src/provider.rs`: Request execution and batching logic
- `reqwest-enum/src/jsonrpc.rs`: JSON-RPC 2.0 implementation with batching
- `reqwest-enum/src/http.rs`: HTTP types and authentication methods
- `examples/ethereum-rpc/`: Real-world usage example

### Enum-Based API Design
HTTP endpoints are defined as enum variants that implement Target:
```rust
pub enum MyAPI {
    GetUser(String),
    CreatePost { title: String, body: String },
}

impl Target for MyAPI {
    fn path(&self) -> String {
        match self {
            Self::GetUser(id) => format!("/users/{}", id),
            Self::CreatePost { .. } => "/posts".to_string(),
        }
    }
    // ... other methods
}
```

### Provider Pattern
Use Provider for request execution:
- Configure base URL and request customization via closures
- Support for authentication, timeouts, and middleware
- JSON-RPC batching with chunking support

### Feature Flags
- `jsonrpc` (default): Enables JSON-RPC 2.0 support
- `middleware`: Enables reqwest-middleware integration

## Testing
- Main library tests in `reqwest-enum/src/`
- Example integration tests in `examples/ethereum-rpc/tests/`
- Use `cargo test --all --all-features` for comprehensive testing