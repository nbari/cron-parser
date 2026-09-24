//! Tour of the compiled `Schedule` API.
//!
//! Every section uses fixed timestamps and asserts its results, so running
//! `cargo run --example schedule` both demonstrates and checks the behavior.

use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::{America::New_York, Tz};
use cron_parser::{ParseErrorKind, Schedule};
use std::{collections::HashSet, error::Error};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const FORMAT: &str = "%a %Y-%m-%d %H:%M:%S %Z";

fn main() -> Result<()> {
    compile_once_and_query()?;
    strict_boundaries_with_seconds()?;
    iterators()?;
    membership()?;
    leap_day_across_non_leap_century()?;
    day_of_month_and_day_of_week()?;
    dst_spring_forward()?;
    dst_fall_back()?;
    structured_errors()?;
    semantic_equality()?;

    println!("\nAll schedule examples passed.");
    Ok(())
}

fn utc(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Result<DateTime<Utc>> {
    Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
        .ok_or_else(|| "invalid fixed timestamp".into())
}

fn section(title: &str) {
    println!("\n== {title} ==");
}

/// Parse once, then reuse the compiled schedule for any number of queries.
fn compile_once_and_query() -> Result<()> {
    section("Compile once, query many times");

    let schedule: Schedule = "*/15 9-17 * * Mon-Fri".parse()?;
    println!("expression: {schedule}");

    // Friday 2026-01-02 17:50 UTC: the last slot of the day has passed.
    let cursor = utc(2026, 1, 2, 17, 50, 0)?;
    let next = schedule.next_after(&cursor).ok_or("no next occurrence")?;
    let previous = schedule
        .previous_before(&cursor)
        .ok_or("no previous occurrence")?;

    println!("cursor:   {}", cursor.format(FORMAT));
    println!("next:     {}  (skips the weekend)", next.format(FORMAT));
    println!("previous: {}", previous.format(FORMAT));

    assert_eq!(next, utc(2026, 1, 5, 9, 0, 0)?);
    assert_eq!(previous, utc(2026, 1, 2, 17, 45, 0)?);
    Ok(())
}

/// `next_after` and `previous_before` never return the cursor itself, and
/// seconds or nanoseconds in the cursor are handled exactly.
fn strict_boundaries_with_seconds() -> Result<()> {
    section("Strict boundaries");

    let schedule: Schedule = "*/5 * * * *".parse()?;

    let on_slot = utc(2026, 1, 1, 10, 0, 0)?;
    assert_eq!(
        schedule.next_after(&on_slot),
        Some(utc(2026, 1, 1, 10, 5, 0)?)
    );
    assert_eq!(
        schedule.previous_before(&on_slot),
        Some(utc(2026, 1, 1, 9, 55, 0)?)
    );
    println!("cursor exactly on 10:00 -> next 10:05, previous 09:55");

    let mid_minute = utc(2026, 1, 1, 10, 0, 30)?;
    assert_eq!(
        schedule.next_after(&mid_minute),
        Some(utc(2026, 1, 1, 10, 5, 0)?)
    );
    assert_eq!(schedule.previous_before(&mid_minute), Some(on_slot));
    println!("cursor at 10:00:30     -> next 10:05, previous 10:00");
    Ok(())
}

/// Forward, reverse, and owned iterators.
fn iterators() -> Result<()> {
    section("Iterators");

    let schedule: Schedule = "0 */6 * * *".parse()?;
    let cursor = utc(2026, 1, 1, 0, 0, 0)?;

    let upcoming: Vec<_> = schedule.after(&cursor).take(4).collect();
    println!("next 4:");
    for value in &upcoming {
        println!("  {}", value.format(FORMAT));
    }
    assert_eq!(
        upcoming,
        [
            utc(2026, 1, 1, 6, 0, 0)?,
            utc(2026, 1, 1, 12, 0, 0)?,
            utc(2026, 1, 1, 18, 0, 0)?,
            utc(2026, 1, 2, 0, 0, 0)?,
        ]
    );

    let recent: Vec<_> = schedule.before(&cursor).take(2).collect();
    println!("previous 2:");
    for value in &recent {
        println!("  {}", value.format(FORMAT));
    }
    assert_eq!(
        recent,
        [utc(2025, 12, 31, 18, 0, 0)?, utc(2025, 12, 31, 12, 0, 0)?]
    );

    // The owned iterator does not borrow the schedule, so it can be returned
    // from a function or stored in a struct.
    let mut owned = daily_noon_from(cursor)?;
    let first = owned.next().ok_or("owned iterator is empty")?;
    println!("owned iterator first: {}", first.format(FORMAT));
    assert_eq!(first, utc(2026, 1, 1, 12, 0, 0)?);
    Ok(())
}

fn daily_noon_from(cursor: DateTime<Utc>) -> Result<impl Iterator<Item = DateTime<Utc>>> {
    let schedule: Schedule = "0 12 * * *".parse()?;
    Ok(schedule.after_owned(cursor))
}

/// `includes` answers whether an exact, minute-aligned instant matches.
fn membership() -> Result<()> {
    section("Membership");

    let schedule: Schedule = "30 9 * * Mon-Fri".parse()?;
    let cases = [
        ("Monday 09:30:00", utc(2026, 1, 5, 9, 30, 0)?, true),
        ("Monday 09:30:01", utc(2026, 1, 5, 9, 30, 1)?, false),
        ("Sunday 09:30:00", utc(2026, 1, 4, 9, 30, 0)?, false),
    ];
    for (label, value, expected) in cases {
        let included = schedule.includes(&value);
        println!("{label}: {included}");
        assert_eq!(included, expected);
    }
    Ok(())
}

/// The search covers the full 400-year Gregorian cycle, so sparse schedules
/// such as leap days work across non-leap centuries like 2100.
fn leap_day_across_non_leap_century() -> Result<()> {
    section("Leap day across 2100");

    let schedule: Schedule = "0 0 29 2 *".parse()?;
    let cursor = utc(2096, 2, 29, 0, 0, 0)?;
    let next = schedule.next_after(&cursor).ok_or("no leap day")?;
    println!("after 2096-02-29 -> {}", next.format(FORMAT));
    assert_eq!(next, utc(2104, 2, 29, 0, 0, 0)?);
    Ok(())
}

/// Day-of-month and day-of-week are combined with AND.
fn day_of_month_and_day_of_week() -> Result<()> {
    section("Day of month AND day of week");

    let friday_13th: Schedule = "0 0 13 * Fri".parse()?;
    let dates: Vec<_> = friday_13th
        .after(&utc(2026, 1, 1, 0, 0, 0)?)
        .take(3)
        .map(|value| value.format("%Y-%m-%d").to_string())
        .collect();
    println!("next Friday the 13ths: {}", dates.join(", "));
    assert_eq!(dates, ["2026-02-13", "2026-03-13", "2026-11-13"]);
    Ok(())
}

fn new_york(utc_value: DateTime<Utc>) -> DateTime<Tz> {
    utc_value.with_timezone(&New_York)
}

/// Local times that do not exist on a spring-forward day are skipped.
fn dst_spring_forward() -> Result<()> {
    section("DST spring forward (America/New_York, 2024-03-10)");

    let schedule: Schedule = "30 2 * * *".parse()?;
    let cursor = new_york(utc(2024, 3, 10, 5, 0, 0)?); // 00:00 EST
    let next = schedule.next_after(&cursor).ok_or("no next occurrence")?;
    println!(
        "02:30 does not exist on 2024-03-10; next is {}",
        next.format(FORMAT)
    );
    assert_eq!(next, new_york(utc(2024, 3, 11, 6, 30, 0)?));
    Ok(())
}

/// Both real instants of a repeated fall-back time are returned, in order.
fn dst_fall_back() -> Result<()> {
    section("DST fall back (America/New_York, 2024-11-03)");

    let schedule: Schedule = "30 1 * * *".parse()?;
    let cursor = new_york(utc(2024, 11, 3, 4, 0, 0)?); // 00:00 EDT
    let values: Vec<_> = schedule.after(&cursor).take(3).collect();
    for value in &values {
        println!(
            "  {}  ({})",
            value.format(FORMAT),
            value.with_timezone(&Utc).format("%H:%M UTC")
        );
    }
    assert_eq!(
        values,
        [
            new_york(utc(2024, 11, 3, 5, 30, 0)?), // 01:30 EDT
            new_york(utc(2024, 11, 3, 6, 30, 0)?), // 01:30 EST
            new_york(utc(2024, 11, 4, 6, 30, 0)?),
        ]
    );

    // Walking backwards visits the same instants in reverse order.
    let reverse: Vec<_> = schedule
        .before(&new_york(utc(2024, 11, 3, 8, 0, 0)?))
        .take(2)
        .collect();
    assert_eq!(
        reverse,
        [
            new_york(utc(2024, 11, 3, 6, 30, 0)?),
            new_york(utc(2024, 11, 3, 5, 30, 0)?)
        ]
    );
    Ok(())
}

/// Errors report the field, the offending token, its byte span, and a kind.
fn structured_errors() -> Result<()> {
    section("Structured errors");

    let cases = [
        ("0 0 31 2 *", ParseErrorKind::ImpossibleSchedule),
        ("Mon * * * *", ParseErrorKind::NameNotAllowed),
        ("*/0 * * * *", ParseErrorKind::ZeroStep),
        (
            "0 24 * * *",
            ParseErrorKind::OutOfRange {
                min: 0,
                max: 23,
                actual: 24,
            },
        ),
        ("1,,2 * * * *", ParseErrorKind::EmptyListElement),
        ("+5 * * * *", ParseErrorKind::InvalidInteger),
        (
            "* * * *",
            ParseErrorKind::WrongFieldCount {
                expected: 5,
                actual: 4,
            },
        ),
    ];

    for (expression, expected) in cases {
        let Err(error) = expression.parse::<Schedule>() else {
            return Err(format!("{expression} should be rejected").into());
        };
        let span = error.span();
        let caret = format!(
            "{}{}",
            " ".repeat(span.start),
            "^".repeat(span.len().max(1))
        );
        println!("{expression:<14} {error}");
        println!("{:<14} {caret}", "");
        assert_eq!(error.kind(), &expected);
    }
    Ok(())
}

/// Equality and hashing compare what a schedule matches, not its spelling.
fn semantic_equality() -> Result<()> {
    section("Semantic equality");

    let spellings = ["*/15 * * * *", "0,15,30,45 * * * *", "0-59/15 * * * *"];
    let mut unique = HashSet::new();
    for spelling in spellings {
        let schedule: Schedule = spelling.parse()?;
        println!("{:<20} source() = {:?}", spelling, schedule.source());
        unique.insert(schedule);
    }
    println!(
        "{} spellings, {} distinct schedule",
        spellings.len(),
        unique.len()
    );
    assert_eq!(unique.len(), 1);
    Ok(())
}
