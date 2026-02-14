# PRD — CricoAI v2

## 1. Overview

### 1.1 Purpose

CricoAI v2 is a cryptocurrency trading platform rebuilt on cyberfabric-core (Rust) and HAI3 (React 19). It provides automated trading with ML-driven predictions, AI-powered sentiment analysis, and real-time market data collection. The rebuild replaces ~120K lines of unstructured Python across 15+ Docker containers with a governed, modular system running in 4 containers.

### 1.2 Background / Problem Statement

CricoAI v1 has grown organically into a functional but architecturally unsound system:

- **No architectural boundaries** — Python files reach across modules freely, making changes risky and unpredictable.
- **Fragmented infrastructure** — ELK stack (4 containers), separate auth-proxy (Node.js), separate FastAPI backend, two Streamlit dashboards — all doing overlapping work.
- **No formal contracts** — API shapes defined implicitly by raw SQL queries and dict construction. Changes break consumers silently.
- **No type safety** — Runtime errors from dict key mismatches, missing fields, and implicit None values.
- **Monolithic trading bot** — Single 1000+ line class handling buy logic, sell logic, risk management, and configuration in one file.

The proven trading logic and ML pipeline must be preserved. The goal is to execute them within cyberfabric-core's spec-driven, modular architecture with proper contracts, type safety, and observability.

### 1.3 Goals (Business Outcomes)

- **G1:** Reduce container count from 15+ to 4 (cyberfabric-core server, ml-service, PostgreSQL, Jaeger)
- **G2:** Achieve type-safe, contract-driven module boundaries enforced at compile time
- **G3:** Consolidate all dashboards into a single HAI3 frontend with 6 screensets
- **G4:** Replace ELK stack with OpenTelemetry + Jaeger for observability
- **G5:** Maintain identical trading decision quality validated by 2-week parallel testnet run
- **G6:** DNA-compliant REST API surface with OData queries, cursor pagination, RFC 9457 errors

### 1.4 Glossary

| Term | Definition |
|------|------------|
| ModKit | cyberfabric-core's module framework — lifecycle, REST, DB, ClientHub, errors |
| SDK pattern | Two-crate layout: `<module>-sdk` (public API trait + models) and `<module>` (implementation) |
| ClientHub | Type-safe inter-module communication — resolves clients by interface trait |
| SecureConn | Tenant-scoped database access layer — enforces AccessScope on all queries |
| DNA | Development Norms & Architecture — REST API standards for cyberfabric projects |
| OoP | Out-of-Process — module running as separate binary, communicating via gRPC |
| Screenset | HAI3's vertical slice pattern — screens + menu + API + state + translations |
| Tenant | Scoping unit for data isolation; CricoAI maps production and testnet as two tenants |

## 2. Actors

### 2.1 Human Actors

#### Trader (Primary User)

**ID**: `cpt-cf-cricoai-actor-trader`

**Role**: Monitors trading performance, reviews agent decisions, manages trading pairs, and controls bot operation (pause/resume).
**Needs**: Real-time visibility into P&L, open orders, model predictions, agent sentiment, and system health.

#### Administrator

**ID**: `cpt-cf-cricoai-actor-admin`

**Role**: Manages users, roles, 2FA, IP blocking, and reviews audit logs.
**Needs**: User CRUD, role assignment, security configuration, audit trail.

### 2.2 System Actors

#### Binance Exchange

**ID**: `cpt-cf-cricoai-actor-binance`

**Role**: External exchange providing market data (OHLCV klines) and order execution (REST + WebSocket).

#### Python ML Service

**ID**: `cpt-cf-cricoai-actor-ml-service`

**Role**: OoP gRPC service providing ML predictions, technical indicators, market regime detection, and model training. Wraps existing Python ML code (scikit-learn, XGBoost, LightGBM, ta-lib).

#### External Sentiment Sources

**ID**: `cpt-cf-cricoai-actor-sentiment-sources`

**Role**: External APIs providing market sentiment data — CryptoPanic, CoinGecko, Reddit RSS, Fear & Greed Index.

#### LLM Gateway

**ID**: `cpt-cf-cricoai-actor-llm-gateway`

