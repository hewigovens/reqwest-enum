list:
    just --list

lint:
    cargo clippy -- -D warnings

test:
    cargo test --all --verbose

publish:
    cargo publish -p reqwest-enum

ethereum-rpc:
    cargo run --package ethereum-rpc

ethereum-rpc-test:
    cargo test --package ethereum-rpc --test integration_test
