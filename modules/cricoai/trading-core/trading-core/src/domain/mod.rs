//! Domain layer - Business logic and entities

pub mod service;
pub mod local_client;
pub mod error;
pub mod models;
pub mod entities;
pub mod strategy;
pub mod binance;
pub mod clients;
pub mod persistence;

pub use error::DomainError;
pub use service::TradingCoreService;
pub use local_client::LocalClient;
pub use strategy::{BuyManager, SellManager};
pub use binance::{BinanceApiClient, PlacedOrder};
pub use clients::ModuleClients;
