
CURDIR=$(shell pwd)

check:
	cargo check --release

test:
	cargo test --release --all

build:
	@@echo Building...
	@@cargo build --release -p nuchain-node

docker_build:
	@@echo Building Linux binary through Docker...
	@@mkdir -p $(CURDIR)/target/docker_build
	docker run --rm -v $(CURDIR):/builds \
		-v $(CURDIR)/target/docker_build:/builds/target/release \
		-v ${HOME}/.cargo:/root/.cargo \
		--name build_nuchain -t mysubstrate:v1 make build
	@@cp target/docker_build/nuchain ./
	@@docker build -t nuchain_runner:latest .

.PHONY: check \
	test \
	build \
	docker_build \
	install

