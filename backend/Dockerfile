# Build stage
FROM rust:1.75-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Create dummy src for dependency caching
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Fetch dependencies (cached layer)
RUN cargo fetch --locked 2>/dev/null || cargo fetch

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM alpine:3.19

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc

# Copy binary from builder
COPY --from=builder /app/target/release/btrace-backend .

# Copy migrations
COPY migrations ./migrations

# Create non-root user
RUN addgroup -g 1000 btrace && adduser -D -u 1000 -G btrace btrace
USER btrace

EXPOSE 3000

CMD ["./btrace-backend"]
