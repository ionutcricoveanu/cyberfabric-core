//! Market Intelligence Module
//!
//! LLM-powered sentiment analysis and market monitoring with tier-based scheduling.
//!
//! Features:
//! - Tier-based agent scheduling (HOT/WARM/STABLE/COLD with different intervals)
//! - Sentiment analysis via LLM (Ollama integration for local dev)
//! - Agent decision persistence to database
//! - REST API for querying decisions
//! - ClientHub registration for inter-module access

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod api;
pub mod config;
pub mod domain;
pub mod infra;
pub mod module;

pub use market_intelligence_sdk::{MarketIntelligenceApi, MarketIntelligenceError, AgentDecision};
