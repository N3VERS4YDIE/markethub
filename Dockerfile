# Build stage
FROM rust:1.91-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev postgresql-dev

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source
COPY src ./src
COPY migrations ./migrations

# Build release binary
RUN cargo build --release

# Runtime stage
FROM alpine:latest

RUN apk add --no-cache libgcc libpq

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/markethub /usr/local/bin/markethub

# Copy migrations
COPY --from=builder /app/migrations ./migrations

EXPOSE 8000

CMD ["markethub"]
