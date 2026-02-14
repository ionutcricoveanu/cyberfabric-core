# DECOMPOSITION — CricoAI v2

## 1. Overview

### 1.1 Context

This document breaks down the CricoAI v2 rebuild into implementation phases with dependencies and verification criteria. Each phase is independently valuable — the system works after any phase.

### 1.2 Related Artifacts

- **PRD**: [PRD.md](./PRD.md) — `cpt-cf-cricoai-fr-*` requirements
- **DESIGN**: [DESIGN.md](./DESIGN.md) — module catalog, API contracts
- **ADRs**: [ADR/](./ADR/) — tenancy, gRPC bridge, DNA compliance

## 2. Requirements Coverage

| PRD Requirement | Phase |
|----------------|-------|
| `cpt-cf-cricoai-fr-trading-stats` | Phase 1, 2 |
| `cpt-cf-cricoai-fr-model-metrics` | Phase 2 |
| `cpt-cf-cricoai-fr-agent-analytics` | Phase 2 |
| `cpt-cf-cricoai-fr-config-crud` | Phase 3 |
| `cpt-cf-cricoai-fr-pair-management` | Phase 3 |
| `cpt-cf-cricoai-fr-user-management` | Phase 3 |
| `cpt-cf-cricoai-fr-kline-collection` | Phase 5 |
| `cpt-cf-cricoai-fr-price-sse` | Phase 5 |
| `cpt-cf-cricoai-fr-data-ingestion` | Phase 6 |
| `cpt-cf-cricoai-fr-sentiment-analysis` | Phase 6 |
| `cpt-cf-cricoai-fr-agent-decision-sse` | Phase 6 |
| `cpt-cf-cricoai-fr-ml-grpc-bridge` | Phase 5 |
| `cpt-cf-cricoai-fr-trading-loop` | Phase 7 |
| `cpt-cf-cricoai-fr-trading-control` | Phase 7 |
| `cpt-cf-cricoai-fr-trade-event-sse` | Phase 7 |
| `cpt-cf-cricoai-nfr-dna-compliance` | Phase 1 (enforced from day one) |
| `cpt-cf-cricoai-nfr-trading-parity` | Phase 7 |
| HAI3 frontend | Phase 4 |

## 3. Implementation Phases

### Phase 1: Foundation, SDK Contracts & First Endpoint

**ID**: `cpt-cf-cricoai-feature-phase-1-foundation`

**Implements:** `cpt-cf-cricoai-component-trading-dashboard`, `cpt-cf-cricoai-principle-sdk-pattern`, `cpt-cf-cricoai-principle-secure-conn`, `cpt-cf-cricoai-principle-dna-rest`, `cpt-cf-cricoai-constraint-preserve-schema`

**Goal:** Prove the integration works. Establish SDK contracts for all modules. Ship one working DNA-compliant endpoint.

**Tasks:**
1. Add cyberfabric-core as git submodule (done)
2. Create all 8 SDK crates with API traits, models, and errors (empty implementations):
   - `trading-dashboard-sdk`
   - `config-manager-sdk`
   - `auth-manager-sdk`
   - `market-data-sdk`
   - `data-ingestion-sdk`
   - `market-intelligence-sdk`
   - `trading-core-sdk`
   - `ml-service-sdk` (with proto file and gRPC client)
3. Create `trading-dashboard` module (implementation crate) following `guidelines/NEW_MODULE.md`
4. Add `tenant_id` column to existing tables via SeaORM migration (see ADR-0001)
5. Generate SeaORM entities from existing PostgreSQL schema, derive `Scopable` on all entities
6. Port the simplest endpoint: `GET /trading-dashboard/v1/stats/summary` with:
   - `#[modkit::module]` declaration
   - SecureConn + AccessScope for tenant-scoped query
   - utoipa OpenAPI annotation
   - RFC 9457 error handling
   - OData support where applicable
7. Configure modkit-auth for JWT validation
8. Verify: server boots, endpoint returns correct DNA-compliant JSON from PostgreSQL

**Deliverable:** One working Rust endpoint + all SDK crate contracts defined. Inter-module API surface validated at compile time.

**Verification:**
- [ ] Server boots with `trading-dashboard` module registered
- [ ] `GET /trading-dashboard/v1/stats/summary` returns correct JSON scoped by tenant_id
- [ ] Response uses `snake_case`, ISO-8601 timestamps with `.000Z`
- [ ] Error responses use RFC 9457 Problem Details
- [ ] OpenAPI available at `/v1/openapi.json`
- [ ] All 8 SDK crates compile (traits + models defined)

