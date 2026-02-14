//! Data Ingestion SDK
//!
//! Public API for the `data-ingestion` module:
//! - `DataIngestionApi` trait
//! - Model types for sentiment entries
//! - Error type (`DataIngestionError`)

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod client;
pub mod errors;
pub mod models;

pub use client::DataIngestionApi;
pub use errors::DataIngestionError;
pub use models::SentimentEntry;
