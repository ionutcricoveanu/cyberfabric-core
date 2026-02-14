# ADR-0003: DNA REST API Compliance for All Endpoints

**ID**: `cpt-cf-cricoai-adr-dna-api-compliance`

## Status

Accepted

## Context and Problem Statement

CricoAI v1's FastAPI endpoints use ad-hoc JSON shapes, no pagination standard, no error format, and no concurrency control. Rebuilding on cyberfabric-core requires choosing an API standard. cyberfabric-core includes the DNA (Development Norms & Architecture) guidelines as a submodule at `guidelines/DNA/`, providing opinionated REST API conventions.

We need to decide whether to adopt DNA fully, partially, or use a different standard.

## Decides For Requirements

- `cpt-cf-cricoai-nfr-dna-compliance` — 100% endpoint compliance with DNA standard
- `cpt-cf-cricoai-interface-rest-api` — REST API surface conventions

## Decision Drivers

- cyberfabric-core's ModKit provides built-in support for DNA patterns: `OperationBuilder`, `ODataFilterable`, `paginate_odata`, `page_to_projected_json`, RFC 9457 `Problem` type
- DNA aligns with industry standards (RFC 9457, OData, IETF RateLimit, W3C Trace Context)
- Consistency across all endpoints reduces frontend integration effort
- HAI3 frontend benefits from predictable response shapes

## Considered Options

### Option A: Full DNA adoption from day one

All endpoints comply with DNA from the first implementation. List responses use `{ items, page_info }`. All lists support OData `$filter`/`$orderby`/`$select` and cursor pagination. Errors use RFC 9457 Problem Details. Writes use `ETag`/`If-Match` and `Idempotency-Key`. OpenAPI 3.1 via utoipa on every endpoint.

**Pros:**
- No technical debt — correct from the start
- ModKit provides all the building blocks
- Frontend can use a single response parser for all endpoints
- OpenAPI generated automatically

**Cons:**
- Slightly higher effort per endpoint during initial port
- OData may be overkill for simple summary endpoints

### Option B: Gradual adoption — basic JSON first, DNA later

Port endpoints with simple JSON responses first. Add DNA compliance (pagination, filtering, errors) in a later phase.

**Pros:**
- Faster initial port
- Lower initial complexity

**Cons:**
- Creates technical debt that must be repaid
- Frontend must handle two response formats during transition
- Retrofitting OData onto existing endpoints is more work than building it in

### Option C: Custom API standard

Design a simpler custom standard that borrows selectively from DNA.

**Pros:**
- Tailored to CricoAI's needs

**Cons:**
- Diverges from cyberfabric-core ecosystem
- Cannot use ModKit's built-in OData/pagination infrastructure
- Must maintain custom documentation

## Decision Outcome

**Chosen option: Option A (Full DNA adoption from day one)**

Every endpoint implements DNA conventions from the first line of code. The overhead per endpoint is minimal because ModKit provides derive macros and builder helpers.

### DNA Conventions Applied

| Convention | Implementation | ModKit Support |
|-----------|---------------|----------------|
| **JSON shape** | snake_case, lists as `{ items, page_info }`, single objects unwrapped | serde rename_all |
| **Timestamps** | ISO-8601 UTC with `.000Z` milliseconds | chrono serialization |
| **Pagination** | Cursor-based, `limit` (default 25, max 200), opaque `cursor` | `paginate_odata`, `Page<T>` |
| **Filtering** | OData `$filter` (`eq`, `ne`, `gt`, `ge`, `lt`, `le`, `in`, `and`, `or`) | `ODataFilterable` derive |
| **Sorting** | OData `$orderby` (e.g., `priority desc,created_at asc`) | `OperationBuilderODataExt` |
| **Field projection** | OData `$select` (e.g., `$select=id,symbol,pnl`) | `apply_select`, `page_to_projected_json` |
| **Errors** | RFC 9457 Problem Details (`application/problem+json`) with `trace_id` | `Problem` type (implements `IntoResponse`) |
| **Concurrency** | `ETag` + `If-Match` on writes (412 on mismatch) | Manual (per handler) |
| **Idempotency** | `Idempotency-Key` on POST/PATCH/DELETE | Manual (per handler) |
| **Creates** | 201 + `Location` header + resource body | Manual (per handler) |
| **Deletes** | Soft-delete with `deleted_at`, return 200 | SeaORM entity pattern |
| **OpenAPI** | 3.1 via utoipa annotations, published at `/v1/openapi.json` | `OperationBuilder` auto-registers |
| **Auth** | Bearer JWT, `.require_auth()` on OperationBuilder | modkit-auth middleware |
| **Tracing** | `traceparent` propagation, `trace_id` in responses | OpenTelemetry integration |

### Endpoint Registration Pattern

```rust
OperationBuilder::get("/trading-dashboard/v1/stats/recent-trades")
    .operation_id("trading_dashboard.list_recent_trades")
    .require_auth(&Resource::TradingStats, &Action::Read)
    .handler(handlers::list_recent_trades)
    .json_response_with_schema::<modkit_odata::Page<dto::TradeDto>>(
        openapi, StatusCode::OK, "Paginated list of recent trades",
    )
    .with_odata_filter::<dto::TradeDtoFilterField>()
    .with_odata_select()
    .with_odata_orderby::<dto::TradeDtoFilterField>()
    .standard_errors(openapi)
    .register(router, openapi);
```

### DTO Pattern

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ODataFilterable)]
pub struct TradeDto {
    #[odata(filter(kind = "Uuid"))]
    pub id: Uuid,
    #[odata(filter(kind = "String"))]
    pub symbol: String,
    pub price: Decimal,
    pub quantity: Decimal,
    #[odata(filter(kind = "DateTimeUtc"))]
    pub created_at: DateTime<Utc>,
}
```

## Consequences

**Positive:**
- Zero technical debt — every endpoint is DNA-compliant from the start
- Frontend uses a single `PageResponse<T>` type for all list endpoints
- OData queries enable powerful filtering/sorting without custom endpoint proliferation
- OpenAPI documentation is always complete and current
- RFC 9457 errors are machine-parseable and include trace context

**Negative:**
- Summary endpoints (e.g., stats/summary) that return a single object do not benefit from OData, but they still use snake_case and Problem errors — minimal overhead
- Developer must learn OData derive macros and OperationBuilder helpers (one-time learning curve)

## Confirmation

- Every list endpoint returns `{ items, page_info }` — verified by integration tests
- Every endpoint has utoipa OpenAPI annotation — verified by `/v1/openapi.json` completeness check
- Every 4xx/5xx response is RFC 9457 Problem Details — verified by error test suite
- Write endpoints require `Idempotency-Key` — verified by 400 response without header

## Traceability

- **PRD**: [PRD.md](../PRD.md) — `cpt-cf-cricoai-nfr-dna-compliance`
- **DESIGN**: [DESIGN.md](../DESIGN.md) — §3.5 REST API Conventions
- **Reference**: `cyberfabric-core/guidelines/DNA/REST/API.md`
- **Reference**: `cyberfabric-core/docs/modkit_unified_system/07_odata_pagination_select_filter.md`
