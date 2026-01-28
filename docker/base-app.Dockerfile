# Build stage
FROM rust:1.75-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy workspace manifests
COPY Cargo.toml Cargo.lock* ./
COPY libs/common/Cargo.toml ./libs/common/
COPY services/base-app/Cargo.toml ./services/base-app/
COPY services/echo-service/Cargo.toml ./services/echo-service/

# Create dummy sources to cache dependencies
RUN mkdir -p libs/common/src services/base-app/src services/echo-service/src && \
    echo "pub fn dummy() {}" > libs/common/src/lib.rs && \
    echo "fn main() {}" > services/base-app/src/main.rs && \
    echo "fn main() {}" > services/echo-service/src/main.rs

RUN cargo build --release --package base-app

# Copy actual source code (all workspace members needed for rebuild)
COPY libs/common/src ./libs/common/src
COPY services/base-app/src ./services/base-app/src
COPY services/echo-service/src ./services/echo-service/src

# Build for release (touch common to force recompilation)
RUN touch libs/common/src/lib.rs services/base-app/src/main.rs && \
    cargo build --release --package base-app

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/base-app /app/base-app

EXPOSE 8080

CMD ["/app/base-app"]
