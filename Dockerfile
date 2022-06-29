FROM rust:1.61.0 AS build

WORKDIR /tmp/
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build
COPY ./src ./src
RUN cargo build --offline


FROM ubuntu:20.04

RUN apt update && apt install -yy openssl ca-certificates

USER nobody
COPY --from=build /tmp/target/debug/near-enhanced-api /
ENTRYPOINT ["/near-enhanced-api"]
