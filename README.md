# MarketHub 🏪

A production-ready, multi-tenant e-commerce platform with role-based access control and private storefronts.

## 🚀 Quick Start

### Prerequisites

- Rust
- PostgreSQL
- Docker & Docker Compose

### Local Development

```bash
# Clone and setup
git clone <repo-url>
cd final-project

# Copy environment variables
cp .env.example .env

# Start database
docker compose up -d postgres

# Run migrations
sqlx migrate run

# Run the server
cargo run

# API available at http://localhost:8000
```

### Run Tests

```bash
cargo test
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Security audit
cargo audit
```

## 📚 Documentation

See [PROJECT.md](./PROJECT.md) for complete architecture and development guidelines.

## 🏗️ Architecture

```
Handlers → Services → Repositories → Database
    ↓
  Models (Domain entities with validation)
```

## 🔐 Key Features

- ✅ Multi-vendor marketplace
- ✅ Public & private stores
- ✅ Role-based access control (RBAC)
- ✅ Multi-store shopping cart
- ✅ Invitation-only store access
- ✅ JWT authentication
- ✅ Prometheus metrics

## 📖 API Documentation

Once running, visit:
- Health: `http://localhost:8000/health`
- Metrics: `http://localhost:8000/metrics`

## 🐳 Docker

```bash
# Build and run with Docker Compose
docker compose up --build

# Run in production mode
docker compose -f docker-compose.prod.yml up -d
```

## 📝 License

MIT