---

### Phase 2: Complete Dashboard API

**ID**: `cpt-cf-cricoai-feature-phase-2-dashboard-api`

**Implements:** `cpt-cf-cricoai-component-trading-dashboard`

**Goal:** Port all read-only query endpoints with full DNA compliance.

**Dependencies:** Phase 1

**Tasks:**
1. Port trading stats endpoints (~6 endpoints)
2. Port order query endpoints (~3 endpoints)
3. Port account balance endpoint
4. Port model dashboard endpoints (~6 endpoints)
5. Port agent analytics endpoints (~5 endpoints)
6. Port agent trade impact endpoints (~3 endpoints)
7. Port performance monitor endpoints (~3 endpoints)
8. Add OData `$filter`/`$orderby`/`$select` on all list endpoints using `ODataFilterable` derive + `OperationBuilderODataExt`
9. Add cursor-based pagination via `paginate_odata` on all list endpoints
10. Implement `TradingDashboardApi` SDK trait (local_client.rs), register in ClientHub

**Deliverable:** Complete read-only API with DNA-compliant OData queries, pagination, field projection.

**Verification:**
- [ ] All ~22 read-only endpoints return correct data
- [ ] List endpoints support `$filter`, `$orderby`, `$select`, cursor pagination
- [ ] Responses match `{ items, page_info }` envelope for lists
- [ ] All endpoints documented in OpenAPI with utoipa annotations
- [ ] Automated test comparing JSON output to existing FastAPI equivalents

---

### Phase 3: Config & Auth Modules

**ID**: `cpt-cf-cricoai-feature-phase-3-config-auth`

**Implements:** `cpt-cf-cricoai-component-config-manager`, `cpt-cf-cricoai-component-auth-manager`, `cpt-cf-cricoai-principle-client-hub`

**Goal:** Port the write-path endpoints with full DNA write conventions.

**Dependencies:** Phase 1

**Tasks:**
1. Create `config-manager` module
   - Dynamic pair config CRUD via SecureConn
   - Static config via `ctx.module_config::<Config>()`
   - `ETag`/`If-Match` on PUT/PATCH endpoints
   - `Idempotency-Key` on POST/PATCH/DELETE
   - Register `ConfigManagerApi` in ClientHub
2. Create `auth-manager` module
   - User CRUD, role management, 2FA, IP blocks, audit logs
   - Soft-delete with `deleted_at`
   - `ETag`/`If-Match` on updates
   - `Idempotency-Key` on all writes
   - Register `AuthManagerApi` in ClientHub
3. Wire modkit-auth JWT middleware for all protected routes (`.require_auth()` on OperationBuilder)
4. Test auth flow end-to-end: login → JWT → protected endpoint → tenant-scoped data

**Deliverable:** Full API parity with existing FastAPI backend. All writes DNA-compliant.

**Verification:**
- [ ] Config GET/PATCH/PUT work with ETag concurrency control
- [ ] User CRUD works with proper 201 + Location on create
- [ ] 2FA setup/verify/disable flow works end-to-end
- [ ] IP blocking works; blocked IP gets 403
- [ ] Audit logs queryable with OData `$filter`
- [ ] Auth flow: login → JWT → protected endpoint returns tenant-scoped data
- [ ] Unauthorized requests get 401; insufficient permissions get 403

---

### Phase 4: HAI3 Frontend

**ID**: `cpt-cf-cricoai-feature-phase-4-frontend`

**Implements:** `cpt-cf-cricoai-seq-module-dependency-graph`

**Goal:** Build the new frontend from scratch using HAI3 screenset architecture.

**Dependencies:** Phase 2, Phase 3

**Tasks:**
1. Scaffold HAI3 app using `@hai3/cli create`
2. Create 6 screensets: Trading, Models, Agents, Performance, Settings, Admin
3. Implement API services with RestProtocol → cyberfabric-core endpoints
4. Build screens using `@hai3/uikit` components
5. Implement event-driven state management (eventBus + Redux slices)
6. Add mock plugins for development mode (each API service registers mock)
7. Configure auth plugin for JWT login flow
8. Add dark/light theme support
9. Subscribe to SSE streams for real-time updates (when available in later phases)

