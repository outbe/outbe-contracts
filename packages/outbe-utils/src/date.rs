use chrono::{Datelike, NaiveDate};
use cosmwasm_std::Timestamp;
use thiserror::Error;

pub const SECONDS_IN_DAY: u64 = 86400;

/// Normalize any timestamp to YYYYMMDD format of that day.
pub fn normalize_to_date(timestamp: &Timestamp) -> WorldwideDay {
    let naive_datetime =
        chrono::DateTime::from_timestamp_nanos(timestamp.nanos() as i64).naive_utc();
    let year = naive_datetime.year() as u32;
    let month = naive_datetime.month();
    let day = naive_datetime.day();

    year * 10000 + month * 100 + day
}

pub fn is_valid(date: &WorldwideDay) -> Result<(), DateError> {
    let year = date / 10000;
    let month = (date / 100) % 100;
    let day = date % 100;

    // Use chrono to validate the actual date
    match NaiveDate::from_ymd_opt(year as i32, month, day) {
        Some(_) => Ok(()),
        None => Err(DateError::InvalidDate {}),
    }
}

/// Worldwide day in YYYYMMDD format
pub type WorldwideDay = u32;


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
    fn test_normalize_to_date() {
        let timestamp = Timestamp::from_nanos(1672531200000000000); // 2023-01-01
        assert_eq!(normalize_to_date(&timestamp), 20230101);

        let timestamp = Timestamp::from_nanos(1688169600000000000); // 2023-07-01
        assert_eq!(normalize_to_date(&timestamp), 20230701);
    }

    #[test]
    fn test_is_valid() {
        // Valid dates
        assert!(is_valid(&20230101).is_ok());
        assert!(is_valid(&20231231).is_ok());
        assert!(is_valid(&20240229).is_ok()); // Leap year

        // Invalid dates
        assert_eq!(is_valid(&20230000), Err(DateError::InvalidDate {}));
        assert_eq!(is_valid(&20231232), Err(DateError::InvalidDate {}));
        assert_eq!(is_valid(&20230431), Err(DateError::InvalidDate {})); // April 31st
        assert_eq!(is_valid(&20230229), Err(DateError::InvalidDate {})); // Not leap year
    }
}
