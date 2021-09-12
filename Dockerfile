FROM rust:1.55.0-bullseye as BUILDER
RUN apt update && apt install -y \
    musl-tools \
    pkgconf
    
COPY ./src /usr/src/plugin_stat_server/src/
COPY ./Cargo.toml /usr/src/plugin_stat_server/
WORKDIR /usr/src/plugin_stat_server
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest
COPY --from=BUILDER /usr/src/plugin_stat_server/target/x86_64-unknown-linux-musl/release/plugin_stat_server /usr/local/bin/plugin_stat_server

EXPOSE 8080
ENTRYPOINT [ "/usr/local/bin/plugin_stat_server" ]