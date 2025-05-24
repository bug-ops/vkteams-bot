//! Time utilities for VK Teams Bot CLI
//!
//! This module provides time parsing, formatting, and manipulation utilities
//! used throughout the CLI application.

use crate::errors::prelude::{CliError, Result as CliResult};
use chrono::{DateTime, Utc, Duration, NaiveDateTime, NaiveDate, Timelike, Datelike};
use cron::Schedule;
use std::str::FromStr;

/// Parse a schedule time string with flexible format support
///
/// # Arguments
/// * `time_str` - The time string to parse
///
/// # Returns
/// * `Ok(DateTime<Utc>)` if successfully parsed
/// * `Err(CliError::InputError)` if parsing fails
pub fn parse_schedule_time(time_str: &str) -> CliResult<DateTime<Utc>> {
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
                Utc
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
        return Err(CliError::InputError("Relative time cannot be empty".to_string()));
    }

    let now = Utc::now();

    // Parse relative times
    if time_str.ends_with('s') {
        if let Ok(seconds) = time_str[..time_str.len()-1].parse::<i64>() {
            return Ok(now + Duration::seconds(seconds));
        }
    }
    if time_str.ends_with('m') {
        if let Ok(minutes) = time_str[..time_str.len()-1].parse::<i64>() {
            return Ok(now + Duration::minutes(minutes));
        }
    }
    if time_str.ends_with('h') {
        if let Ok(hours) = time_str[..time_str.len()-1].parse::<i64>() {
            return Ok(now + Duration::hours(hours));
        }
    }
    if time_str.ends_with('d') {
        if let Ok(days) = time_str[..time_str.len()-1].parse::<i64>() {
            return Ok(now + Duration::days(days));
        }
    }
    if time_str.ends_with('w') {
        if let Ok(weeks) = time_str[..time_str.len()-1].parse::<i64>() {
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

/// Format a duration into a human-readable string
///
/// # Arguments
/// * `duration` - The duration to format
///
/// # Returns
/// * A human-readable string representation of the duration
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.num_seconds();
    
    if total_seconds < 0 {
        return format!("-{}", format_duration(-duration));
    }

    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    let mut parts = Vec::new();

    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{}s", seconds));
    }

    parts.join(" ")
}

/// Format a datetime into a user-friendly string
///
/// # Arguments
/// * `dt` - The datetime to format
///
/// # Returns
/// * A formatted string representation of the datetime
pub fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Format a datetime relative to now (e.g., "in 5 minutes", "2 hours ago")
///
/// # Arguments
/// * `dt` - The datetime to format relative to now
///
/// # Returns
/// * A relative time string
pub fn format_datetime_relative(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = dt.signed_duration_since(now);

    if diff.num_seconds().abs() < 60 {
        return "now".to_string();
    }

    let abs_diff = if diff.num_seconds() < 0 { -diff } else { diff };
    let formatted = format_duration(abs_diff);

    if diff.num_seconds() < 0 {
        format!("{} ago", formatted)
    } else {
        format!("in {}", formatted)
    }
}

/// Get the next occurrence of a cron expression
///
/// # Arguments
/// * `cron_expr` - The cron expression
/// * `from_time` - Optional base time (defaults to now)
///
/// # Returns
/// * `Ok(DateTime<Utc>)` if the next occurrence can be calculated
/// * `Err(CliError::InputError)` if the cron expression is invalid
pub fn get_next_cron_occurrence(
    cron_expr: &str,
    from_time: Option<DateTime<Utc>>,
) -> CliResult<DateTime<Utc>> {
    let schedule = Schedule::from_str(cron_expr)
        .map_err(|e| CliError::InputError(format!("Invalid cron expression: {}", e)))?;

    let base_time = from_time.unwrap_or_else(Utc::now);
    
    schedule
        .after(&base_time)
        .next()
        .ok_or_else(|| CliError::InputError("No upcoming time for cron expression".to_string()))
}

