//! Small, read-only client for the WSPR.live ClickHouse HTTP API.
//! WSPR.live permits only the `query` URL parameter and GET requests.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::time::Duration;

const ENDPOINT: &str = "https://db1.wspr.live/";
const DEFAULT_LIMIT: u32 = 500;
const MAX_LIMIT: u32 = 5_000;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid callsign: use 1-20 ASCII letters, digits, or /")]
    InvalidCallsign,
    #[error("start must be earlier than end")]
    InvalidTimeRange,
    #[error("limit must be between 1 and {MAX_LIMIT}")]
    InvalidLimit,
    #[error("WSPR.live request failed: {0}")]
    Http(#[from] reqwest::Error),
}

/// A report in which a receiver decoded a transmitter's WSPR signal.
#[derive(Debug, Clone, Deserialize)]
pub struct Reception {
    pub id: u64,
    /// WSPR.live returns time as a UTC-like ClickHouse DateTime string.
    pub time: String,
    pub band: i16,
    pub rx_sign: String,
    pub rx_loc: String,
    pub tx_sign: String,
    pub tx_loc: String,
    pub distance: u16,
    pub azimuth: u16,
    pub frequency: u64,
    pub power: i8,
    pub snr: i8,
    pub drift: i8,
}

#[derive(Debug, Clone)]
pub struct ReceptionQuery {
    pub callsign: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    /// WSPR.live band values: 14 = 20m, 7 = 40m, 28 = 10m, etc.
    pub band: Option<i16>,
    pub limit: u32,
}

impl ReceptionQuery {
    pub fn new(callsign: impl Into<String>, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { callsign: callsign.into(), start, end, band: None, limit: DEFAULT_LIMIT }
    }

    pub fn band(mut self, band: i16) -> Self {
        self.band = Some(band);
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    fn sql(&self) -> Result<String, Error> {
        let callsign = self.callsign.trim().to_ascii_uppercase();
        if callsign.is_empty()
            || callsign.len() > 20
            || !callsign.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'/')
        {
            return Err(Error::InvalidCallsign);
        }
        if self.start >= self.end {
            return Err(Error::InvalidTimeRange);
        }
        if !(1..=MAX_LIMIT).contains(&self.limit) {
            return Err(Error::InvalidLimit);
        }

        // Numeric UNIX UTC timestamps avoid any ambiguous local timezone parsing.
        // ClickHouse supports DateTime comparisons via fromUnixTimestamp().
        let mut sql = format!(
            "SELECT id, time, band, rx_sign, rx_loc, tx_sign, tx_loc, distance, azimuth, \
             frequency, power, snr, drift FROM wspr.rx WHERE rx_sign = '{callsign}' \
             AND time >= fromUnixTimestamp({}) AND time < fromUnixTimestamp({})",
            self.start.timestamp(), self.end.timestamp()
        );
        if let Some(band) = self.band {
            sql.push_str(&format!(" AND band = {band}"));
        }
        sql.push_str(&format!(" ORDER BY time DESC LIMIT {} FORMAT JSON", self.limit));
        Ok(sql)
    }
}

#[derive(Debug, Deserialize)]
struct ClickHouseResponse {
    data: Vec<Reception>,
}

#[derive(Clone, Debug)]
pub struct WsprLiveClient {
    http: reqwest::Client,
}

impl WsprLiveClient {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("wspr-live-client/0.1")
                .build()?,
        })
    }

    /// Fetch spots received by `query.callsign` in the half-open UTC interval [start, end).
    pub async fn receptions(&self, query: &ReceptionQuery) -> Result<Vec<Reception>, Error> {
        let sql = query.sql()?;
        let response = self.http.get(ENDPOINT).query(&[("query", sql)]).send().await?
            .error_for_status()?;
        Ok(response.json::<ClickHouseResponse>().await?.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample() -> ReceptionQuery {
        ReceptionQuery::new("n9iox", Utc.timestamp_opt(1_700_000_000, 0).unwrap(),
            Utc.timestamp_opt(1_700_003_600, 0).unwrap())
    }

    #[test]
    fn receiver_sql_has_time_and_callsign_filters() {
        let sql = sample().band(14).limit(100).sql().unwrap();
        assert!(sql.contains("rx_sign = 'N9IOX'"));
        assert!(sql.contains("time >= fromUnixTimestamp(1700000000)"));
        assert!(sql.contains("time < fromUnixTimestamp(1700003600)"));
        assert!(sql.contains("band = 14"));
        assert!(sql.contains("LIMIT 100 FORMAT JSON"));
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert!(matches!(ReceptionQuery::new("N9IOX' OR 1=1", sample().start, sample().end).sql(), Err(Error::InvalidCallsign)));
        assert!(matches!(sample().limit(0).sql(), Err(Error::InvalidLimit)));
        let q = sample();
        assert!(matches!(ReceptionQuery::new("N9IOX", q.end, q.start).sql(), Err(Error::InvalidTimeRange)));
    }
}
