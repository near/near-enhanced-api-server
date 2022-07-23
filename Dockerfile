FROM rust:1.61.0 AS build

# We have to use sparse-registry nightly cargo feature to avoid running out of RAM:
# https://github.com/rust-lang/cargo/issues/10781
RUN rustup toolchain install nightly

WORKDIR /tmp/
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo +nightly build -Z sparse-registry --release && rm -r src
COPY ./src ./src
RUN cargo +nightly build -Z sparse-registry --offline --release


FROM ubuntu:20.04

RUN apt update && apt install -yy openssl ca-certificates

USER nobody
COPY --from=build /tmp/target/release/near-enhanced-api /
ENTRYPOINT ["/near-enhanced-api"]
