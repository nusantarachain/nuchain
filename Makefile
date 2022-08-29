
CURDIR=$(shell pwd)
NODE_VERSION=$(shell grep 'version = ' bin/nuchain-node/cli/Cargo.toml | head -1 | cut -d '"' -f2)
GIT_REV=$(shell git rev-parse --short HEAD)
OS:=$(shell uname | sed -e 's/\(.*\)/\L\1/')
BIN_NAME=nuchain-$(NODE_VERSION)-$(GIT_REV)-$(OS)
WASM_RUNTIME_OUT=nuchain-runtime-$(GIT_REV).compact.wasm
DISTRO=$(cat /etc/os-release | grep '^VERSION_ID=' | cut -d '"' -f 2)
RUNTIME_SPEC_VER=$(shell grep -o 'spec_version: [0-9]\+' bin/nuchain-node/runtime/src/lib.rs | grep -o '[0-9]\+')

check:
	cargo check --release

test:
	cargo test --release --all

build:
	@@echo Building...
	@@cargo build --release -p nuchain-node

build-debug:
	@@echo "Building (debug mode)..."
	@@cargo build -p nuchain-node

build-wasm-runtime:
	@@echo Building WASM runtime...
	@@cargo build --release -p nuchain-runtime
	@@cd target/release/wbuild/nuchain-runtime && \
		cp nuchain_runtime.compact.wasm nuchain_runtime-$(RUNTIME_SPEC_VER).compact.wasm
	@@echo runtime build: nuchain_runtime-$(RUNTIME_SPEC_VER).compact.wasm

build-benchmark:
	@@echo Building binary for benchmark...
	cargo build -p nuchain-node --release --features="runtime-benchmarks"

deb:
	@@echo Packaging for $(DISTRO)
	@@cargo deb -p nuchain-node
	cp target/debian/nuchain_$(NODE_VERSION)_amd64.deb bin_archives/nuchain-$(NODE_VERSION)-$(GIT_REV)-$(DISTRO)_amd64.deb

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
	build-benchmark \
	package \
	deb


