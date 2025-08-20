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
pub fn is_valid(date: &WorldwideDay) -> Result<(), DateError> {
    if date % SECONDS_IN_DAY == 0 {
        Ok(())
    } else {
        Err(DateError::InvalidDate {})
    }
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
                .ok_or(DateError::InvalidDate {})?
                .and_utc()
                .timestamp();
            if timestamp_seconds < 0 {
                return Err(DateError::InvalidDate {});
            }
            Ok(timestamp_seconds.unsigned_abs())
        }
        Err(_) => Err(DateError::InvalidDate {}),
    }
}

const EPOCH: NaiveDate = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();

pub fn iso_to_days(date: &Iso8601Date) -> Result<u64, DateError> {
    match NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d") {
        Ok(parsed_date) => {
            // Define the EPOCH as January 1, 2025

            // Calculate days since EPOCH
            let days_since_epoch = parsed_date.signed_duration_since(EPOCH).num_days();

            // Ensure the result is non-negative
            if days_since_epoch < 0 {
                return Err(DateError::DateBeforeEpoch {});
            }
            Ok(days_since_epoch as u64)
        }
        Err(_) => Err(DateError::InvalidDate {}),
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum DateError {
    #[error("Invalid date")]
    InvalidDate {},
    #[error("Date before EPOCH")]
    DateBeforeEpoch {},
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

    #[test]
    fn test_iso_to_days_success() {
        let result = iso_to_days(&"2025-01-01".to_string()).unwrap();
        assert_eq!(result, 0); // Epoch date

        let result = iso_to_days(&"2025-01-02".to_string()).unwrap();
        assert_eq!(result, 1);

        let result = iso_to_days(&"2026-01-01".to_string()).unwrap();
        assert_eq!(result, 365);
    }

    #[test]
    fn test_iso_to_days_invalid_format() {
        let result = iso_to_days(&"01-01-2025".to_string());
        assert!(result.is_err());

        let result = iso_to_days(&"2025/01/01".to_string());
        assert!(result.is_err());

        let result = iso_to_days(&"invalid".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_iso_to_days_before_epoch() {
        let result = iso_to_days(&"2024-12-31".to_string());
        assert!(result.is_err());

        let result = iso_to_days(&"2020-01-01".to_string());
        assert!(result.is_err());
    }
}
