//! Data Ingestion Module
//!
//! Handles continuous collection of external data from multiple sources:
//! - Reddit RSS feeds (sentiment analysis)
//! - CoinGecko (market trends, global data)
//! - Fear & Greed Index (market sentiment)
//! - On-chain data (whale transactions, exchange flows)

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod api;
pub mod config;
pub mod domain;
pub mod infra;
pub mod module;

pub use data_ingestion_sdk::{DataIngestionApi, DataIngestionError, SentimentEntry};
