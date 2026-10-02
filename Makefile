.PHONY: fmt fmt-check clippy test check build

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --locked -- -D warnings

test:
	cargo test --locked

check: fmt-check clippy test

build:
	cargo build --locked --release
