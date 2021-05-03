# --------- STAGE 1 -------------------------------------------------------

FROM nuchain-build:develop as build

WORKDIR /src

# unneeded this lines, already frozen in nuchain-build:develop image
# RUN apt-get update && apt-get install git -y && apt install -y cmake pkg-config libssl-dev git build-essential clang libclang-dev curl libz-dev
# RUN curl https://sh.rustup.rs -sSf | bash -s -- -y --default-toolchain nightly-2021-01-29
# RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2021-01-29 && cargo install cargo-deb

COPY . /src/

RUN . ~/.cargo/env && cargo build --release -p nuchain-node
RUN strip target/release/nuchain
RUN . ~/.cargo/env && make deb

# --------- STAGE 2 -------------------------------------------------------

FROM debian:stretch-slim

WORKDIR /nuchain

COPY --from=build /src/target/release/nuchain /usr/bin/
COPY --from=build /src/target/debian /nuchain/

EXPOSE 9944/tcp
EXPOSE 30333/tcp

CMD nuchain --validator --base-path=/data

