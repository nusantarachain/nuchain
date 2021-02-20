

check:
	cargo check --release

test:
	cargo test --release --all

build:
	cargo build --release -p nuchain-node

.PHONY: check test build

