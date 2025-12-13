# Source: https://kerkour.com/rust-docker-from-scratch
FROM rust:1.90-alpine AS backend
# Note: To pass the secrets run with --secret id=JUMPSELLER --secret id=PUBSUB
WORKDIR /app
RUN mkdir ./local
RUN --mount=type=secret,id=JUMPSELLER \
    --mount=type=secret,id=PUBSUB \
    --mount=type=secret,id=PASSWORD \
    --mount=type=secret,id=SALT \
    cat /run/secrets/JUMPSELLER > /app/local/jumpseller_cred.json && \
    cat /run/secrets/PUBSUB > /app/local/pubsub.json && \
    cat /run/secrets/PASSWORD > /app/local/password.txt && \
    cat /run/secrets/SALT > /app/local/salt.txt

RUN apk update && \
    apk upgrade --no-cache && \
    apk add --no-cache lld mold musl musl-dev libc-dev cmake clang clang-dev openssl file \
    libressl-dev git make build-base bash curl wget zip gnupg coreutils gcc g++ zstd binutils ca-certificates upx \
    protobuf-dev protobuf protobuf-c
COPY Cargo.toml Cargo.lock build.rs .env ./
COPY src ./src
COPY .sqlx ./.sqlx
COPY proto ./proto
ENV RUSTFLAGS="-C target-cpu=x86-64-v2"
RUN cargo build --release


FROM scratch
WORKDIR /app

COPY --from=backend /app/target/release/ds-prototype /app/ds-prototype
COPY --from=backend /app/local /app/local

EXPOSE 8080
ENV RUST_LOG="off"
ENV GOOGLE_APPLICATION_CREDENTIALS="./local/pubsub.json"
CMD ["./ds-prototype", "-p", "8080", "run", "/app/local/password.txt", "/app/local/salt.txt", "-d", "sqlite:/mnt/database/.sqlite3"]
# CMD ["./ds-prototype", "-p", "8080", "kiosk"]
