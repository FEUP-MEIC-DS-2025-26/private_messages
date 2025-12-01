# Source: https://kerkour.com/rust-docker-from-scratch
FROM rust:1.90-alpine AS backend
RUN apk update && \
    apk upgrade --no-cache && \
    apk add --no-cache lld mold musl musl-dev libc-dev cmake clang clang-dev openssl file \
    libressl-dev git make build-base bash curl wget zip gnupg coreutils gcc g++ zstd binutils ca-certificates upx \
    protobuf-dev protobuf protobuf-c
    WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs .env ./
COPY src ./src
COPY .sqlx ./.sqlx
COPY proto ./proto
ENV RUSTFLAGS="-C target-cpu=x86-64-v2"
RUN cargo build --release

FROM scratch
WORKDIR /app
# COPY credentials ./credentials
COPY --from=backend /app/target/release/ds-prototype /app/ds-prototype

EXPOSE 8080
ENV RUST_LOG="off"
# CMD ["./ds-prototype", "-p", "8080", "run", "./credentials/password.txt", "./credentials/salt.txt", "-d", "sqlite:.sqlite3"]
CMD ["./ds-prototype", "-p", "8080", "kiosk"]