**Deliverable:** Fully functional frontend replacing React + Streamlit dashboards.

**Verification:**
- [ ] All 6 screensets render with correct data
- [ ] Auth flow: login → JWT → protected screens
- [ ] OData queries work from frontend (filtering, sorting, pagination)
- [ ] Theme switching works (dark/light)
- [ ] Mock mode works without backend

See [frontend.md](./frontend.md) for detailed HAI3 architecture.

---

### Phase 5: ML gRPC Bridge & Market Data Module

**ID**: `cpt-cf-cricoai-feature-phase-5-ml-market-data`

**Implements:** `cpt-cf-cricoai-component-ml-service-sdk`, `cpt-cf-cricoai-component-market-data`, `cpt-cf-cricoai-constraint-ml-oop-only`

**Goal:** Connect Python ML service via gRPC. Replace Python kline collector with Rust.

**Dependencies:** Phase 1 (SDK crates)

**Tasks:**

**5a: ML gRPC Bridge**
1. Implement `ml-service-sdk` gRPC client using `modkit_transport_grpc::client` utilities
2. Compile proto with `tonic-build` in SDK's `build.rs`
3. Implement `MlServiceGrpcClient` (implements `MlServiceApi` trait)
4. Implement `wire_client()` / `build_client()` wiring helpers
5. Generate Python gRPC stubs with `grpcio-tools`
6. Wrap `PredictionSystem` in thin Python gRPC server adapter
7. Wrap `UnifiedTrainer` status/trigger in gRPC server
8. Test: Rust client calls Python server, receives valid predictions

**5b: Market Data Module**
1. Create `market-data` module with `lifecycle(entry = "run_collector", stop_timeout = "30s", await_ready)`
2. Build Binance REST client using `modkit-http` (tower stack with TLS, retries, timeouts, OTel tracing)
3. Port kline collection logic from `klines_unified_collector.py`
4. Implement SSE broadcaster for price updates
5. Register `MarketDataApi` in ClientHub
6. Parallel run both collectors, validate data matches over 48 hours
7. Cut over to Rust-only

**Deliverable:** Python ML accessible via type-safe gRPC. Kline collection in Rust.

**Verification:**
- [ ] `ctx.client_hub().get::<dyn MlServiceApi>()` resolves and returns valid predictions
- [ ] gRPC client handles connection failures with retry
- [ ] Kline data matches between Python and Rust collectors over 48 hours
- [ ] SSE price stream works from frontend
- [ ] Graceful shutdown via CancellationToken stops all collector tasks

---

### Phase 6: Data Ingestion & Market Intelligence

**ID**: `cpt-cf-cricoai-feature-phase-6-ingestion-intelligence`

**Implements:** `cpt-cf-cricoai-component-data-ingestion`, `cpt-cf-cricoai-component-market-intelligence`

**Goal:** Replace Python agent service with Rust modules using LLM Gateway.

**Dependencies:** Phase 5 (market-data available via ClientHub)

**Tasks:**

**6a: Data Ingestion Module**
1. Create `data-ingestion` module with `lifecycle(entry = "run_ingestion")`
2. Port CryptoPanic, CoinGecko, Reddit RSS, Fear & Greed API clients using `modkit-http`
3. Write to `market_sentiment` table via SecureConn
4. Register `DataIngestionApi` in ClientHub

**6b: Market Intelligence Module**
1. Create `market-intelligence` module with `lifecycle(entry = "run_scheduler")`
2. Wire to `llm-gateway` via `ctx.client_hub().get::<dyn LlmGatewayApi>()`
3. Wire to `data-ingestion` via `ctx.client_hub().get::<dyn DataIngestionApi>()`
4. Port tier-based agent scheduler logic from `agents/scheduler.py`
5. Port sentiment analysis prompts from `agents/market_intelligence/agent.py`
6. Port confidence scoring and decay from `agents/market_intelligence/market_context.py`
7. Write decisions to `agent_decisions` table via SecureConn
8. Implement SSE broadcaster for agent decisions
9. Add REST endpoints: `GET /market-intelligence/v1/decisions` (OData), `GET /market-intelligence/v1/status`
10. Register `MarketIntelligenceApi` in ClientHub

**6c: Validation**
1. Compare agent decisions from Rust vs Python over 48 hours
2. Cut over to Rust-only

