FROM rust:1.85-alpine AS chef
USER root

RUN apk add --no-cache musl-dev && cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin load_balancer

FROM debian:bullseye-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/load_balancer /usr/local/bin
ENTRYPOINT ["/usr/local/bin/load_balancer", "-c", "/app/config.yaml", "-a", "0.0.0.0:3000"]