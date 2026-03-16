//! ML Service SDK
//!
//! Public API for the Python ML service gRPC bridge:
//! - `MlServiceApi` trait
//! - Model types for predictions, indicators, regime
//! - Error type (`MlServiceError`)
//!
//! This is an SDK-only crate (no module implementation crate).
//! The server is Python; this crate provides the Rust gRPC client.

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod client;
pub mod errors;
pub mod models;
pub mod grpc;

pub use client::MlServiceApi;
pub use errors::MlServiceError;
pub use models::{MarketRegime, Prediction, TechnicalIndicators};
pub use grpc::MlServiceClient;

// Include generated proto code
pub mod ml_service {
    tonic::include_proto!("ml_service");
}