**Role**: cyberfabric-core built-in module providing unified access to LLM providers (local Ollama or external). Used by market-intelligence module for AI-powered sentiment analysis.

#### HAI3 Frontend

**ID**: `cpt-cf-cricoai-actor-frontend`

**Role**: React 19 SPA consuming REST API + SSE streams from cyberfabric-core server.

## 3. Operational Concept & Environment

### 3.1 Module-Specific Environment Constraints

- PostgreSQL 16 with existing schema across 4 databases: `Binance` (production trading), `BinanceTN` (testnet trading), `Binance_Klines` (market data), `Model_Data` (ML/agent data)
- Production and testnet are modeled as separate tenants sharing the same cyberfabric-core instance (see ADR-0001)
- Python ML service requires: scikit-learn, XGBoost, LightGBM, pandas, numpy, ta-lib (C library) — no Rust equivalents exist
- Binance API rate limits: 1200 requests/minute (REST), 5 messages/second (WebSocket)

## 4. Scope

### 4.1 In Scope

- 7 cyberfabric-core modules: trading-dashboard, config-manager, auth-manager, market-data, market-intelligence, data-ingestion, trading-core
- 1 Rust SDK-only crate for ML service gRPC bridge (ml-service-sdk)
- 1 Python gRPC server wrapping existing ML code
- 1 HAI3 frontend with 6 screensets
- SeaORM entity generation from existing PostgreSQL schema
- Tenant-scoped data access (production/testnet) via SecurityContext + AccessScope
- DNA-compliant REST API with OData queries, cursor pagination, RFC 9457 errors, OpenAPI 3.1
- SSE endpoints for real-time trading events, price updates, and agent decisions
- OpenTelemetry observability replacing ELK stack

### 4.2 Out of Scope

- Rewriting ML/training pipeline in Rust (stays in Python permanently)
- Multi-user SaaS features (single-operator deployment)
- Mobile application
- Backtesting engine (future consideration)
- Additional exchange support beyond Binance (future consideration)

## 5. Functional Requirements

### 5.1 Trading Data & Dashboard

#### Read-only trading data API

- [ ] `p1` - **ID**: `cpt-cf-cricoai-fr-trading-stats`

The system MUST expose read-only REST endpoints for trading statistics, P&L, open orders, account balances, and trade history. All list endpoints MUST support OData `$filter`, `$orderby`, `$select`, and cursor-based pagination per DNA standard.

**Rationale**: Replaces 6 FastAPI router files with a single governed module.
**Actors**: `cpt-cf-cricoai-actor-trader`, `cpt-cf-cricoai-actor-frontend`

#### Model metrics API

- [ ] `p1` - **ID**: `cpt-cf-cricoai-fr-model-metrics`

The system MUST expose REST endpoints for model training history, calibration metrics, regime analysis, prediction history, and performance distribution.

**Rationale**: Provides visibility into ML model health and decision quality.
**Actors**: `cpt-cf-cricoai-actor-trader`, `cpt-cf-cricoai-actor-frontend`

#### Agent analytics API

- [ ] `p1` - **ID**: `cpt-cf-cricoai-fr-agent-analytics`

The system MUST expose REST endpoints for agent decisions, sentiment trends, LLM usage, trade impact analysis, and agent effectiveness metrics.

**Rationale**: Provides visibility into AI agent behavior and its impact on trading.
**Actors**: `cpt-cf-cricoai-actor-trader`, `cpt-cf-cricoai-actor-frontend`

### 5.2 Trading Configuration

#### Config CRUD

- [ ] `p2` - **ID**: `cpt-cf-cricoai-fr-config-crud`

The system MUST expose REST endpoints to read and update trading configuration (trade parameters, indicators, intervals, ML settings, profit targets, system settings). Dynamic pair-level configuration MUST be stored in the database and editable via REST. Static configuration MUST use cyberfabric-core's built-in typed config mechanism (`ctx.module_config::<Config>()`).

**Rationale**: Replaces settings_graphs.py with governed config management.
**Actors**: `cpt-cf-cricoai-actor-trader`

#### Pair management

- [ ] `p2` - **ID**: `cpt-cf-cricoai-fr-pair-management`