**Deliverable:** Agent system running in Rust via LLM Gateway. Python agent container eliminated.

**Verification:**
- [ ] Data ingestion collects from all sources with rate limiting
- [ ] Agent decisions produced on schedule (5m/15m/30m/1h tiers)
- [ ] Decisions include confidence scores with decay
- [ ] SSE agent decision stream works
- [ ] LLM usage tracked (cost, latency, token count)
- [ ] Decision quality matches Python agent over 48 hours

---

### Phase 7: Trading Core Module

**ID**: `cpt-cf-cricoai-feature-phase-7-trading-core`

**Implements:** `cpt-cf-cricoai-component-trading-core`, `cpt-cf-cricoai-constraint-rust-binance-client`

**Goal:** Rewrite the trading bot itself in Rust.

**Dependencies:** Phase 5 (ml-service-sdk, market-data), Phase 6 (market-intelligence)

**Tasks:**
1. Create `trading-core` module with `lifecycle(entry = "run_trading_loop", stop_timeout = "60s")`
2. Build Binance order execution client (REST via `modkit-http` + WebSocket via `tokio-tungstenite`)
3. Port BuyOrderManager — decision pipeline, filter validation, order placement
4. Port SellOrderManager — profit targets, stop-loss, trailing stops
5. Port CircuitBreaker, TradeSignalManager, ProfitMonitor
6. Wire inter-module calls:
   - `ctx.client_hub().get::<dyn MlServiceApi>()` for predictions
   - `ctx.client_hub().get::<dyn MarketIntelligenceApi>()` for sentiment
   - `ctx.client_hub().get::<dyn ConfigManagerApi>()` for pair config
   - `ctx.client_hub().get::<dyn MarketDataApi>()` for prices
7. Implement SSE broadcaster for trade events
8. Add REST endpoints: status, pause, resume (with Idempotency-Key)
9. Register `TradingCoreApi` in ClientHub
10. **Parallel testnet validation:** Run Rust and Python bots side-by-side for 2 weeks, compare every decision

**Deliverable:** Trading bot running in Rust, validated against Python version.

**Verification:**
- [ ] Rust trading core makes identical decisions to Python core on testnet for 2 weeks
- [ ] Pause/resume works without order corruption
- [ ] Circuit breaker triggers correctly on system failures
- [ ] SSE trade event stream works
- [ ] Graceful shutdown completes open operations before stopping

---

### Phase 8: Consolidation

**ID**: `cpt-cf-cricoai-feature-phase-8-consolidation`

**Implements:** `cpt-cf-cricoai-constraint-single-binary`

**Goal:** Remove old containers, finalize deployment.

**Dependencies:** All previous phases

**Tasks:**
1. Remove: FastAPI, auth-proxy, ELK stack (6 containers), Python trading bot, Python kline collector, Python agent service, Python data ingestion, Streamlit dashboards (2 containers)
2. Final Docker Compose: cyberfabric-core, ml-service, postgres, jaeger
3. Update CI/CD pipelines
4. Write operational runbook
5. Configure OpenTelemetry export to Jaeger
6. Production smoke test

**Deliverable:** Production-ready system with 4 containers instead of 15+.

**Verification:**
- [ ] All 4 containers start cleanly
- [ ] End-to-end flow: frontend → REST/SSE → cyberfabric-core → DB/gRPC → response
- [ ] OpenTelemetry traces visible in Jaeger across all modules
- [ ] All 7 modules healthy (`/health` endpoints)
- [ ] ML gRPC bridge functioning under production load

## 4. Phase Dependencies

```
Phase 1 (Foundation + SDK Contracts)
  ├── Phase 2 (Dashboard API)
  │     └── Phase 4 (HAI3 Frontend) ← also needs Phase 3
  ├── Phase 3 (Config + Auth)
  │     └── Phase 4 (HAI3 Frontend)
  └── Phase 5 (ML gRPC + Market Data)
        └── Phase 6 (Data Ingestion + Market Intelligence)
              └── Phase 7 (Trading Core)
                    └── Phase 8 (Consolidation)
```

Phases 2, 3, and 5 can run in parallel after Phase 1.

## 5. Traceability

- **PRD**: [PRD.md](./PRD.md)
- **DESIGN**: [DESIGN.md](./DESIGN.md)
- **ADRs**: [ADR/](./ADR/)
- **Frontend**: [frontend.md](./features/frontend.md)
