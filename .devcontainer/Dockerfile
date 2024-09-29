FROM debian:trixie-slim AS build

RUN apt-get update && apt-get install -y curl git gcc xz-utils

# install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y -t wasm32-unknown-unknown

FROM build AS dev

# install tools
RUN apt-get update && apt-get install -y tokei unzip

# install deno
RUN curl -fsSL https://deno.land/install.sh | sh
ENV PATH=/root/.deno/bin:$PATH

# install rust tools
RUN ~/.cargo/bin/cargo install wasm-tools
