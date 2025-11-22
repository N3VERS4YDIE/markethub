# MarketHub Test Plan

## 1. Introduction

This playbook explains the automated checks we run before merging or shipping MarketHub. The workflow is intentionally simple so every developer does the same steps:

- Prove that critical flows (auth, RBAC, catalog, cart, checkout, analytics) match the requirements in `PROJECT.md`.
- Catch regressions early through repeatable automation (`cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`, `cargo audit`).
- Keep expectations identical locally and in CI so “run cargo, push, pass CI” is the entire routine.

Treat this document like a living checklist.

## 2. Test Scope

### 2.1 What We Test (via automated suites)

#### **Authentication & Authorization**

- User registration with email/password validation
- Login flow with JWT token generation
- Password hashing (Argon2) and verification
- Auth middleware (required/optional user extraction)
- Permission middleware and RBAC enforcement

#### **Store Management**

- Store creation with owner bootstrapping
- Slug uniqueness validation
- Public/private store filtering
- Store member listing and role assignments
- Access grant invitation and revocation

#### **Product & Inventory**

- Product CRUD operations with price/stock validation
- SKU uniqueness per store
- Stock decrement during checkout
- Product listing scoped by store

#### **Shopping Cart**

- Add/remove items with quantity validation
- Cart persistence per user
- Multi-store cart aggregation
- Cart clearing after checkout

#### **Order Processing**

- Atomic checkout workflow across multiple stores
- Order group creation with per-store orders
- Order item persistence with price locking
- Stock management during order creation
- Order history retrieval per user/store

#### **Analytics**

- Store revenue calculations
- Top products by quantity sold
- Order history with temporal queries
- Aggregate metrics per store

#### **Utilities & Validation**

- JWT token generation and verification
- Slug format validation (alphanumeric + hyphens)
- Shipping address JSON validation
- Price and quantity constraints
- Domain model validation (DTOs)


### 2.2 What We Skip (for now)

- Third-party payment processing (mocked).
- UI/front-end clients.
- Email/SMS notifications.
- Chaos testing and heavy-load labs.
- Performance/load testing.

#### **Excluded from Coverage (No Tests Required)**

- `main.rs` - Application entry point
- `lib.rs` - Library exports
- `server.rs` - Router wiring (validated by integration tests)
- `state.rs` - Simple state struct (validated by integration tests)
- `config.rs` - Config loading (validated by integration tests)
- `error.rs` - Error type definitions (used throughout tests)
- `mod.rs` files - Module declarations only

## 3. Test Approach

1. **Run the cargo trio**: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`. Nothing merges without all three green locally.
2. **Integration smoke in code**: The `tests/` harness boots the Axum app and exercises health/auth/analytics endpoints. Expand this suite whenever routers grow.
3. **Data discipline**: Use SQLx migrations for schema parity, seed helpers for deterministic fixtures, and transactional tests that rollback automatically.
4. **Continuous feedback**: GitHub Actions executes the same commands (plus `cargo audit`). If CI fails, fix the code/tests.

## 4. Testing Types & Implementation Status

| Type                | What We Test                            |
| ------------------- | --------------------------------------- |
| **Unit (Utils)**    | Password hashing, JWT, validators       |
| **Unit (Models)**   | Domain validation, state transitions    |
| **Service**         | Business logic, orchestration workflows |
| **Repository**      | SQL queries, transactions, constraints  |
| **Middleware**      | Auth extraction, permission checks      |
| **Integration/E2E** | Full HTTP flows, API contracts          |
| **Security**        | `cargo audit`, vulnerability scanning   |

## 5. Tools & Environment

- **Rust toolchain**: Stable 1.91+.
- **Core commands**: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`, `cargo audit`.
- **Coverage tool**: `cargo llvm-cov` - Generate HTML reports with `cargo llvm-cov --html`.
- **Database**: PostgreSQL 18 via `docker compose up -d postgres`, migrations applied with `sqlx-cli`.
- **Test crates**: `tokio-test`, `rstest`, `fake` for generators.
- **Infra**: `.env.example`, `docker-compose.yml`, and `sqlx migrate run` keep everyone on the same schema.
- **Optional tooling**: Additional automated runners (e.g., `cargo nextest`, load-test scripts) can be added later.

## 6. Risk Log

| Risk                   | Impact on Dev Loop                  | Mitigation                                                                                   |
| ---------------------- | ----------------------------------- | -------------------------------------------------------------------------------------------- |
| Schema drift           | SQL queries break after migration.  | Enforce `sqlx prepare` + run migrations in CI; keep fixtures updated.                        |
| Slow DB bootstrap      | Devs avoid integration tests.       | Provide docker-compose profile + lightweight transactional tests.                            |
| Flaky async tests      | Reduces trust in suite.             | Use deterministic timeouts, isolate state, prefer `#[tokio::test(flavor = "multi_thread")]`. |
| Permission regressions | Unauthorized access or lockouts.    | Add regression tests when fixing bugs; require code review on auth-heavy PRs.                |
| Latency spikes         | Checkout/analytics slow under load. | Add automated load scripts when needed; do not rely on ad-hoc manual tests.                  |

## 7. Entry & Exit Criteria

### Starting Tests

- Feature is defined (edge cases listed in code/tests).
- Local env ready: `.env` copied, Postgres running, migrations applied.
- All dependencies installed: `cargo build`

### Calling It Done

- `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` all pass locally.
- No HIGH severity issues in `cargo audit`.
- CI pipeline passes on the PR.
- New/updated automated tests cover the code paths touched.
- Flaky tests are fixed before merge.

## 8. What We Produce

- **Source-level tests**: Rust test modules under `tests/` and alongside source files.
- **Coverage reports**: HTML coverage reports via `cargo llvm-cov --html`.
- **Automation logs**: Outputs from local runs and CI artifacts for reference.
- **Issue tracker entries**: Bugs discovered during testing documented with repro steps.
- **Metrics**: Test execution time, coverage percentages.
