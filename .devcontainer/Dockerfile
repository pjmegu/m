FROM debian:trixie-slim

RUN apt-get update && apt-get install -y curl git tokei gcc xz-utils

# install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y -t wasm32-unknown-unknown

# install rust tools
RUN ~/.cargo/bin/cargo install wasm-tools

RUN curl https://wasmtime.dev/install.sh -sSf | bash