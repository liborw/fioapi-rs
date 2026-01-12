use crate::error::{ApiError, FioError};
use crate::models::{AccountInfo, Transaction, parse_account_info, parse_transactions};
use chrono::NaiveDate;
use log::{debug, info};
use reqwest::{Client as HttpClient, Response, StatusCode};
use std::fmt;
use std::time::Duration;

const BASE_URL: &str = "https://fioapi.fio.cz/v1/rest";
const TOKEN_LENGTH: usize = 64;

#[derive(Debug, Clone, Copy)]
pub enum TransactionReportFmt {
    Csv,
    Gpc,
    Html,
    Json,
    Ofx,
    Xml,
}

impl fmt::Display for TransactionReportFmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = match self {
            TransactionReportFmt::Csv => "csv",
            TransactionReportFmt::Gpc => "gpc",
            TransactionReportFmt::Html => "html",
            TransactionReportFmt::Json => "json",
            TransactionReportFmt::Ofx => "ofx",
            TransactionReportFmt::Xml => "xml",
        };
        f.write_str(v)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AccountStatementFmt {
    Csv,
    Gpc,
    Html,
    Json,
    Ofx,
    Xml,
    Pdf,
    Mt940,
    CbaXml,
    SbaXml,
}

impl fmt::Display for AccountStatementFmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = match self {
            AccountStatementFmt::Csv => "csv",
            AccountStatementFmt::Gpc => "gpc",
            AccountStatementFmt::Html => "html",
            AccountStatementFmt::Json => "json",
            AccountStatementFmt::Ofx => "ofx",
            AccountStatementFmt::Xml => "xml",
            AccountStatementFmt::Pdf => "pdf",
            AccountStatementFmt::Mt940 => "mt940",
            AccountStatementFmt::CbaXml => "cba_xml",
            AccountStatementFmt::SbaXml => "sba_xml",
        };
        f.write_str(v)
    }
}

#[derive(Debug, Clone)]
pub enum StatementData {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LastStatementInfo {
    pub year: i32,
    pub statement_id: i32,
}

#[derive(Debug, Clone)]
pub struct Client {
    token: String,
    http: HttpClient,
    base_url: String,
}

impl Client {
    /// Create a new client with the default base URL.
    pub fn new(token: impl Into<String>) -> Result<Self, FioError> {
        let token = token.into();
        if token.len() != TOKEN_LENGTH {
            return Err(FioError::InvalidTokenLength {
                expected: TOKEN_LENGTH,
                actual: token.len(),
            });
        }

        let http = HttpClient::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        info!("Initialized Fio API client with default base URL");
        Ok(Self {
            token,
            http,
            base_url: BASE_URL.to_string(),
        })
    }

    /// Override the base URL (useful for tests or proxies).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        info!("Updated Fio API base URL to {}", self.base_url);
        self
    }

    /// Fetch transaction report for the given period in the requested format.
    pub async fn fetch_transaction_report_for_period(
        &self,
        date_from: NaiveDate,
        date_to: NaiveDate,
        fmt: TransactionReportFmt,
    ) -> Result<String, FioError> {
        if date_from > date_to {
            return Err(FioError::InvalidDateRange {
                start: date_from,
                end: date_to,
            });
        }
        let path = format!(
            "/periods/{}/{}/{}/transactions.{}",
            self.token,
            date_from.format("%Y-%m-%d"),
            date_to.format("%Y-%m-%d"),
            fmt
        );
        debug!(
            "Fetching transaction report for period {} to {} as {}",
            date_from, date_to, fmt
        );
        self.get_text(path).await
    }

    /// Fetch account statement identified by year and statement id.
    pub async fn fetch_account_statement(
        &self,
        year: i32,
        statement_id: i64,
        fmt: AccountStatementFmt,
    ) -> Result<StatementData, FioError> {
        if statement_id < 0 {
            return Err(FioError::InvalidParameter(
                "statement_id must be a positive integer",
            ));
        }
        let path = format!(
            "/by-id/{}/{}/{}/transactions.{}",
            self.token, year, statement_id, fmt
        );
        debug!(
            "Fetching account statement year={} id={} as {}",
            year, statement_id, fmt
        );
        match fmt {
            AccountStatementFmt::Pdf => self.get_binary(path).await.map(StatementData::Binary),
            _ => self.get_text(path).await.map(StatementData::Text),
        }
    }

