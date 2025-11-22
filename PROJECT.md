# MarketHub - Multi-Vendor E-Commerce Platform

## 🎯 Project Vision

A production-ready, multi-tenant marketplace enabling **public and private storefronts** with **role-based access control** and **multi-store shopping**.

**Core Innovation**: Private stores with invitation-only access (B2B wholesale, exclusive clubs, employee stores).

---

## 🏗️ Architecture Philosophy

### **Pragmatic Principles**

1. **DRY (Don't Repeat Yourself)**: Shared logic lives in reusable modules
2. **Composition over Inheritance**: Prefer traits and functions over complex hierarchies
3. **Explicit over Implicit**: Clear, readable code beats clever abstractions
4. **Fail Fast**: Validate early, return meaningful errors
5. **Type Safety**: Leverage Rust's type system to prevent bugs at compile time

### **Code Organization**

```
Layered Architecture:
┌─────────────────────────────────────┐
│  Handlers (HTTP/API Layer)          │ ← Routes, request/response
├─────────────────────────────────────┤
│  Services (Business Logic)          │ ← Domain rules, orchestration
├─────────────────────────────────────┤
│  Repositories (Data Access)         │ ← Database operations
├─────────────────────────────────────┤
│  Models (Domain Entities)           │ ← Structs, enums, validation
└─────────────────────────────────────┘
```

**Rules:**

- Handlers NEVER contain business logic
- Services NEVER directly access database
- Repositories NEVER contain business rules
- Models are pure data + validation

---

## 📊 Domain Model

### **Core Entities** (6 Tables)

```
Users ←──────┬─────── Stores (owner_id)
  ↓          │          ↓
  ↓          │          ↓ has_many
  ↓          │       Products
  ↓          │          ↓
  ↓          │          ↓
CartItems    │       Orders ←─── OrderGroup
  ↓          │          ↓
  └──────────┼──────────┘
             │
             ├─────→ StoreMembers (many-to-many + roles)
             └─────→ StoreAccessGrants (private store invites)
```

### **Entity Responsibilities**

| Entity             | Purpose                                 | Key Rules                                  |
| ------------------ | --------------------------------------- | ------------------------------------------ |
| `User`             | Universal account (customer + employee) | Email unique, soft delete                  |
| `Store`            | Independent storefront                  | Public/private, owner cannot be removed    |
| `Product`          | Store inventory                         | SKU unique per store, belongs to one store |
| `Order`            | Purchase from ONE store                 | Status flow, stock validation              |
| `OrderGroup`       | Multi-store checkout session            | Links related orders                       |
| `CartItem`         | Pre-checkout product selection          | No expiration, cross-store                 |
| `StoreMember`      | User-Store association with role        | RBAC permissions, audit trail              |
| `StoreAccessGrant` | Private store invitation                | Expirable, revocable, access levels        |

---

## 🔐 Authorization Model

### **Two-Layer Security**

1. **Authentication**: JWT-based, stateless
2. **Authorization**: Context-dependent (store membership + permissions)

### **Permission Check Flow**

```rust
Request → Authenticate → Extract user_id → Check context:
  
  Is public store? → Allow read
  Is store member? → Check permissions
  Has access grant? → Check access level
  Else → Deny
```

### **Permission Granularity**

```rust
pub enum Permission {
    // Products
    ViewProducts, CreateProducts, EditProducts, DeleteProducts,
    
    // Orders
    ViewOrders, ProcessOrders, CancelOrders,
    
    // Members
    ViewMembers, InviteMembers, EditPermissions,
    
    // Access (private stores)
    GrantAccess, RevokeAccess,
    
    // Analytics
    ViewStats, ExportReports,
}

// Predefined roles (JSON in DB)
Owner:  [ALL]
Admin:  [ALL except store deletion]
Manager: [Products, Orders, Stats]
Staff:   [ViewProducts, ViewOrders]
```

---

## 🚀 Technical Stack

### **Infrastructure**

- **Language**: Rust
- **Web Framework**: Axum
- **Database**: PostgreSQL
- **Cache**: Redis (future: session, rate limiting)
- **Container**: Docker + docker compose
- **CI/CD**: GitHub Actions
- **Monitoring**: Prometheus + Grafana 

---

## 📐 Database Design Principles

1. **UUIDs for IDs**: Distributed-friendly, no collision risk
2. **Timestamps**: `created_at`, `updated_at` on all tables
3. **Soft Deletes**: `is_active` flag for users, stores
4. **Indexes**: On foreign keys, frequently queried fields
5. **JSONB**: For flexible data (permissions, addresses)
6. **Constraints**: Enforce rules at DB level (unique, foreign keys, check)

---

## 🧪 Testing Strategy

### **Test Pyramid**

```
       /\
      /E2E\         10% - Full API flows
     /──────\
    /  INT   \      30% - Service + Repository integration
   /──────────\
  /   UNIT     \    60% - Business logic, validation
 /──────────────\
```

---

## 🔄 Development Workflow

### **Git Strategy**

- `main`: Production-ready
- `dev`: Integration branch
- Feature branches: `feat/store-permissions`
- Fix branches: `fix/cart-validation`

### **Commit Convention**

```
feat: add private store access control
fix: prevent duplicate SKU across stores
refactor: extract permission validation to middleware
test: add order checkout integration tests
docs: update API endpoint documentation
```

### **CI/CD Pipeline**

```yaml
On PR:
  ✓ Cargo fmt check
  ✓ Cargo clippy (deny warnings)
  ✓ Cargo test (all tests)
  ✓ Cargo audit (security)
  ✓ Build Docker image

On merge to main:
  ✓ All above
  ✓ Push Docker image to registry
  ✓ Deploy to staging
```

---

## 📏 Code Quality Standards

### **Rust Idioms**

```rust
// ✅ DO: Use Result for fallible operations
pub async fn get_product(id: Uuid) -> Result<Product, Error>

// ✅ DO: Use Options for nullable values
pub struct User { avatar_url: Option<String> }

// ✅ DO: Use enums for state
pub enum OrderStatus { Pending, Confirmed, Shipped }

// ❌ DON'T: Use panic in business logic
// ❌ DON'T: Use unwrap() outside tests
// ❌ DON'T: Create giant structs/functions
```

### **Naming Conventions**

- Types: `PascalCase` (User, OrderService)
- Functions: `snake_case` (get_user, create_order)
- Constants: `SCREAMING_SNAKE` (MAX_CART_ITEMS)
- Modules: `snake_case` (user_service, order_repository)

### **Function Size**

- Max 50 lines per function (ideally < 30)
- Extract complex logic to helper functions
- One level of indentation preferred

---

## 🚨 Non-Goals (Out of Scope)

- ❌ Payment processing (mock only)
- ❌ Email service (simulated logs)
- ❌ File uploads (use URLs)
- ❌ Real-time chat
- ❌ Mobile apps
- ❌ Microservices (monolith first)
- ❌ GraphQL (REST only)
- ❌ Frontend (API only)

---

## 📚 API Design Principles

1. **RESTful**: Standard HTTP methods + status codes
2. **Consistent**: All responses follow same structure
3. **Versioned**: `/api/v1/...` (future-proof)
4. **Documented**: OpenAPI/Swagger spec
5. **Paginated**: List endpoints return max 50 items
6. **Filtered**: Support query params for filtering
7. **Errors**: Meaningful messages with error codes

### **Standard Response Format**

```json
// Success
{
  "data": { ... },
  "meta": { "timestamp": "..." }
}

// Error
{
  "error": {
    "code": "INSUFFICIENT_STOCK",
    "message": "Product 'Laptop' only has 2 items in stock",
    "details": { "available": 2, "requested": 5 }
  }
}

// List
{
  "data": [...],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 150
  }
}
```

---

## 🤖 AI Agent Guidelines

When modifying this project:

1. **Read context first**: Check existing patterns before implementing
2. **Follow the layers**: Don't mix concerns across boundaries
3. **Reuse existing code**: Check for similar functionality before creating new
4. **Validate inputs**: Use validator crate on all DTOs
5. **Handle errors properly**: Propagate with `?`, don't unwrap
6. **Add tests**: Every new feature needs tests
7. **Update docs**: Keep API docs in sync
8. **Check consistency**: Naming, structure, patterns must align

### **Before Creating New Code**

Ask:

- Does similar functionality exist?
- Can I extract common logic?
- Is this the right layer for this code?
- Have I handled all error cases?
- Is this testable?

---

## 📖 Quick Reference

| Task           | Command                       |
| -------------- | ----------------------------- |
| Run locally    | `cargo run`                   |
| Run tests      | `cargo test`                  |
| Check code     | `cargo clippy`                |
| Format         | `cargo fmt`                   |
| DB migrate     | `sqlx migrate run`            |
| Docker build   | `docker build -t markethub .` |
| Docker compose | `docker compose up`           |

---

**Last Updated**: 2025-11-16
**Status**: Active Development
