list:
    just --list

fix:
    cargo clippy --fix --workspace --all-targets --allow-dirty

lint:
    cargo clippy -- -D warnings

test:
    cargo test --all --all-features

publish:
    cargo publish -p reqwest-enum

ethereum-rpc:
    cargo run --package ethereum-rpc

ethereum-rpc-test:
    cargo test --package ethereum-rpc --test integration_test