The system MUST expose REST endpoints to list trading pairs with evolution data, view technical indicator signals per pair, and update profit targets per pair. Write endpoints MUST support `Idempotency-Key` and `ETag`/`If-Match` concurrency control per DNA standard.

**Rationale**: Enables fine-grained control over which pairs are traded and at what thresholds.
**Actors**: `cpt-cf-cricoai-actor-trader`

### 5.3 Authentication & Security

#### User and role management

- [ ] `p2` - **ID**: `cpt-cf-cricoai-fr-user-management`

The system MUST expose REST endpoints for user CRUD, role assignment/removal, 2FA setup/verify/disable, IP blocking/whitelisting, and audit log queries. This module is the **data provider**; JWT validation and enforcement is handled by modkit-auth at the gateway level.

**Rationale**: Replaces auth_management.py and Node.js auth-proxy.
**Actors**: `cpt-cf-cricoai-actor-admin`

### 5.4 Market Data Collection

#### Kline collection

- [ ] `p1` - **ID**: `cpt-cf-cricoai-fr-kline-collection`

The system MUST collect OHLCV candlestick data from Binance REST API across intervals (1m, 5m, 15m, 1h, 4h) as a background service using `lifecycle(entry = "...")`. The service MUST use `CancellationToken` for graceful shutdown and retry with exponential backoff on failure.

**Rationale**: Replaces klines_unified_collector.py.
**Actors**: `cpt-cf-cricoai-actor-binance`

#### Real-time price SSE

- [ ] `p2` - **ID**: `cpt-cf-cricoai-fr-price-sse`

The system MUST broadcast price updates via SSE to subscribed frontend clients.

**Rationale**: Enables real-time price display without polling.
**Actors**: `cpt-cf-cricoai-actor-frontend`

### 5.5 Market Intelligence (AI Agents)

#### LLM-powered sentiment analysis

- [ ] `p1` - **ID**: `cpt-cf-cricoai-fr-sentiment-analysis`

The system MUST analyze market sentiment per trading symbol using cyberfabric-core's LLM Gateway (via ClientHub). Tier-based scheduling: hot symbols every 5m, warm every 15m, stable every 30m, cold every 1h. Decisions MUST be written to the `agent_decisions` table with confidence scores and decay.

**Rationale**: Replaces Python LangChain + Ollama agent system with native Rust module using LLM Gateway.
**Actors**: `cpt-cf-cricoai-actor-llm-gateway`

#### Agent decision SSE

- [ ] `p2` - **ID**: `cpt-cf-cricoai-fr-agent-decision-sse`

The system MUST broadcast agent decisions via SSE as they are produced.

**Rationale**: Real-time agent visibility for the trader.
**Actors**: `cpt-cf-cricoai-actor-frontend`

### 5.6 Data Ingestion

#### External sentiment collection

- [ ] `p1` - **ID**: `cpt-cf-cricoai-fr-data-ingestion`

The system MUST collect market sentiment data from CryptoPanic, CoinGecko, Reddit RSS, and Fear & Greed Index as a background service. Rate limiting per source to respect API quotas. Data written to `market_sentiment` table.

**Rationale**: Replaces data_ingestion_service.py with Rust using reqwest.
**Actors**: `cpt-cf-cricoai-actor-sentiment-sources`

### 5.7 Trading Core (Bot)

#### Automated trading loop

- [ ] `p1` - **ID**: `cpt-cf-cricoai-fr-trading-loop`

The system MUST run a configurable-interval trading loop as a background service. Buy strategy: iterate pairs, apply filters, get ML prediction (gRPC to ml-service via ClientHub), get sentiment (ClientHub to market-intelligence), evaluate decision pipeline, place order via Binance REST. Sell strategy: monitor open positions, check profit targets / stop-loss / trailing stops, check agent sentiment, execute exit.

**Rationale**: Core trading functionality. Replaces CricoAI_new.py and trade_logic modules.
**Actors**: `cpt-cf-cricoai-actor-binance`, `cpt-cf-cricoai-actor-ml-service`

#### Operational control

- [ ] `p2` - **ID**: `cpt-cf-cricoai-fr-trading-control`

The system MUST expose REST endpoints for bot status, pause, and resume. Write endpoints MUST support `Idempotency-Key`.

