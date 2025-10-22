FROM node:20-alpine AS frontend
WORKDIR /app/frontend
COPY frontend/next.config.ts frontend/package.json frontend/package-lock.json frontend/postcss.config.mjs frontend/tsconfig.json ./
COPY frontend/app ./app
COPY frontend/public ./public
ENV NEXT_TELEMETRY_DISABLED="1"
RUN npm install
RUN npm run build

FROM rust:1.90-slim AS backend
WORKDIR /app
COPY --from=frontend /app/frontend/out /app/frontend/out
COPY Cargo.toml Cargo.lock ./
COPY src ./src
ENV RUSTFLAGS="-C target-cpu=native"
RUN cargo build --release

EXPOSE 8080

CMD ["./target/release/ds-prototype"]
