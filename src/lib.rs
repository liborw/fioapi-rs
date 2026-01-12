//! Rust client for the Fio banka REST API.
//! Provides a small synchronous interface and helpers for parsing JSON
//! statements into typed domain models.

pub mod client;
pub mod error;
pub mod models;

pub use client::{
    AccountStatementFmt, Client, LastStatementInfo, StatementData, TransactionReportFmt,
};
pub use error::{ApiError, FioError};
pub use models::{AccountInfo, Transaction};
