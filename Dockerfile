ARG SERVICE_NAME

FROM rust:slim-bookworm AS builder
ARG SERVICE_NAME
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy entire workspace
COPY Cargo.toml Cargo.lock* ./
COPY shared/ shared/
COPY services/ services/

# Build only the target service
RUN cargo build --release -p ${SERVICE_NAME}

FROM debian:bookworm-slim
ARG SERVICE_NAME
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/${SERVICE_NAME} ./service

# Copy migrations if they exist (auth, user, appointment services)
COPY services/${SERVICE_NAME}/migrations* ./migrations/

CMD ["./service"]
