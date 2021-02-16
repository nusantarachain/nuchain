
CURDIR=$(shell pwd)

check:
	cargo check --release

test:
	cargo test --release --all

build:
	@@echo Building...
	@@cargo build --release -p nuchain-node


.PHONY: check \
	test \
	build

