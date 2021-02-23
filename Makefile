
CURDIR=$(shell pwd)
GIT_REV=$(shell git rev-parse --short HEAD)
WASM_RUNTIME_OUT=bin_archives/nuchain-runtime-$(GIT_REV).compact.wasm

check:
	cargo check --release

test:
	cargo test --release --all

build:
	@@echo Building...
	@@cargo build --release -p nuchain-node

build-wasm-runtime:
	@@echo Building WASM runtime...
	@@cargo build --release -p nuchain-runtime
	@@cp target/release/wbuild/nuchain-runtime/nuchain_runtime.compact.wasm $(WASM_RUNTIME_OUT)
	@@echo Done: $(WASM_RUNTIME_OUT)

.PHONY: check \
	test \
	build \
	build-wasm-runtime