**Rationale**: Enables safe operational control without restarting.
**Actors**: `cpt-cf-cricoai-actor-trader`

#### Trade event SSE

- [ ] `p2` - **ID**: `cpt-cf-cricoai-fr-trade-event-sse`

The system MUST broadcast trade events (new orders, fills, P&L changes) via SSE.

**Rationale**: Real-time trade visibility for the trader.
**Actors**: `cpt-cf-cricoai-actor-frontend`

### 5.8 ML Service Bridge

#### gRPC bridge to Python ML

- [ ] `p1` - **ID**: `cpt-cf-cricoai-fr-ml-grpc-bridge`

The system MUST provide a Rust SDK-only crate (`ml-service-sdk`) with API trait, gRPC client, and wiring helpers following cyberfabric-core's OoP SDK pattern. The Python ML service implements the gRPC server. Consumer modules resolve the ML client via `ctx.client_hub().get::<dyn MlServiceApi>()`.

**Rationale**: ML stays in Python; Rust accesses it via type-safe gRPC bridge.
**Actors**: `cpt-cf-cricoai-actor-ml-service`

## 6. Non-Functional Requirements

### 6.1 Module-Specific NFRs

#### DNA API compliance

- [ ] `p1` - **ID**: `cpt-cf-cricoai-nfr-dna-compliance`

All REST endpoints MUST comply with DNA REST API standard: snake_case JSON, `{ items, page_info }` list envelope, cursor-based pagination (default 25, max 200), OData `$filter`/`$orderby`/`$select`, RFC 9457 Problem Details errors, `ETag`/`If-Match` on writes, `Idempotency-Key` on POST/PATCH/DELETE, OpenAPI 3.1 via utoipa.

**Threshold**: 100% endpoint compliance
**Rationale**: Ensures consistency with cyberfabric-core ecosystem
**Architecture Allocation**: See ADR-0003

#### Trading decision parity

- [ ] `p1` - **ID**: `cpt-cf-cricoai-nfr-trading-parity`

The Rust trading-core MUST produce identical buy/sell decisions to the Python trading bot when given the same inputs, validated by 2-week parallel testnet run.

**Threshold**: 100% decision match over 2 weeks on testnet
**Rationale**: Trading logic regression is the highest-severity risk

## 7. Public Library Interfaces

### 7.1 Public API Surface

#### Module SDK traits

- [ ] `p1` - **ID**: `cpt-cf-cricoai-interface-sdk-traits`

**Type**: Rust traits in `*-sdk` crates
**Stability**: unstable (pre-1.0)
**Description**: Each module exposes a public SDK trait (e.g., `TradingDashboardApi`, `ConfigManagerApi`, `MarketDataApi`) for inter-module communication via ClientHub. SDK crates are transport-agnostic — no serde, no axum, no HTTP types.
**Breaking Change Policy**: Coordinated within monorepo; no external consumers yet.

#### REST API

- [ ] `p1` - **ID**: `cpt-cf-cricoai-interface-rest-api`

**Type**: REST API (JSON)
**Stability**: unstable (pre-1.0, path-versioned as `/v1`)
**Description**: All 7 modules expose REST endpoints via cyberfabric-core's API gateway. OpenAPI 3.1 auto-generated via utoipa at `/v1/openapi.json`.
**Breaking Change Policy**: Path version bump (`/v2`) for breaking changes.

### 7.2 External Integration Contracts

#### ML Service gRPC

- [ ] `p1` - **ID**: `cpt-cf-cricoai-contract-ml-grpc`

**Direction**: required from Python ML service
**Protocol/Format**: gRPC (protobuf)
**Compatibility**: Proto file versioned in `ml-service-sdk` crate; backward-compatible field additions only.

## 8. Use Cases

#### View trading dashboard

- [ ] `p1` - **ID**: `cpt-cf-cricoai-usecase-view-dashboard`

**Actor**: `cpt-cf-cricoai-actor-trader`

**Preconditions**:
- Trader is authenticated via JWT
- cyberfabric-core server is running with trading-dashboard module

