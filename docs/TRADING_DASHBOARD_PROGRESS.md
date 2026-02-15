# Trading Dashboard — Port Progress

> Python backend → Rust `hyperspot-server` on branch `dev/phase-1-cricoai`

## Phase 1 — Project Scaffolding ✅

- [x] SDK crates (`trading-dashboard-sdk`)
- [x] Module wiring (`TradingDashboard` module struct, `module.rs`)
- [x] Config (`config/cricoai.yaml`, PostgreSQL `Binance` DB)
- [x] Server boots on `127.0.0.1:8087`

## Phase 2 — Trading Dashboard Endpoints ✅

All 8 endpoints live with PostgreSQL data (25 OpenAPI operations total).

| # | Endpoint | Description | Status |
|---|----------|-------------|--------|
| 1 | `GET /trading-dashboard/v1/stats/summary` | Trade count, open orders, P&L, asset value | ✅ |
| 2 | `GET /trading-dashboard/v1/stats/pnl-by-day` | Daily P&L aggregation (267 days) | ✅ |
| 3 | `GET /trading-dashboard/v1/stats/recent-trades` | Recent sells with symbol + P&L | ✅ |
| 4 | `GET /trading-dashboard/v1/stats/statistics` | Aggregate metrics, pair evolution | ✅ |
| 5 | `GET /trading-dashboard/v1/stats/top-pairs` | 198 pairs ranked by total P&L | ✅ |
| 6 | `GET /trading-dashboard/v1/stats/total-assets-value` | 24h hourly asset snapshots | ✅ |
| 7 | `GET /trading-dashboard/v1/orders/open` | Open buy orders (3726) | ✅ |
| 8 | `GET /trading-dashboard/v1/account/balances` | USDC, asset value, invested, P&L% | ✅ |

### Key files changed

- `libs/modkit-db/src/secure/cond.rs` — bug fix: unrestricted entities get `WHERE true`
- `trading-dashboard/src/api/rest/dto.rs` — 10 DTOs
- `trading-dashboard/src/api/rest/handlers/mod.rs` — 8 handlers
- `trading-dashboard/src/api/rest/routes/mod.rs` — 8 route registrations
- `trading-dashboard/src/domain/service/mod.rs` — 8 service methods
- `trading-dashboard/src/infra/storage/entity/trade_pairs.rs` — new entity
- `trading-dashboard/src/infra/storage/entity/mod.rs` — export

### Bugs found & fixed

1. **`build_scope_condition` deny-all for unrestricted entities** — `AccessScope::default()` is empty, triggering `WHERE false` for all `#[secure(unrestricted)]` entities. Fixed by adding Rule 0: check `IS_UNRESTRICTED` first, return `WHERE true`.
2. **`sell_orders.pnl` is ALL NULL** — P&L data lives in the `pnl` table, joined via `client_order_id`. Service methods updated to query `pnl` table directly.

## Phase 3 — Remaining Endpoints ⏳

### Model Dashboard (~8 endpoints)

Requires `Model_Data` PostgreSQL database connection.

| # | Endpoint | Description | Status |
|---|----------|-------------|--------|
| 1 | `GET /models/list` | List all ML models | ⏳ |
| 2 | `GET /models/{id}/metrics` | Model performance metrics | ⏳ |
| 3 | `GET /models/{id}/predictions` | Recent predictions | ⏳ |
| 4 | `GET /models/{id}/training-history` | Training runs | ⏳ |
| 5 | `GET /models/comparison` | Compare model performance | ⏳ |
| 6 | `GET /models/active` | Currently active models | ⏳ |
| 7 | `GET /models/{id}/config` | Model configuration | ⏳ |
| 8 | `GET /models/summary` | Aggregate model stats | ⏳ |

### Agent Analytics (~13 endpoints)

| Status |
|--------|
| ⏳ Not started — endpoints TBD from Python backend |

### Performance Monitor (~3 endpoints)

| Status |
|--------|
| ⏳ Not started — endpoints TBD from Python backend |

## Phase 4 — OData Support ⏳

Add `$filter`, `$orderby`, `$select` query support to list endpoints.

- [ ] Derive `ODataFilterable` on list DTOs
- [ ] Use `OData(query)` extractor in handlers
- [ ] Use `paginate_odata` in service methods
- [ ] Register with `OperationBuilderODataExt`

## Notes

- **Branch:** `dev/phase-1-cricoai`
- **Server:** `127.0.0.1:8087`
- **Database:** PostgreSQL `Binance` on `localhost:5432` (user: `binance`)
- **Auth:** Disabled for local dev (`auth_disabled: true`)
