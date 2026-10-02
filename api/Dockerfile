# Build context: the repository root.
FROM rust:1.98.1-alpine3.24@sha256:7cc1c22d77d9432f7fe012a70e6d3e555af54c2a6832700ed7d553f1769ae89f AS chef
RUN cargo install cargo-chef --locked --version 0.1.78
WORKDIR /usr/local/src/cinelist

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/local/src/cinelist/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json
COPY . .
RUN cargo build --release --locked --package cinelist-api \
 && addgroup -S -g 65532 cinelist \
 && adduser -S -D -H -u 65532 -G cinelist -h /nonexistent -s /sbin/nologin cinelist

FROM scratch
COPY --from=builder /etc/passwd /etc/group /etc/
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /usr/local/src/cinelist/target/release/cinelist-api /usr/local/bin/cinelist-api
USER 65532:65532
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/cinelist-api"]
