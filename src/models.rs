use crate::error::FioError;
use chrono::NaiveDate;
use log::debug;
use rust_decimal::Decimal;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct AccountInfo {
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
    #[serde(rename = "bankId")]
    pub bank_id: Option<String>,
    pub currency: Option<String>,
    pub iban: Option<String>,
    pub bic: Option<String>,
    #[serde(rename = "openingBalance")]
    pub opening_balance: Option<Decimal>,
    #[serde(rename = "closingBalance")]
    pub closing_balance: Option<Decimal>,
    #[serde(rename = "dateStart", deserialize_with = "deserialize_date_opt")]
    pub date_start: Option<NaiveDate>,
    #[serde(rename = "dateEnd", deserialize_with = "deserialize_date_opt")]
    pub date_end: Option<NaiveDate>,
    #[serde(rename = "yearList")]
    pub year_list: Option<i32>,
    #[serde(rename = "idList")]
    pub id_list: Option<i32>,
    #[serde(rename = "idFrom")]
    pub id_from: Option<i64>,
    #[serde(rename = "idTo")]
    pub id_to: Option<i64>,
    #[serde(rename = "idLastDownload")]
    pub id_last_download: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub transaction_id: String,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub currency: String,
    pub account_id: Option<String>,
    pub account_name: Option<String>,
    pub bank_id: Option<String>,
    pub bank_name: Option<String>,
    pub ks: Option<String>,
    pub vs: Option<String>,
    pub ss: Option<String>,
    pub user_identification: Option<String>,
    pub remittance_info: Option<String>,
    pub transaction_type: Option<String>,
    pub executor: Option<String>,
    pub specification: Option<String>,
    pub comment: Option<String>,
    pub bic: Option<String>,
    pub order_id: Option<i64>,
    pub payer_reference: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FioResponse {
    #[serde(rename = "accountStatement")]
    pub account_statement: AccountStatement,
}

impl FioResponse {
    pub fn account_info(&self) -> &AccountInfo {
        &self.account_statement.info
    }

    pub fn transactions(&self) -> Result<Vec<Transaction>, FioError> {
        self.account_statement
            .transaction_list
            .transaction
            .iter()
            .map(Transaction::try_from)
            .collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct AccountStatement {
    pub info: AccountInfo,

    #[serde(rename = "transactionList")]
    pub transaction_list: TransactionList,
}

#[derive(Debug, Deserialize)]
pub struct TransactionList {
    pub(crate) transaction: Vec<RawTransaction>,
}

#[derive(Debug, Deserialize)]
struct ColumnValue<T> {
    value: T,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawTransaction {
    #[serde(rename = "column22")]
    transaction_id: ColumnValue<Value>,

    #[serde(rename = "column0")]
    date: ColumnValue<String>,

    #[serde(rename = "column1")]
    amount: ColumnValue<Decimal>,

    #[serde(rename = "column14")]
    currency: ColumnValue<String>,

    #[serde(rename = "column2")]
    account_id: Option<ColumnValue<Value>>,

    #[serde(rename = "column10")]
    account_name: Option<ColumnValue<Value>>,

    #[serde(rename = "column3")]
    bank_id: Option<ColumnValue<Value>>,

    #[serde(rename = "column12")]
    bank_name: Option<ColumnValue<Value>>,

    #[serde(rename = "column4")]
    ks: Option<ColumnValue<Value>>,

    #[serde(rename = "column5")]
    vs: Option<ColumnValue<Value>>,

    #[serde(rename = "column6")]
    ss: Option<ColumnValue<Value>>,

    #[serde(rename = "column7")]
    user_identification: Option<ColumnValue<Value>>,

    #[serde(rename = "column16")]
    remittance_info: Option<ColumnValue<Value>>,

    #[serde(rename = "column8")]
    transaction_type: Option<ColumnValue<Value>>,

    #[serde(rename = "column9")]
    executor: Option<ColumnValue<Value>>,

    #[serde(rename = "column18")]
    specification: Option<ColumnValue<Value>>,

    #[serde(rename = "column25")]
    comment: Option<ColumnValue<Value>>,

    #[serde(rename = "column26")]
    bic: Option<ColumnValue<Value>>,

    #[serde(rename = "column17")]
    order_id: Option<ColumnValue<Value>>,

    #[serde(rename = "column27")]
    payer_reference: Option<ColumnValue<Value>>,
}

fn deserialize_date_opt<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: Option<String> = Option::deserialize(deserializer)?;
    match raw {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => parse_date(&s)
            .map(Some)
            .ok_or_else(|| D::Error::custom("invalid date value")),
    }
}

fn parse_date(raw: &str) -> Option<NaiveDate> {
    let prefix = raw.get(0..10)?;
    NaiveDate::parse_from_str(prefix, "%Y-%m-%d").ok()
}

fn json_value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn optional_string(value: &Option<ColumnValue<Value>>) -> Option<String> {
    value
        .as_ref()
        .and_then(|c| json_value_to_string(&c.value))
        .filter(|s| !s.is_empty())
}

fn optional_i64(value: &Option<ColumnValue<Value>>) -> Option<i64> {
    value.as_ref().and_then(|c| c.value.as_i64())
}

impl TryFrom<&RawTransaction> for Transaction {
    type Error = FioError;

    fn try_from(raw: &RawTransaction) -> Result<Self, Self::Error> {
        let transaction_id =
            json_value_to_string(&raw.transaction_id.value).ok_or(FioError::InvalidResponse)?;
        let date = parse_date(&raw.date.value).ok_or(FioError::InvalidResponse)?;

        Ok(Transaction {
            transaction_id,
            date,
            amount: raw.amount.value,
            currency: raw.currency.value.clone(),
            account_id: optional_string(&raw.account_id),
            account_name: optional_string(&raw.account_name),
            bank_id: optional_string(&raw.bank_id),
            bank_name: optional_string(&raw.bank_name),
            ks: optional_string(&raw.ks),
            vs: optional_string(&raw.vs),
            ss: optional_string(&raw.ss),
            user_identification: optional_string(&raw.user_identification),
            remittance_info: optional_string(&raw.remittance_info),
            transaction_type: optional_string(&raw.transaction_type),
            executor: optional_string(&raw.executor),
            specification: optional_string(&raw.specification),
            comment: optional_string(&raw.comment),
            bic: optional_string(&raw.bic),
            order_id: optional_i64(&raw.order_id),
            payer_reference: optional_string(&raw.payer_reference),
        })
    }
}

pub fn parse_account_info(data: &str) -> Result<AccountInfo, FioError> {
    let parsed: FioResponse = serde_json::from_str(data).map_err(|_| FioError::InvalidResponse)?;
    debug!("Parsed account info");
    Ok(parsed.account_statement.info)
}

pub fn parse_transactions(data: &str) -> Result<Vec<Transaction>, FioError> {
    let parsed: FioResponse = serde_json::from_str(data).map_err(|_| FioError::InvalidResponse)?;
    let txns = parsed.transactions()?;
    debug!("Parsed {} transactions", txns.len());
    Ok(txns)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::str::FromStr;

    fn sample_payload() -> String {
        let payload = json!({
            "accountStatement": {
                "info": {
                    "accountId": "2000000000",
                    "bankId": "2010",
                    "currency": "CZK",
                    "iban": "CZ1000000000002000000000",
                    "bic": "FIOZSKBA",
                    "openingBalance": "100.00",
                    "closingBalance": "200.00",
                    "dateStart": "2023-01-01+0000",
                    "dateEnd": "2023-01-02+0000",
                    "yearList": 2023,
                    "idList": 1,
                    "idFrom": 123,
                    "idTo": 124,
                    "idLastDownload": 124
                },
                "transactionList": {
                    "transaction": [
                        {
                            "column22": { "value": 10001 },
                            "column0": { "value": "2023-01-02+0000" },
                            "column1": { "value": "50.25" },
                            "column14": { "value": "CZK" },
                            "column2": { "value": "123456789" },
                            "column10": { "value": "John Doe" },
                            "column3": { "value": "2010" },
                            "column12": { "value": "Fio banka" },
                            "column4": { "value": "0558" },
                            "column5": { "value": "12345" },
                            "column6": { "value": "001" },
                            "column7": { "value": "user info" },
                            "column16": { "value": "payment" },
                            "column8": { "value": "type" },
                            "column9": { "value": "executor" },
                            "column18": { "value": "spec" },
                            "column25": { "value": "comment" },
                            "column26": { "value": "BICCODE" },
                            "column17": { "value": 77 },
                            "column27": { "value": "payer" }
                        }
                    ]
                }
            }
        });
        payload.to_string()
    }

    #[test]
    fn parses_account_info() {
        let json = sample_payload();
        let info = parse_account_info(&json).expect("info should parse");
        assert_eq!(info.account_id.as_deref(), Some("2000000000"));
        assert_eq!(info.currency.as_deref(), Some("CZK"));
        assert_eq!(info.date_start, NaiveDate::from_ymd_opt(2023, 1, 1));
    }

    #[test]
    fn parses_transactions() {
        let json = sample_payload();
        let txns = parse_transactions(&json).expect("transactions should parse");
        assert_eq!(txns.len(), 1);
        let txn = &txns[0];
        assert_eq!(txn.transaction_id, "10001");
        assert_eq!(txn.amount, Decimal::from_str("50.25").unwrap());
        assert_eq!(txn.date, NaiveDate::from_ymd_opt(2023, 1, 2).unwrap());
        assert_eq!(txn.vs.as_deref(), Some("12345"));
        assert_eq!(txn.order_id, Some(77));
    }
}
