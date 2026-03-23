# Contributing to reqwest-enum

## Requirements

- Rust stable toolchain (2024 edition)
- `just` (optional)

## Development loop

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test --all --verbose
```

Or use `just`:

```bash
just lint
just test
```

## Running examples

```bash
just ethereum-rpc          # Run Ethereum RPC example
just ethereum-rpc-test     # Run Ethereum RPC tests
```
