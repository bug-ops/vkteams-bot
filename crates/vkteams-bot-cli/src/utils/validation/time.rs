//! Time-related validation functions
//!
//! This module contains validation functions for time formats, cron expressions,
//! and scheduling-related time values.

use crate::errors::prelude::{CliError, Result as CliResult};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Utc};
use cron::Schedule;
use std::str::FromStr;

/// Validate a cron expression
///
/// # Arguments
/// * `cron_expr` - The cron expression to validate
///
/// # Returns
/// * `Ok(())` if the cron expression is valid
/// * `Err(CliError::InputError)` if the cron expression is invalid
pub fn validate_cron_expression(cron_expr: &str) -> CliResult<()> {
    if cron_expr.trim().is_empty() {
        return Err(CliError::InputError(
            "Cron expression cannot be empty".to_string(),
        ));
    }

    Schedule::from_str(cron_expr)
        .map_err(|e| CliError::InputError(format!("Invalid cron expression: {}", e)))?;

    Ok(())
}

/// Validate that a datetime string can be parsed
///
/// # Arguments
/// * `datetime_str` - The datetime string to validate
///
/// # Returns
/// * `Ok(DateTime<Utc>)` if the datetime string is valid and parsed
/// * `Err(CliError::InputError)` if the datetime string is invalid
pub fn validate_datetime_string(datetime_str: &str) -> CliResult<DateTime<Utc>> {
    if datetime_str.trim().is_empty() {
        return Err(CliError::InputError(
            "Datetime string cannot be empty".to_string(),
        ));
    }

    parse_datetime_flexible(datetime_str)
}

/// Parse a datetime string with multiple format support
///
/// # Arguments
/// * `time_str` - The time string to parse
///
/// # Returns
/// * `Ok(DateTime<Utc>)` if successfully parsed
/// * `Err(CliError::InputError)` if parsing fails
pub fn parse_datetime_flexible(time_str: &str) -> CliResult<DateTime<Utc>> {
    // Try different formats
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d",
        "%H:%M:%S",
        "%H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y-%m-%dT%H:%M:%S%.3fZ",
    ];

    for format in &formats {
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(time_str, format) {
            return Ok(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
        }
        if let Ok(naive_date) = NaiveDate::parse_from_str(time_str, format) {
            return Ok(DateTime::from_naive_utc_and_offset(
                naive_date.and_hms_opt(0, 0, 0).unwrap(),
                Utc,
            ));
        }
    }

    // Try relative times
    if let Ok(dt) = parse_relative_time(time_str) {
        return Ok(dt);
    }

    Err(CliError::InputError(format!(
        "Invalid time format: {}. Use YYYY-MM-DD HH:MM:SS, or relative time like '30m', '2h', '1d'",
        time_str
    )))
}

/// Parse relative time expressions (e.g., "30m", "2h", "1d")
///
/// # Arguments
/// * `time_str` - The relative time string to parse
///
/// # Returns
/// * `Ok(DateTime<Utc>)` if successfully parsed
/// * `Err(CliError::InputError)` if parsing fails
pub fn parse_relative_time(time_str: &str) -> CliResult<DateTime<Utc>> {
    let time_str = time_str.trim().to_lowercase();

    if time_str.is_empty() {
        return Err(CliError::InputError(
            "Relative time cannot be empty".to_string(),
        ));
    }

    let now = Utc::now();

    // Parse relative times
    if let Some(stripped) = time_str.strip_suffix('s') {
        if let Ok(seconds) = stripped.parse::<i64>() {
            return Ok(now + Duration::seconds(seconds));
        }
    }
    if let Some(stripped) = time_str.strip_suffix('m') {
        if let Ok(minutes) = stripped.parse::<i64>() {
            return Ok(now + Duration::minutes(minutes));
        }
    }
    if let Some(stripped) = time_str.strip_suffix('h') {
        if let Ok(hours) = stripped.parse::<i64>() {
            return Ok(now + Duration::hours(hours));
        }
    }
    if let Some(stripped) = time_str.strip_suffix('d') {
        if let Ok(days) = stripped.parse::<i64>() {
            return Ok(now + Duration::days(days));
        }
    }
    if let Some(stripped) = time_str.strip_suffix('w') {
        if let Ok(weeks) = stripped.parse::<i64>() {
            return Ok(now + Duration::weeks(weeks));
        }
    }

    // Special keywords
    match time_str.as_str() {
        "now" => Ok(now),
        "tomorrow" => Ok(now + Duration::days(1)),
        "yesterday" => Ok(now - Duration::days(1)),
        _ => Err(CliError::InputError(format!(
            "Invalid relative time format: {}. Use formats like '30s', '5m', '2h', '1d', '1w' or 'now'",
            time_str
        ))),
    }
}

/// Validate that a duration value is reasonable
///
/// # Arguments
/// * `duration_seconds` - The duration in seconds to validate
/// * `min_seconds` - Minimum allowed duration in seconds
/// * `max_seconds` - Maximum allowed duration in seconds
///
/// # Returns
/// * `Ok(())` if the duration is within valid bounds
/// * `Err(CliError::InputError)` if the duration is invalid
pub fn validate_duration_bounds(
    duration_seconds: u64,
    min_seconds: u64,
    max_seconds: u64,
) -> CliResult<()> {
    if duration_seconds < min_seconds {
        return Err(CliError::InputError(format!(
            "Duration too short: {} seconds (minimum: {} seconds)",
            duration_seconds, min_seconds
        )));
    }
    if duration_seconds > max_seconds {
        return Err(CliError::InputError(format!(
            "Duration too long: {} seconds (maximum: {} seconds)",
            duration_seconds, max_seconds
        )));
    }
    Ok(())
}

