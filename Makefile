
CURDIR=$(shell pwd)

check:
	cargo check --release

test:
	cargo test --release --all

target/release/nuchain:
	@@echo Building...
	@@cargo build --release -p nuchain-node

build: target/release/nuchain

docker_build:
	@@echo Building Linux binary through Docker...
	@@mkdir -p $(CURDIR)/target/docker_build
	docker run --rm -v $(CURDIR):/builds \
		-v $(CURDIR)/target/docker_build:/builds/target/release \
		-v ${HOME}/.cargo:/root/.cargo \
		--name build_nuchain -t mysubstrate:v1 make build
	@@cp target/docker_build/nuchain ./
	@@docker build -t nuchain_runner:latest .

install: target/release/nuchain
	@@echo Installing...
	@@cp $< /usr/local/bin/nuchain

.PHONY: check \
	test \
	build \
	docker_build \
	install