**Main Flow**:
1. Frontend sends GET request to `/trading-dashboard/v1/stats/summary`
2. modkit-auth validates JWT, extracts tenant_id into SecurityContext
3. trading-dashboard handler builds AccessScope from SecurityContext
4. SecureConn queries trading data scoped to tenant
5. Response returned as DNA-compliant JSON with `{ items, page_info }` for lists

**Postconditions**:
- Trader sees current P&L, open orders, recent trades

#### Execute automated trade

- [ ] `p1` - **ID**: `cpt-cf-cricoai-usecase-auto-trade`

**Actor**: `cpt-cf-cricoai-actor-binance`, `cpt-cf-cricoai-actor-ml-service`

**Preconditions**:
- trading-core module is running and not paused
- ML service is reachable via gRPC
- market-intelligence has recent sentiment data

**Main Flow**:
1. Trading loop iterates configured pairs
2. For each pair: query ml-service via ClientHub for prediction
3. Query market-intelligence via ClientHub for sentiment
4. Evaluate decision pipeline (filters, thresholds, risk checks)
5. If buy signal: place order via Binance REST API
6. Write order to database via SecureConn
7. Broadcast trade event via SSE

**Postconditions**:
- Order placed on Binance, recorded in DB, visible in dashboard

## 9. Acceptance Criteria

- [ ] All 7 modules boot within cyberfabric-core and pass health checks
- [ ] REST API returns DNA-compliant responses (verified by automated test suite)
- [ ] Trading decisions match Python bot over 2-week parallel testnet run
- [ ] HAI3 frontend renders all 6 screensets with correct data
- [ ] Container count is 4 (cyberfabric-core, ml-service, postgres, jaeger)
- [ ] OpenTelemetry traces flow end-to-end from frontend to database

## 10. Dependencies

| Dependency | Description | Criticality |
|------------|-------------|-------------|
| cyberfabric-core | Modular Rust framework (ModKit, API gateway, ClientHub, SecureConn) | p1 |
| HAI3 | React 19 frontend framework (screensets, API services, state) | p1 |
| PostgreSQL 16 | Existing database with trading data | p1 |
| Binance API | Exchange REST + WebSocket for market data and order execution | p1 |
| Python ML libraries | scikit-learn, XGBoost, LightGBM, ta-lib, pandas, numpy | p1 |
| LLM provider | Ollama (local) or external LLM for sentiment analysis | p2 |

## 11. Assumptions

- Existing PostgreSQL schema is stable and can be extended via SeaORM migrations (adding `tenant_id` columns)
- Binance API v3 remains available and stable
- cyberfabric-core's LLM Gateway supports the prompt patterns used by the current agent system
- HAI3 v0.1.0 core features (screensets, API services, state) are sufficiently stable for production use
- Single-operator deployment — no multi-user SaaS requirements

## 12. Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Trading logic regression during port | CRITICAL | 2-week parallel testnet validation comparing every buy/sell decision |
| Binance API client must be built from scratch in Rust | HIGH | Start REST-only, add WebSocket later. Keep Python fallback during trading-core phase. |
| Rust learning curve (beginner level) | MEDIUM | Start with read-only endpoints (Phase 1-2), build skills before trading core (Phase 7) |
| HAI3 is v0.1.0 alpha | MEDIUM | Core features (screensets, API services, state) are stable. Avoid roadmap-only features. |
| Existing DB schema needs tenant_id migration | MEDIUM | Add tenant_id columns via SeaORM migrations; backfill existing rows. See ADR-0001. |
| LLM Gateway prompt compatibility | LOW | Same prompts work — LLM Gateway is a provider abstraction, not a prompt framework |
| Each phase is independently valuable | — | Can stop after any phase with a working system |

## 13. Open Questions

- Should kline data (Binance_Klines) and model data (Model_Data) be consolidated into the main tenant databases, or kept as separate databases with cross-tenant access?
- What is the exact tenant_id assignment for production vs testnet? Static UUIDs or generated at first boot?
- Should the config-manager module support hot-reload of trading parameters without bot restart?

## 14. Traceability

- **Design**: [DESIGN.md](./DESIGN.md)
- **ADRs**: [ADR/](./ADR/)
- **Decomposition**: [DECOMPOSITION.md](./DECOMPOSITION.md)
- **Frontend**: [frontend.md](./features/frontend.md)
