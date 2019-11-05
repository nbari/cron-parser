use crate::ParseError;
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use std::collections::BTreeSet;

/// Parse cron syntax
/// ```text
/// ┌───────────── minute (0 - 59)
/// │ ┌───────────── hour (0 - 23)
/// │ │ ┌───────────── day of the month (1 - 31)
/// │ │ │ ┌───────────── month (1 - 12)
/// │ │ │ │ ┌───────────── day of the week (0 - 6)
/// │ │ │ │ │
/// │ │ │ │ │
/// │ │ │ │ │
/// * * * * * command to execute
/// ```
pub fn next_minute(
    minutes: BTreeSet<u32>,
    dt: DateTime<Utc>,
    current_minute: u32,
) -> Result<DateTime<Utc>, ParseError> {
    let next_minute: u32;
    let mut iterator = minutes.iter();
    loop {
        match iterator.next() {
            Some(minute) => {
                if *minute >= dt.minute() {
                    next_minute = *minute;
                    break;
                }
            }
            None => {
                next_minute = *minutes.iter().nth(0).unwrap();
                break;
            }
        }
    }
    let mut next_dt =
        chrono::Utc
            .ymd(dt.year(), dt.month(), dt.day())
            .and_hms(dt.hour(), next_minute, 0);
    if next_minute != dt.minute() {
        if dt.minute() > next_minute {
            next_dt = next_dt + chrono::Duration::hours(1);
        }
    }
    Ok(next_dt)
}

// * <hour> * * * command to execute
pub fn next_hour(
    hours: BTreeSet<u32>,
    dt: DateTime<Utc>,
    current_hour: u32,
) -> Result<DateTime<Utc>, ParseError> {
    Ok(dt)
}

// * * <dom> * * command to execute
pub fn next_dom(
    days: BTreeSet<u32>,
    dt: DateTime<Utc>,
    current_day: u32,
) -> Result<DateTime<Utc>, ParseError> {
    let next_day: u32;
    let mut iterator = days.iter();
    loop {
        match iterator.next() {
            Some(day) => {
                if *day >= current_day {
                    next_day = *day;
                    break;
                }
            }
            None => {
                next_day = *days.iter().nth(0).unwrap();
                break;
            }
        }
    }
    let mut next_dt = chrono::Utc
        .ymd(dt.year(), dt.month(), dt.day())
        .and_hms(0, 0, 0);
    if next_day != current_day {
        println!("------>{}, {}", next_day, dt.day());
        if next_day < dt.day() {
            if dt.month() == 12 {
                next_dt = chrono::Utc
                    .ymd(next_dt.year() + 1, 1, next_day)
                    .and_hms(0, 0, 0);
            } else {
                next_dt = chrono::Utc
                    .ymd(next_dt.year(), next_dt.month() + 1, next_day)
                    .and_hms(0, 0, 0);
            }
        }
    }
    Ok(next_dt)
}

// * * * <month> * command to execute
pub fn next_month(months: BTreeSet<u32>, dt: DateTime<Utc>) -> Result<DateTime<Utc>, ParseError> {
    let next_month: u32;
    let mut iterator = months.iter();
    loop {
        match iterator.next() {
            Some(month) => {
                if *month >= dt.month() {
                    next_month = *month;
                    break;
                }
            }
            None => {
                next_month = *months.iter().nth(0).unwrap();
                break;
            }
        }
    }
    let mut next_dt = dt;
    if next_month != dt.month() {
        if next_month < dt.month() {
            next_dt = chrono::Utc
                .ymd(dt.year() + 1, next_month, 1)
                .and_hms(0, 0, 0);
        }
    }
    Ok(next_dt)
}

// * * * * <dow> command to execute
//pub fn next_dow(hours: BTreeSet<u32>, dt: DateTime<Utc>) -> Result<DateTime<Utc>, ParseError> {}