/// Calculate the next run time for an interval-based schedule
///
/// # Arguments
/// * `duration_seconds` - The interval duration in seconds
/// * `start_time` - The schedule start time
/// * `from_time` - Optional base time (defaults to now)
///
/// # Returns
/// * The next scheduled run time
pub fn get_next_interval_occurrence(
    duration_seconds: u64,
    start_time: DateTime<Utc>,
    from_time: Option<DateTime<Utc>>,
) -> DateTime<Utc> {
    let base_time = from_time.unwrap_or_else(Utc::now);

    if base_time < start_time {
        return start_time;
    }

    let elapsed = base_time.signed_duration_since(start_time);
    let interval = Duration::seconds(duration_seconds as i64);
    let intervals_passed = elapsed.num_seconds() / interval.num_seconds();
    let next_interval = intervals_passed + 1;

    start_time + Duration::seconds(next_interval * interval.num_seconds())
}

/// Check if a datetime is within business hours (9 AM to 5 PM UTC)
///
/// # Arguments
/// * `dt` - The datetime to check
///
/// # Returns
/// * `true` if the datetime is within business hours
/// * `false` otherwise
pub fn is_business_hours(dt: DateTime<Utc>) -> bool {
    let hour = dt.hour();
    hour >= 9 && hour < 17
}

/// Check if a datetime is on a weekend (Saturday or Sunday)
///
/// # Arguments
/// * `dt` - The datetime to check
///
/// # Returns
/// * `true` if the datetime is on a weekend
/// * `false` otherwise
pub fn is_weekend(dt: DateTime<Utc>) -> bool {
    let weekday = dt.weekday();
    weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun
}

/// Round a datetime to the nearest minute
///
/// # Arguments
/// * `dt` - The datetime to round
///
/// # Returns
/// * The rounded datetime
pub fn round_to_minute(dt: DateTime<Utc>) -> DateTime<Utc> {
    let rounded_naive = dt.naive_utc()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();
    
    DateTime::from_naive_utc_and_offset(rounded_naive, Utc)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_format_duration() {
        let duration = Duration::seconds(3661); // 1 hour, 1 minute, 1 second
        let formatted = format_duration(duration);
        assert_eq!(formatted, "1h 1m 1s");

        let duration = Duration::seconds(60); // 1 minute
        let formatted = format_duration(duration);
        assert_eq!(formatted, "1m");

        let duration = Duration::seconds(0); // 0 seconds
        let formatted = format_duration(duration);
        assert_eq!(formatted, "0s");
    }

    #[test]
    fn test_parse_schedule_time() {
        assert!(parse_schedule_time("2024-01-01 12:00:00").is_ok());
        assert!(parse_schedule_time("2024-01-01 12:00").is_ok());
        assert!(parse_schedule_time("2024-01-01").is_ok());
        assert!(parse_schedule_time("30m").is_ok());
        
        assert!(parse_schedule_time("invalid-date").is_err());
    }

    #[test]
    fn test_get_next_cron_occurrence() {
        // Test every hour (6-field format: sec min hour day month weekday)
        assert!(get_next_cron_occurrence("0 0 * * * *", None).is_ok());
        
        // Test invalid cron
        assert!(get_next_cron_occurrence("invalid", None).is_err());
    }

    #[test]
    fn test_format_datetime_relative() {
        let now = Utc::now();
        let future = now + Duration::minutes(5);
        let past = now - Duration::hours(2);
        
        let future_str = format_datetime_relative(future);
        assert!(future_str.contains("in"));
        
        let past_str = format_datetime_relative(past);
        assert!(past_str.contains("ago"));
    }

    #[test]
    fn test_is_business_hours() {
        // Create a datetime at 10 AM UTC
        let dt = Utc::now().date_naive().and_hms_opt(10, 0, 0).unwrap();
        let dt_utc = DateTime::from_naive_utc_and_offset(dt, Utc);
        assert!(is_business_hours(dt_utc));
        
        // Create a datetime at 6 PM UTC
        let dt = Utc::now().date_naive().and_hms_opt(18, 0, 0).unwrap();
        let dt_utc = DateTime::from_naive_utc_and_offset(dt, Utc);
        assert!(!is_business_hours(dt_utc));
    }
}