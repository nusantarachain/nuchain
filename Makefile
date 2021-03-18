
CURDIR=$(shell pwd)
NODE_VERSION=$(shell grep 'version = ' bin/node/cli/Cargo.toml | head -1 | cut -d '"' -f2)
GIT_REV=$(shell git rev-parse --short HEAD)
OS:=$(shell uname | sed -e 's/\(.*\)/\L\1/')
BIN_NAME=nuchain-$(NODE_VERSION)-$(GIT_REV)-$(OS)
RUNTIME_SPEC_VER=$(shell grep -Po 'spec_version: \d+' bin/node/runtime/src/lib.rs | grep -Po '\d+')
WASM_RUNTIME_OUT=nuchain-runtime-spec_ver_$(RUNTIME_SPEC_VER)-git_$(GIT_REV).compact.wasm


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
	cp target/release/wbuild/nuchain-runtime/nuchain_runtime.compact.wasm bin_archives/$(WASM_RUNTIME_OUT)

package:
	@@echo Packaging...
	make build-wasm-runtime
	@@cd target/release/wbuild/nuchain-runtime && \
		zip ../../../../bin_archives/$(WASM_RUNTIME_OUT).zip nuchain_runtime.compact.wasm
	@@echo Done: bin_archives/$(WASM_RUNTIME_OUT).zip
	make build
	@@cd target/release && \
		zip ../../bin_archives/$(BIN_NAME).zip nuchain
	@@echo Done.
	@@echo Runtime: bin_archives/$(WASM_RUNTIME_OUT).zip
	@@echo Exe Bin: bin_archives/$(BIN_NAME).zip


.PHONY: check \
	test \
	build \
	build-wasm-runtime \
	package


