# fioapi

Async Rust client for the Fio banka REST API. It mirrors the official API (see `API_Bankovnictvi.pdf`) and the bundled Python reference (`fio_banka.py`), exposing typed models, error mapping, and helpers for parsing JSON statements.

## Features
- Async reqwest client with explicit format enums for transaction reports and account statements.
- Typed models for account info and transactions with serde column mapping.
- Error types that map HTTP status codes to domain errors.
- Helpers to parse JSON payloads into domain types without hitting the network.

## Installation
```toml
[dependencies]
fioapi = { path = "." }
```
Requires Rust 1.74+ and Tokio runtime.

## Usage
```rust
use fioapi::{Client, TransactionReportFmt};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenvy::dotenv().ok();
    let token = env::var("FIO_API_TOKEN")?;
    let client = Client::new(token)?;

    let payload = client
        .fetch_transaction_report_since_last_download(TransactionReportFmt::Json)
        .await?;
    let transactions = client.parse_transactions(&payload)?;

    for txn in transactions {
        println!("{} | {} {} {}", txn.transaction_id, txn.date, txn.amount, txn.currency);
    }
    Ok(())
}
```

## Examples
- `cargo run --example fetch_transactions` reads `FIO_API_TOKEN` from environment or `.env` and prints transactions from the last 30 days.
- `cargo run --example set_last_unsuccessful_download_date [YYYY-MM-DD]` sets the last unsuccessful download date (defaults to yesterday when no date is provided).

## Development
- Format and lint: `cargo fmt`, `cargo clippy --all-targets --all-features`
- Tests: `cargo test`

Keep secrets (API tokens) out of VCS; pass them via env vars or `.env`. Default base URL is `https://fioapi.fio.cz/v1/rest`; override with `Client::with_base_url` for testing.***
