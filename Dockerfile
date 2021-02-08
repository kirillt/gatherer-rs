FROM rustlang/rust:nightly-slim AS builder
RUN apt-get update && apt-get install -y git pkg-config openssl libssl-dev

RUN mkdir /build
COPY ./Cargo.* /build/
COPY ./src /build/src
COPY ./test /build/test

RUN cargo install --path /build

FROM debian:buster-slim
RUN apt-get update && apt-get install -y openssl

COPY --from=builder /usr/local/cargo/bin/gather /bin/gather

RUN mkdir /etc/certs
COPY keystore.pkcs12 /etc/certs/

ENTRYPOINT ["/bin/gather", "-t", "/etc/certs/keystore.pkcs12"]
