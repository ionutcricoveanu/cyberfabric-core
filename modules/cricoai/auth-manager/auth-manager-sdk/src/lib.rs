//! Auth Manager SDK
//!
//! Public API for the `auth-manager` module:
//! - `AuthManagerApi` trait
//! - Model types for users, roles, audit logs
//! - Error type (`AuthManagerError`)

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod client;
pub mod errors;
pub mod models;

pub use client::AuthManagerApi;
pub use errors::AuthManagerError;
pub use models::User;
