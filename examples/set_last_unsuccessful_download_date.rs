use chrono::Days;
use chrono::NaiveDate;
use fioapi::Client;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenvy::dotenv().ok();

    let token = env::var("FIO_API_TOKEN")
        .map_err(|_| "Set FIO_API_TOKEN in your environment or .env file")?;
    let client = Client::new(token)?;

    // Default to yesterday; optionally override via YYYY-MM-DD arg.
    let date = match env::args().nth(1) {
        Some(arg) => NaiveDate::parse_from_str(&arg, "%Y-%m-%d")?,
        None => chrono::Utc::now()
            .date_naive()
            .checked_sub_days(Days::new(1))
            .expect("valid date"),
    };

    client.set_last_unsuccessful_download_date(date).await?;
    println!("Updated last unsuccessful download date to {}", date);

    Ok(())
}
