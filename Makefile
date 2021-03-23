
CURDIR=$(shell pwd)
NODE_VERSION=$(shell grep 'version = ' bin/node/cli/Cargo.toml | head -1 | cut -d '"' -f2)
GIT_REV=$(shell git rev-parse --short HEAD)
OS:=$(shell uname | sed -e 's/\(.*\)/\L\1/')
BIN_NAME=nuchain-$(NODE_VERSION)-$(GIT_REV)-$(OS)
WASM_RUNTIME_OUT=nuchain-runtime-$(GIT_REV).compact.wasm


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

deb:
	@@echo Packaging for Debian
	@@cargo deb -p nuchain-node
	cp target/debian/nuchain_$(NODE_VERSION)_amd64.deb bin_archives/nuchain-$(NODE_VERSION)-$(GIT_REV)_amd64.deb

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
	package \
	deb


