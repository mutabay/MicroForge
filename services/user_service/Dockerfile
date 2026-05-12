FROM rust:1-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY orchestrator/ orchestrator/
COPY services/ services/
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo build --release --bin user_service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/user_service /usr/local/bin/user_service
RUN mkdir -p /data
EXPOSE 8001
ENV DATABASE_URL=sqlite:///data/users.db?mode=rwc
CMD ["user_service"]
