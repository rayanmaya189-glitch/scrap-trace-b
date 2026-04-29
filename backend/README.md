# B-Trace Backend

Industrial Traceability & Credit Protocol - Rust/Axum API Server

## Quick Start

### Prerequisites
- Rust 1.75+
- Docker & Docker Compose (for dependencies)

### Development Setup

1. **Start Infrastructure Dependencies**
```bash
docker-compose up -d
```

2. **Copy Environment Configuration**
```bash
cp .env.example .env
```

3. **Run Database Migrations**
```bash
sqlx migrate run --database-url postgres://postgres:postgres@localhost:5432/btrace
```

4. **Start the Server**
```bash
cargo run
```

The API will be available at `http://localhost:3000`

## Architecture

B-Trace follows a strict **NATS JetStream-only-write architecture**:

```
┌─────────────┐    ┌─────────────┐    ┌──────────────────┐    ┌──────────────┐
│  PWA/Edge   │───▶│  Axum API   │───▶│ NATS JetStream   │───▶│ PG Consumer  │
│ (Offline)   │    │ (Auth/Rate) │    │ (Persistent Bus) │    │ (Rust/sqlx)  │
└─────────────┘    └─────────────┘    └──────────────────┘    └──────────────┘
```

**Key Principle**: Only NATS JetStream events write to PostgreSQL. The Axum API layer publishes events; consumers process and persist them. Zero direct DB writes from the API layer.

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Health check |
| `POST` | `/v1/auth/request` | Request OTP |
| `POST` | `/v1/auth/verify` | Verify OTP & get JWT |
| `POST` | `/v1/material` | Create material passport |
| `POST` | `/v1/handshake/confirm` | Confirm digital handshake |
| `GET` | `/v1/score/:supplier_id` | Get credit score |
| `GET` | `/v1/export/:plugin/:id.:format` | Generate compliance export |

## Project Structure

```
backend/
├── src/
│   ├── main.rs          # Application entry point
│   ├── config.rs        # Configuration management
│   ├── crypto.rs        # Cryptographic utilities
│   ├── error.rs         # Error types and handling
│   ├── models.rs        # Data models and DTOs
│   ├── nats.rs          # NATS JetStream integration
│   ├── state.rs         # Application state
│   └── routes/          # HTTP route handlers
│       ├── mod.rs
│       ├── health.rs
│       ├── auth.rs
│       ├── material.rs
│       ├── handshake.rs
│       ├── score.rs
│       └── export.rs
├── migrations/          # SQLx database migrations
├── Cargo.toml
└── .env.example
```

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```

## Building for Production

```bash
cargo build --release
```

The binary will be optimized with LTO and stripped for minimal size.

## License

MIT