/// Validate that a datetime is in the future
///
/// # Arguments
/// * `datetime` - The datetime to validate
///
/// # Returns
/// * `Ok(())` if the datetime is in the future
/// * `Err(CliError::InputError)` if the datetime is in the past
pub fn validate_future_datetime(datetime: DateTime<Utc>) -> CliResult<()> {
    let now = Utc::now();
    if datetime <= now {
        return Err(CliError::InputError(format!(
            "Datetime must be in the future. Given: {}, Current: {}",
            datetime.format("%Y-%m-%d %H:%M:%S UTC"),
            now.format("%Y-%m-%d %H:%M:%S UTC")
        )));
    }
    Ok(())
}

/// Validate that a datetime is not too far in the future
///
/// # Arguments
/// * `datetime` - The datetime to validate
/// * `max_future_days` - Maximum number of days in the future allowed
///
/// # Returns
/// * `Ok(())` if the datetime is within reasonable future bounds
/// * `Err(CliError::InputError)` if the datetime is too far in the future
pub fn validate_reasonable_future(datetime: DateTime<Utc>, max_future_days: i64) -> CliResult<()> {
    let now = Utc::now();
    let max_future = now + Duration::days(max_future_days);

    if datetime > max_future {
        return Err(CliError::InputError(format!(
            "Datetime is too far in the future. Given: {}, Maximum allowed: {}",
            datetime.format("%Y-%m-%d %H:%M:%S UTC"),
            max_future.format("%Y-%m-%d %H:%M:%S UTC")
        )));
    }
    Ok(())
}

/// Parse and validate a schedule time string
///
/// # Arguments
/// * `time_str` - The time string to parse and validate
///
/// # Returns
/// * `Ok(DateTime<Utc>)` if successfully parsed and validated
/// * `Err(CliError::InputError)` if parsing or validation fails
pub fn parse_and_validate_schedule_time(time_str: &str) -> CliResult<DateTime<Utc>> {
    let datetime = parse_datetime_flexible(time_str)?;
    validate_future_datetime(datetime)?;
    validate_reasonable_future(datetime, 365)?; // Max 1 year in future
    Ok(datetime)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_cron_expression() {
        assert!(validate_cron_expression("0 0 * * * *").is_ok()); // Every hour
        assert!(validate_cron_expression("0 */5 * * * *").is_ok()); // Every 5 minutes
        assert!(validate_cron_expression("0 0 9 * * *").is_ok()); // 9 AM daily
        assert!(validate_cron_expression("").is_err()); // Empty
        assert!(validate_cron_expression("invalid").is_err()); // Invalid format
    }

    #[test]
    fn test_parse_relative_time() {
        let _now = Utc::now();

        assert!(parse_relative_time("30s").is_ok());
        assert!(parse_relative_time("5m").is_ok());
        assert!(parse_relative_time("2h").is_ok());
        assert!(parse_relative_time("1d").is_ok());
        assert!(parse_relative_time("1w").is_ok());
        assert!(parse_relative_time("now").is_ok());
        assert!(parse_relative_time("tomorrow").is_ok());

        assert!(parse_relative_time("invalid").is_err());
        assert!(parse_relative_time("").is_err());
    }

    #[test]
    fn test_parse_datetime_flexible() {
        assert!(parse_datetime_flexible("2024-01-01 12:00:00").is_ok());
        assert!(parse_datetime_flexible("2024-01-01 12:00").is_ok());
        assert!(parse_datetime_flexible("2024-01-01").is_ok());
        assert!(parse_datetime_flexible("30m").is_ok());

        assert!(parse_datetime_flexible("invalid-date").is_err());
    }

    #[test]
    fn test_validate_duration_bounds() {
        assert!(validate_duration_bounds(60, 1, 3600).is_ok()); // 1 minute, within bounds
        assert!(validate_duration_bounds(0, 1, 3600).is_err()); // Too short
        assert!(validate_duration_bounds(7200, 1, 3600).is_err()); // Too long
    }

    #[test]
    fn test_validate_future_datetime() {
        let future = Utc::now() + Duration::hours(1);
        let past = Utc::now() - Duration::hours(1);

        assert!(validate_future_datetime(future).is_ok());
        assert!(validate_future_datetime(past).is_err());
    }

    #[test]
    fn test_validate_reasonable_future() {
        let near_future = Utc::now() + Duration::days(30);
        let far_future = Utc::now() + Duration::days(400);

        assert!(validate_reasonable_future(near_future, 365).is_ok());
        assert!(validate_reasonable_future(far_future, 365).is_err());
    }
}

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_validate_cron_expression_random(s in ".{0,128}") {
            let _ = validate_cron_expression(&s);
        }

        #[test]
        fn prop_parse_datetime_flexible_random(s in ".{0,128}") {
            let _ = parse_datetime_flexible(&s);
        }

        #[test]
        fn prop_parse_relative_time_random(s in ".{0,32}") {
            let _ = parse_relative_time(&s);
        }
    }
}
