FROM rust:latest as BUILDER
COPY . /usr/src/stat_server
WORKDIR /usr/src/stat_server
RUN cargo install --path .


FROM alpine:latest

ENV GLIBC_REPO=https://github.com/sgerrand/alpine-pkg-glibc
ENV GLIBC_VERSION=2.30-r0

RUN set -ex && \
    apk --update add libstdc++ curl ca-certificates && \
    for pkg in glibc-${GLIBC_VERSION} glibc-bin-${GLIBC_VERSION}; \
        do curl -sSL ${GLIBC_REPO}/releases/download/${GLIBC_VERSION}/${pkg}.apk -o /tmp/${pkg}.apk; done && \
    apk add --allow-untrusted /tmp/*.apk && \
    rm -v /tmp/*.apk && \
    /usr/glibc-compat/sbin/ldconfig /lib /usr/glibc-compat/lib

COPY --from=BUILDER /usr/local/cargo/bin/stat_server /usr/local/bin/stat_server
EXPOSE 8080

CMD ["sh", "-c", "/usr/local/bin/stat_server"]