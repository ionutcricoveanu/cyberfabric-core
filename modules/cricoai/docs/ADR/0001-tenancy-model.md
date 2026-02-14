# ADR-0001: Tenancy Model — Production/Testnet as Tenant IDs

**ID**: `cpt-cf-cricoai-adr-tenancy-model`

## Status

Accepted

## Context and Problem Statement

CricoAI v1 uses separate PostgreSQL databases for production (`Binance`) and testnet (`BinanceTN`) trading, switched via an `env=production|testnet` query parameter. cyberfabric-core's tenancy model uses `SecurityContext` → `AccessScope` → `SecureConn` with `tenant_id` columns on all entities. These two approaches are incompatible.

We need a tenancy strategy that:
- Fits cyberfabric-core's `SecureConn` + `AccessScope` pattern
- Works with the existing separate-database layout
- Requires minimal migration effort
- Prevents cross-tenant data leakage by construction

## Decides For Requirements

- `cpt-cf-cricoai-nfr-dna-compliance` — tenant scoping via SecurityContext, not query params
- `cpt-cf-cricoai-fr-trading-stats` — all data queries scoped by tenant

## Decision Drivers

- cyberfabric-core enforces `SecureConn` for all DB access — raw connections forbidden
- `AccessScope` requires a `tenant_id` on entities (or `unrestricted` for global tables)
- Tenant ID must flow from JWT, not from user-supplied query parameters (security)
- Existing databases contain production data that cannot be lost during migration

## Considered Options

### Option A: Add tenant_id column to existing tables

Add a `tenant_id` UUID column to all trading tables via SeaORM migrations. Backfill existing rows: production rows get tenant UUID A, testnet rows get tenant UUID B. Both environments share the same database. All entities derive `Scopable` with `tenant_col = "tenant_id"`.

**Pros:**
- Full SecureConn compatibility with no workarounds
- Single database simplifies operations
- Standard cyberfabric-core pattern

**Cons:**
- Migration adds column + backfill to every existing table
- Slight storage overhead (16 bytes per row)
- Must handle kline data (Binance_Klines) and model data (Model_Data) which are shared

### Option B: Database-per-tenant with tenant resolver

Map each tenant UUID to a separate database connection. The tenant resolver selects the connection based on `SecurityContext.tenant_id()`. Tables do not need a `tenant_id` column. Entities use `#[secure(unrestricted)]`.

**Pros:**
- No schema changes to existing tables
- Physical data isolation
- Matches existing database layout

**Cons:**
- `unrestricted` entities bypass SecureConn scoping — weaker security guarantees
- Cannot query across tenants (e.g., compare production vs testnet)
- cyberfabric-core's `AccessScope` provides no value if all entities are unrestricted
- Shared data (klines, model data) must be duplicated or use a third connection

### Option C: Hybrid — tenant_id on trading tables, shared databases for klines/models

Add `tenant_id` to trading tables (`Binance`/`BinanceTN`) and consolidate into one database. Keep `Binance_Klines` and `Model_Data` as shared databases with `#[secure(unrestricted)]` since kline data and trained models are not tenant-specific.

## Decision Outcome

**Chosen option: Option C (Hybrid)**

Trading data tables (pnl, buy_orders, sell_orders, etc.) gain a `tenant_id` column and are consolidated into a single `cricoai` database. Market data (`Binance_Klines`) and model data (`Model_Data`) remain shared and use `#[secure(unrestricted)]` since they are not tenant-scoped.

**Tenant mapping:**
- Production: `tenant_id = 00000000-0000-0000-0000-000000000001` (static UUID)
- Testnet: `tenant_id = 00000000-0000-0000-0000-000000000002` (static UUID)

**JWT carries tenant_id** — the login endpoint returns a JWT with the tenant claim. The frontend never sends `env=` parameters. modkit-auth extracts tenant_id into SecurityContext automatically.

## Consequences

**Positive:**
- Full SecureConn scoping on trading data — cross-tenant leakage impossible by construction
- Standard cyberfabric-core pattern for all trading queries
- Single database for trading data simplifies backup/restore
- Kline and model data shared efficiently (no duplication)

**Negative:**
- One-time migration to add `tenant_id` to ~15 trading tables and backfill rows
- `unrestricted` entities for klines/models are less governed but acceptable (shared data by nature)
- Static UUIDs must be consistent across environments

## Confirmation

- All trading entities derive `Scopable` with `tenant_col = "tenant_id"`
- Kline/model entities derive `Scopable` with `#[secure(unrestricted)]`
- No handler uses `env` query parameter — tenant flows from JWT only
- Integration tests verify tenant isolation: tenant A cannot see tenant B's orders

## Traceability

- **PRD**: [PRD.md](../PRD.md) — `cpt-cf-cricoai-nfr-dna-compliance`
- **DESIGN**: [DESIGN.md](../DESIGN.md) — §3.4 Tenancy & DB Access
