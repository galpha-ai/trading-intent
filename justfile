default:
    @just --list

build:
    cargo build

test:
    cargo test

test-one TEST:
    cargo test {{TEST}} -- --nocapture

fmt:
    cargo fmt

lint:
    cargo clippy -- -D warnings

ci: fmt lint test
    @echo "CI passed"

serve:
    TIM_CONFIG_PATH=config/local.yaml cargo run --bin tim_server

doc:
    cargo doc --open
