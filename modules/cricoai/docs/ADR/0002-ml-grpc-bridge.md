# ADR-0002: ML Service gRPC Bridge — SDK-Only Crate Pattern

**ID**: `cpt-cf-cricoai-adr-ml-grpc-bridge`

## Status

Accepted

## Context and Problem Statement

CricoAI's ML pipeline (scikit-learn, XGBoost, LightGBM, ta-lib, pandas, numpy) has no Rust equivalents. The ML code must stay in Python permanently. The Rust trading-core module needs to call ML predictions as part of its decision pipeline.

We need a bridge that:
- Gives Rust modules type-safe access to Python ML predictions
- Follows cyberfabric-core's inter-module communication patterns
- Allows the ML service to evolve independently
- Handles connection failures gracefully

## Decides For Requirements

- `cpt-cf-cricoai-fr-ml-grpc-bridge` — type-safe gRPC bridge
- `cpt-cf-cricoai-fr-trading-loop` — trading-core needs ML predictions

## Decision Drivers

- cyberfabric-core's OoP module pattern uses SDK crates with gRPC clients (see `docs/modkit_unified_system/09_oop_grpc_sdk_pattern.md`)
- ClientHub provides type-safe resolution by trait — consumers should not know the transport
- The Python service is external — it has no Rust module implementation crate
- `modkit_transport_grpc::client` provides `connect_with_stack` and `connect_with_retry` utilities

## Considered Options

### Option A: SDK-only crate (no module implementation)

Create `ml-service-sdk` with API trait, proto file, gRPC client, and wiring helpers. No `ml-service` module crate — the server is Python. The `trading-core` module wires the gRPC client and registers it in ClientHub during `init()`.

**Pros:**
- Follows cyberfabric-core's OoP SDK pattern exactly
- Consumers use `ctx.client_hub().get::<dyn MlServiceApi>()` — transport-agnostic
- Proto file versioned in SDK crate alongside the Rust client
- Minimal Rust code — just the client side

**Cons:**
- No module lifecycle management for the Python process (must be managed externally)
- Proto changes require coordinated updates between Rust SDK and Python server

### Option B: Full OoP module with process management

Create both `ml-service-sdk` and `ml-service` module crate. The module crate manages the Python process lifecycle (start/stop/health) and hosts the gRPC client registration.

**Pros:**
- cyberfabric-core manages the Python process lifecycle
- Health monitoring integrated into module orchestrator

**Cons:**
- Over-engineering — the Python service is a Docker container managed by Docker Compose
- Module crate would be mostly boilerplate with no business logic
- Process management adds complexity for no clear benefit in a container environment

### Option C: Direct reqwest HTTP calls (no gRPC)

Expose Python ML as a REST API instead of gRPC. Call it with `modkit-http` from Rust.

**Pros:**
- No proto compilation needed
- Simpler tooling

**Cons:**
- No streaming support for future needs
- Weaker type safety — JSON vs protobuf
- REST overhead for high-frequency prediction calls
- Does not follow cyberfabric-core's OoP pattern

## Decision Outcome

**Chosen option: Option A (SDK-only crate)**

Create `ml-service-sdk` as a standalone SDK crate with no corresponding module implementation. The Python gRPC server is managed as a Docker container. The `trading-core` module wires the gRPC client in its `init()` and registers it in ClientHub.

**Crate structure:**
```
modules/ml-service/
└── ml-service-sdk/
    ├── Cargo.toml
    ├── build.rs                    # tonic-build proto compilation
    ├── proto/
    │   └── ml_service.v1.proto
    └── src/
        ├── lib.rs                  # Re-exports
        ├── api.rs                  # MlServiceApi trait + types + errors
        ├── client.rs               # MlServiceGrpcClient (implements MlServiceApi)
        └── wiring.rs               # wire_client(), build_client()
```

**Registration pattern:**
```rust
// In trading-core's init()
let ml_client = ml_service_sdk::wire_client(&config.ml_service_endpoint).await?;
ctx.client_hub().register::<dyn ml_service_sdk::MlServiceApi>(ml_client);
```

**Consumer pattern:**
```rust
// Anywhere in trading-core's domain logic
let ml = ctx.client_hub().get::<dyn ml_service_sdk::MlServiceApi>()?;
let prediction = ml.get_prediction("BTCUSDT", "1h").await?;
```

## Consequences

**Positive:**
- Standard cyberfabric-core OoP pattern — no custom integration code
- Type-safe at both Rust and Python boundaries (protobuf contract)
- Transport-transparent — consumers use the SDK trait, not gRPC directly
- gRPC client uses `modkit_transport_grpc::client` utilities (retries, timeouts, tracing)

**Negative:**
- Python process lifecycle managed by Docker Compose, not cyberfabric-core
- Proto file is the contract boundary — changes require updating both Rust SDK and Python stubs
- No automatic health monitoring from module orchestrator (use Docker health checks instead)

## Confirmation

- `ml-service-sdk` compiles with `cargo build`
- `MlServiceGrpcClient` implements `MlServiceApi` trait
- `wire_client()` connects to Python gRPC server and returns valid predictions
- Integration test: start Python server, call from Rust, verify response
- `trading-core` resolves ML client via `ctx.client_hub().get::<dyn MlServiceApi>()`

## Traceability

- **PRD**: [PRD.md](../PRD.md) — `cpt-cf-cricoai-fr-ml-grpc-bridge`, `cpt-cf-cricoai-contract-ml-grpc`
- **DESIGN**: [DESIGN.md](../DESIGN.md) — §3.2 ML Service (SDK-Only Crate)
- **Reference**: `cyberfabric-core/docs/modkit_unified_system/09_oop_grpc_sdk_pattern.md`
