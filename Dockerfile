FROM rust:1.62 AS build

WORKDIR /tmp/
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -r src
COPY ./src ./src
RUN touch src/main.rs && cargo build --offline --release


FROM ubuntu:20.04

RUN apt update && apt install -yy openssl ca-certificates

USER nobody
COPY --from=build /tmp/target/release/near-enhanced-api /
ENTRYPOINT ["/near-enhanced-api"]
