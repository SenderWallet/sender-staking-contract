FROM rust:1.61.0

LABEL description="Container for builds"

RUN rustup default 1.61.0 && rustup target add wasm32-unknown-unknown

# RUN apt-get update && apt-get install -y git less vim