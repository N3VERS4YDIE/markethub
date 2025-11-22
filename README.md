# MarketHub 🏪

A production-ready, multi-tenant e-commerce platform with role-based access control and private storefronts.

## 📚 Documentation

See [PROJECT.md](./PROJECT.md) for complete architecture and development guidelines.

## 🏗️ Architecture

```
Handlers → Services → Repositories → Database
    ↓
  Models (Domain entities with validation)
```

## 🔐 Key Features

### Core Capabilities

- **Multi-Tenant Architecture**: Independent vendor storefronts with isolated data
- **Advanced RBAC**: Owner/Manager/Viewer roles with granular permission system
- **Private Storefronts**: Invitation-only stores with access grant management
- **Smart Shopping Cart**: Single cart aggregating products across multiple stores
- **Atomic Checkout**: Multi-store transactions with automatic stock management

### Security & Auth

- **JWT Authentication**: Secure token-based auth with configurable expiration
- **Argon2 Password Hashing**: Industry-standard password security
- **Permission Middleware**: Request-level authorization with membership validation
- **Soft Deletes**: User account recovery and data retention

### Operations & Monitoring

- **Store Analytics**: Revenue tracking, top products, order history per store
- **Prometheus Metrics**: Request latency, error rates, business KPIs
- **Grafana Dashboards**: Real-time monitoring and visualization
- **Structured Logging**: Distributed tracing with correlation IDs

### Developer Experience

- **Layered Architecture**: Clean separation (Handlers → Services → Repositories)
- **Type-Safe Queries**: SQLx compile-time verification
- **Comprehensive Testing**: Unit, service, integration, and E2E test suites
- **CI/CD Pipeline**: Automated format, lint, test, security audit, and Docker builds
- **Docker Ready**: Multi-stage builds with PostgreSQL integration


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
# Grafana at http://localhost:3000 (admin/admin)
# Prometheus at http://localhost:9090
```

### Run Tests

```bash
cargo test

# For coverage report
./scripts/coverage.sh
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
