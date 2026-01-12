use chrono::NaiveDate;
use clap::{Parser, Subcommand, ValueEnum};
use fioapi::{AccountStatementFmt, Client, LastStatementInfo, StatementData, TransactionReportFmt};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "fioapi-cli", about = "CLI wrapper for the Fio banka API")]
struct Cli {
    /// API token; falls back to FIO_API_TOKEN env var
    #[arg(long, env = "FIO_API_TOKEN")]
    token: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Fetch transactions for a date range (inclusive)
    FetchPeriod {
        /// Start date YYYY-MM-DD
        #[arg(long, value_parser = parse_date)]
        start: NaiveDate,
        /// End date YYYY-MM-DD
        #[arg(long, value_parser = parse_date)]
        end: NaiveDate,
        /// Output format
        #[arg(long, value_enum, default_value = "json")]
        format: TxnFmt,
    },
    /// Fetch transactions since last successful download
    FetchLast {
        #[arg(long, value_enum, default_value = "json")]
        format: TxnFmt,
    },
    /// Fetch account statement by year and statement id
    FetchStatement {
        #[arg(long)]
        year: i32,
        #[arg(long, value_name = "ID")]
        statement_id: i64,
        #[arg(long, value_enum, default_value = "json")]
        format: StatementFmt,
        /// Output file when requesting PDF
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Show year and ID of last account statement
    LastInfo,
    /// Set last downloaded transaction ID
    SetLastId {
        #[arg(long, value_name = "ID")]
        transaction_id: i64,
    },
    /// Set last unsuccessful download date
    SetLastDate {
        #[arg(long, value_parser = parse_date)]
        date: NaiveDate,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TxnFmt {
    Csv,
    Gpc,
    Html,
    Json,
    Ofx,
    Xml,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum StatementFmt {
    Csv,
    Gpc,
    Html,
    Json,
    Ofx,
    Xml,
    Pdf,
    Mt940,
    #[value(name = "cba_xml")]
    CbaXml,
    #[value(name = "sba_xml")]
    SbaXml,
}

fn parse_date(raw: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(raw, "%Y-%m-%d").map_err(|e| e.to_string())
}

impl From<TxnFmt> for TransactionReportFmt {
    fn from(value: TxnFmt) -> Self {
        match value {
            TxnFmt::Csv => TransactionReportFmt::Csv,
            TxnFmt::Gpc => TransactionReportFmt::Gpc,
            TxnFmt::Html => TransactionReportFmt::Html,
            TxnFmt::Json => TransactionReportFmt::Json,
            TxnFmt::Ofx => TransactionReportFmt::Ofx,
            TxnFmt::Xml => TransactionReportFmt::Xml,
        }
    }
}

impl From<StatementFmt> for AccountStatementFmt {
    fn from(value: StatementFmt) -> Self {
        match value {
            StatementFmt::Csv => AccountStatementFmt::Csv,
            StatementFmt::Gpc => AccountStatementFmt::Gpc,
            StatementFmt::Html => AccountStatementFmt::Html,
            StatementFmt::Json => AccountStatementFmt::Json,
            StatementFmt::Ofx => AccountStatementFmt::Ofx,
            StatementFmt::Xml => AccountStatementFmt::Xml,
            StatementFmt::Pdf => AccountStatementFmt::Pdf,
            StatementFmt::Mt940 => AccountStatementFmt::Mt940,
            StatementFmt::CbaXml => AccountStatementFmt::CbaXml,
            StatementFmt::SbaXml => AccountStatementFmt::SbaXml,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    dotenvy::dotenv().ok();

    let cli = Cli::parse();
    let client = Client::new(cli.token)?;

    match cli.command {
        Commands::FetchPeriod { start, end, format } => {
            let fmt = format.into();
            let payload = client
                .fetch_transaction_report_for_period(start, end, fmt)
                .await?;
            print_text(payload);
        }
        Commands::FetchLast { format } => {
            let fmt = format.into();
            let payload = client
                .fetch_transaction_report_since_last_download(fmt)
                .await?;
            print_text(payload);
        }
        Commands::FetchStatement {
            year,
            statement_id,
            format,
            output,
        } => {
            let fmt = format.into();
            let data = client
                .fetch_account_statement(year, statement_id, fmt)
                .await?;
            handle_statement_output(data, fmt, output)?;
        }
        Commands::LastInfo => {
            let info: LastStatementInfo = client.fetch_last_account_statement_info().await?;
            println!("year={}, statement_id={}", info.year, info.statement_id);
        }
        Commands::SetLastId { transaction_id } => {
            client
                .set_last_downloaded_transaction_id(transaction_id)
                .await?;
            println!("Set last downloaded transaction id to {}", transaction_id);
        }
        Commands::SetLastDate { date } => {
            client.set_last_unsuccessful_download_date(date).await?;
            println!("Set last unsuccessful download date to {}", date);
        }
    }

    Ok(())
}

fn print_text(payload: String) {
    println!("{payload}");
}

fn handle_statement_output(
    data: StatementData,
    fmt: AccountStatementFmt,
    output: Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    match data {
        StatementData::Text(text) => {
            print!("{text}");
        }
        StatementData::Binary(bytes) => {
            let path = output.ok_or("Output path required for binary formats (e.g., PDF)")?;
            fs::write(&path, &bytes)?;
            println!(
                "Wrote {} bytes to {} ({})",
                bytes.len(),
                path.display(),
                match fmt {
                    AccountStatementFmt::Pdf => "PDF",
                    _ => "binary",
                }
            );
        }
    }
    Ok(())
}
