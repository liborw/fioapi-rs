use chrono::Days;
use fioapi::{Client, TransactionReportFmt};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    env_logger::init();
    let token = env::var("FIO_API_TOKEN")
        .map_err(|_| "Set FIO_API_TOKEN in your environment or .env file")?;

    let client = Client::new(token)?;

    // Fetch transactions for the last 30 days in JSON form.
    let end = chrono::Utc::now().date_naive();
    let start = end
        .checked_sub_days(Days::new(29))
        .expect("valid start date window");

    let payload = client
        .fetch_transaction_report_for_period(start, end, TransactionReportFmt::Json)
        .await?;
    let transactions = client.parse_transactions(&payload)?;

    println!(
        "Fetched {} transactions from {} to {}:",
        transactions.len(),
        start,
        end
    );
    for txn in &transactions {
        println!(
            "{} | {} {} {}",
            txn.transaction_id, txn.date, txn.amount, txn.currency
        );
    }

    Ok(())
}
