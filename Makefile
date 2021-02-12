

check:
	cargo check --release

test:
	cargo test --release --all

build:
	cargo build --release -p node-cli

.PHONY: check test build

