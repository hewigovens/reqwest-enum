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