    /// Fetch transactions since the last successful download.
    pub async fn fetch_transaction_report_since_last_download(
        &self,
        fmt: TransactionReportFmt,
    ) -> Result<String, FioError> {
        let path = format!("/last/{}/transactions.{}", self.token, fmt);
        debug!("Fetching transaction report since last download as {}", fmt);
        self.get_text(path).await
    }

    /// Retrieve metadata about the last available account statement.
    pub async fn fetch_last_account_statement_info(&self) -> Result<LastStatementInfo, FioError> {
        let path = format!("/lastStatement/{}/statement", self.token);
        debug!("Fetching last account statement metadata");
        let body = self.get_text(path).await?;
        let mut parts = body.split(',');
        let year: i32 = parts
            .next()
            .and_then(|p| p.trim().parse().ok())
            .ok_or(FioError::InvalidResponse)?;
        let statement_id: i32 = parts
            .next()
            .and_then(|p| p.trim().parse().ok())
            .ok_or(FioError::InvalidResponse)?;
        Ok(LastStatementInfo { year, statement_id })
    }

    /// Set ID of last successfully downloaded transaction.
    pub async fn set_last_downloaded_transaction_id(
        &self,
        transaction_id: i64,
    ) -> Result<(), FioError> {
        if transaction_id < 0 {
            return Err(FioError::InvalidParameter(
                "transaction_id must be a positive integer",
            ));
        }
        let path = format!("/set-last-id/{}/{}/", self.token, transaction_id);
        info!(
            "Updating last downloaded transaction id to {}",
            transaction_id
        );
        self.get_void(path).await
    }

    /// Set date of last unsuccessful download.
    pub async fn set_last_unsuccessful_download_date(
        &self,
        download_date: NaiveDate,
    ) -> Result<(), FioError> {
        let path = format!(
            "/set-last-date/{}/{}/",
            self.token,
            download_date.format("%Y-%m-%d")
        );
        info!(
            "Updating last unsuccessful download date to {}",
            download_date
        );
        self.get_void(path).await
    }

    /// Parse account info from a JSON string returned by Fio API.
    pub fn parse_account_info(&self, data: &str) -> Result<AccountInfo, FioError> {
        parse_account_info(data)
    }

    /// Parse transactions from a JSON string returned by Fio API.
    pub fn parse_transactions(&self, data: &str) -> Result<Vec<Transaction>, FioError> {
        parse_transactions(data)
    }

    async fn get_text(&self, path: String) -> Result<String, FioError> {
        let response = self.get(path).await?;
        response.text().await.map_err(FioError::from)
    }

    async fn get_binary(&self, path: String) -> Result<Vec<u8>, FioError> {
        let response = self.get(path).await?;
        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(FioError::from)
    }

    async fn get_void(&self, path: String) -> Result<(), FioError> {
        self.get(path).await?;
        Ok(())
    }

    async fn get(&self, path: String) -> Result<Response, FioError> {
        let url = format!("{}{}", self.base_url, path);
        let redacted_path = path.replace(&self.token, "<token>");
        debug!("GET request to {}{}", self.base_url, redacted_path);
        let response = self.http.get(url).send().await?;
        debug!("Received status {}", response.status());
        self.handle_status(response.status())?;
        Ok(response)
    }

    fn handle_status(&self, status: StatusCode) -> Result<(), FioError> {
        if status.is_success() {
            return Ok(());
        }
        let api_error = match status {
            StatusCode::NOT_FOUND => ApiError::InvalidRequest,
            StatusCode::CONFLICT => ApiError::TimeLimit,
            StatusCode::PAYLOAD_TOO_LARGE => ApiError::TooManyItems,
            StatusCode::UNPROCESSABLE_ENTITY => ApiError::Authorization,
            StatusCode::INTERNAL_SERVER_ERROR => ApiError::InvalidToken,
            _ => ApiError::UnexpectedStatus(status),
        };
        Err(FioError::Api(api_error))
    }
}
