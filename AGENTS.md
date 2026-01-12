# Repository Guidelines

## Project Structure & Module Organization
- Rust crate lives in `src/`: `lib.rs` re-exports `client`, `error`, and `models`; `client.rs` should encapsulate HTTP calls; `models.rs` holds serde models and domain conversions; `error.rs` defines `FioError`; `main.rs` is a placeholder binary.
- `fio_banka.py` contains a Python reference client and examples of expected API behavior; mirror its semantics when filling out the Rust client.
- `API_Bankovnictvi.pdf` documents the Fio API; keep it nearby when mapping columns/fields. Place new fixtures under `tests/fixtures/` if you add integration tests.

## Build, Test, and Development Commands
- `cargo fmt` formats the Rust sources with rustfmt defaults.
- `cargo clippy --all-targets --all-features` lints for common mistakes and style issues.
- `cargo test` runs unit and integration tests; add `-- --nocapture` to see test output.
- `cargo run --bin fioapi` runs the current binary entry point (useful for manual experiments while the client is developed).
- For Python reference work: `python -m venv .venv && source .venv/bin/activate && pip install requests` then import `fio_banka` in a REPL to compare behaviors.

## Coding Style & Naming Conventions
- Rust: keep rustfmt defaults (4-space indent); modules/files are `snake_case`, types `UpperCamelCase`, functions and fields `snake_case`. Prefer `?` for error propagation and return `Result<_, FioError>`.
- Serde mappings: keep `#[serde(rename = "...")]` aligned with API column numbers; use `Decimal` for currency and `NaiveDate` for dates.
- Python snippets should follow PEP 8 and reuse existing enums/NamedTuples.

## Testing Guidelines
- Use `#[cfg(test)]` modules alongside implementations for unit coverage; add integration tests under `tests/` when exercising HTTP-level flows.
- Favor static JSON fixtures over live API calls; validate parsed transactions (id/date/amount/currency) and error paths (invalid token, status handling).

## Commit & Pull Request Guidelines
- No commit history yet; use Conventional Commits going forward (e.g., `feat: add account statement parsing`, `fix: handle invalid token response`).
- Pull requests should describe scope and rationale, note any API surface changes, list commands/tests run, and include sample payloads or logs when debugging HTTP issues.

## Security & Configuration Tips
- Never commit API tokens; pass them via env vars (e.g., `FIO_API_TOKEN`) or local `.env` files kept out of version control.
- Respect request timeouts and rate limits noted in the API PDF; avoid hard-coding secrets or URLs in tests.***
