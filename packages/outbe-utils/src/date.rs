use chrono::NaiveDate;
use cosmwasm_std::Timestamp;
use thiserror::Error;

const SECONDS_IN_DAY: u64 = 86400;

/// Normalize any timestamp to midnight UTC of that day.
pub fn normalize_to_date(timestamp: &Timestamp) -> WorldwideDay {
    let seconds = timestamp.seconds();
    let days = seconds / SECONDS_IN_DAY;
    days * SECONDS_IN_DAY
}

/// Worldwide day in seconds
pub type WorldwideDay = u64;

/// ISO 8601 Date format string
pub type Iso8601Date = String;

pub fn iso_to_ts(date: &Iso8601Date) -> Result<WorldwideDay, DateError> {
    match NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d") {
        Ok(parsed_date) => {
            let timestamp_seconds = parsed_date
                .and_hms_opt(0, 0, 0)
                .ok_or(DateError::InvalidDateFormat {})?
                .and_utc()
                .timestamp();
            if timestamp_seconds < 0 {
                return Err(DateError::InvalidDateFormat {});
            }
            Ok(timestamp_seconds.unsigned_abs())
        }
        Err(_) => Err(DateError::InvalidDateFormat {}),
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum DateError {
    #[error("Invalid date format")]
    InvalidDateFormat {},
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iso_to_ts_valid() {
        let result = iso_to_ts(&"2024-01-15".to_string()).unwrap();
        assert_eq!(result, 1705276800);
    }

    #[test]
    fn test_iso_to_ts_invalid_format() {
        let result = iso_to_ts(&"15-01-2024".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_iso_to_ts_negative_date() {
        let result = iso_to_ts(&"1800-01-01".to_string());
        assert!(result.is_err());
    }
}
