#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime, TimeZone, Utc};
use chrono_tz::America::New_York;
use cron_parser::{CronField, ParseErrorKind, Schedule, parse_field};
use proptest::prelude::*;
use std::{collections::HashSet, str::FromStr};

fn utc(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
        .single()
        .expect("fixed test date")
}

#[test]
fn schedule_round_trips_and_has_semantic_equality() {
    let stepped = Schedule::from_str("*/15 * * * *").unwrap();
    let listed = Schedule::from_str("0,15,30,45 * * * *").unwrap();
    assert_eq!(stepped, listed);
    assert_eq!(stepped.to_string(), "*/15 * * * *");
    assert_eq!(stepped.source(), "*/15 * * * *");

    let mut schedules = HashSet::new();
    schedules.insert(stepped);
    schedules.insert(listed);
    assert_eq!(schedules.len(), 1);
}

#[test]
fn next_previous_and_iterators_are_strict() {
    let schedule = Schedule::from_str("*/5 * * * *").unwrap();
    let boundary = utc(2026, 1, 1, 0, 0);
    assert_eq!(schedule.next_after(&boundary), Some(utc(2026, 1, 1, 0, 5)));
    assert_eq!(
        schedule.previous_before(&boundary),
        Some(utc(2025, 12, 31, 23, 55))
    );
    assert_eq!(
        schedule.after(&boundary).take(3).collect::<Vec<_>>(),
        [
            utc(2026, 1, 1, 0, 5),
            utc(2026, 1, 1, 0, 10),
            utc(2026, 1, 1, 0, 15),
        ]
    );
    assert_eq!(
        schedule.before(&boundary).take(2).collect::<Vec<_>>(),
        [utc(2025, 12, 31, 23, 55), utc(2025, 12, 31, 23, 50)]
    );
    assert_eq!(
        schedule.after_owned(boundary).take(2).collect::<Vec<_>>(),
        [utc(2026, 1, 1, 0, 5), utc(2026, 1, 1, 0, 10)]
    );
}

#[test]
fn includes_requires_exact_minute_alignment() {
    let schedule = Schedule::from_str("30 9 * * Mon-Fri").unwrap();
    let monday = utc(2026, 1, 5, 9, 30);
    assert!(schedule.includes(&monday));
    assert!(!schedule.includes(&(monday + Duration::seconds(1))));
    assert!(!schedule.includes(&utc(2026, 1, 4, 9, 30)));
}

#[test]
fn leap_day_crosses_non_leap_century() {
    let schedule = Schedule::from_str("0 0 29 2 *").unwrap();
    assert_eq!(
        schedule.next_after(&utc(2096, 2, 29, 0, 0)),
        Some(utc(2104, 2, 29, 0, 0))
    );
}

#[test]
fn day_of_month_and_day_of_week_use_and_semantics() {
    let schedule = Schedule::from_str("0 0 1 * Mon").unwrap();
    assert_eq!(
        schedule.next_after(&utc(2024, 1, 2, 0, 0)),
        Some(utc(2024, 4, 1, 0, 0))
    );
}

#[test]
fn impossible_dates_are_rejected_at_compile_time() {
    let error = Schedule::from_str("0 0 31 2 *").unwrap_err();
    assert_eq!(error.kind(), &ParseErrorKind::ImpossibleSchedule);
}

#[test]
fn structured_errors_identify_the_field_and_token() {
    let error = Schedule::from_str("Mon * * * *").unwrap_err();
    assert_eq!(error.field(), Some(CronField::Minute));
    assert_eq!(error.token(), "Mon");
    assert_eq!(error.span(), 0..3);
    assert_eq!(error.kind(), &ParseErrorKind::NameNotAllowed);
}

#[test]
fn numeric_values_reject_a_leading_plus() {
    for expression in [
        "+5 * * * *",
        "*/+5 * * * *",
        "1-+5 * * * *",
        "0 +5 * * *",
        "0 0 +5 * *",
        "0 0 * +5 *",
        "0 0 * * +5",
    ] {
        let error = Schedule::from_str(expression).unwrap_err();
        assert_eq!(error.kind(), &ParseErrorKind::InvalidInteger);
    }

    let error = parse_field("+5", 0, 59).unwrap_err();
    assert_eq!(error.kind(), &ParseErrorKind::InvalidInteger);
}

#[test]
fn spring_forward_skips_nonexistent_local_time() {
    let schedule = Schedule::from_str("30 2 * * *").unwrap();
    let before = utc(2024, 3, 10, 5, 0).with_timezone(&New_York);
    assert_eq!(
        schedule.next_after(&before).unwrap().with_timezone(&Utc),
        utc(2024, 3, 11, 6, 30)
    );
}

#[test]
fn fall_back_emits_both_instants_in_absolute_order() {
    let schedule = Schedule::from_str("30 1 * * *").unwrap();
    let before = utc(2024, 11, 3, 4, 0).with_timezone(&New_York);
    let actual = schedule
        .after(&before)
        .take(3)
        .map(|value| value.with_timezone(&Utc))
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            utc(2024, 11, 3, 5, 30),
            utc(2024, 11, 3, 6, 30),
            utc(2024, 11, 4, 6, 30),
        ]
    );

    let second_fold = utc(2024, 11, 3, 6, 15).with_timezone(&New_York);
    assert_eq!(
        schedule
            .next_after(&second_fold)
            .unwrap()
            .with_timezone(&Utc),
        utc(2024, 11, 3, 6, 30)
    );

    let first_occurrence = utc(2024, 11, 3, 5, 30).with_timezone(&New_York);
    assert_eq!(
        schedule
            .next_after(&first_occurrence)
            .unwrap()
            .with_timezone(&Utc),
        utc(2024, 11, 3, 6, 30)
    );

    let second_occurrence = utc(2024, 11, 3, 6, 30).with_timezone(&New_York);
    assert_eq!(
        schedule
            .next_after(&second_occurrence)
            .unwrap()
            .with_timezone(&Utc),
        utc(2024, 11, 4, 6, 30)
    );
}

#[test]
fn dense_fall_back_sequence_remains_monotonic() {
    let schedule = Schedule::from_str("*/15 1 * * *").unwrap();
    let before = utc(2024, 11, 3, 4, 59).with_timezone(&New_York);
    let values = schedule
        .after(&before)
        .take(8)
        .map(|value| value.with_timezone(&Utc))
        .collect::<Vec<_>>();
    assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(values[0], utc(2024, 11, 3, 5, 0));
    assert_eq!(values[4], utc(2024, 11, 3, 6, 0));
}

#[test]
fn reverse_fall_back_emits_both_instants() {
    let schedule = Schedule::from_str("30 1 * * *").unwrap();
    let after = utc(2024, 11, 3, 8, 0).with_timezone(&New_York);
    let actual = schedule
        .before(&after)
        .take(3)
        .map(|value| value.with_timezone(&Utc))
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            utc(2024, 11, 3, 6, 30),
            utc(2024, 11, 3, 5, 30),
            utc(2024, 11, 2, 5, 30),
        ]
    );
}

#[test]
fn representable_boundaries_do_not_panic() {
    let schedule = Schedule::from_str("* * * * *").unwrap();
    assert!(schedule.next_after(&DateTime::<Utc>::MAX_UTC).is_none());
    assert!(
        schedule
            .previous_before(&DateTime::<Utc>::MIN_UTC)
            .is_none()
    );

    let east_offset = FixedOffset::east_opt(14 * 60 * 60).unwrap();
    let east = DateTime::<Utc>::MAX_UTC.with_timezone(&east_offset);
    let last_east_minute = NaiveDateTime::MAX.date().and_hms_opt(23, 59, 0).unwrap();
    assert!(!schedule.includes(&east));
    assert!(schedule.next_after(&east).is_none());
    assert_eq!(
        schedule.previous_before(&east),
        east_offset.from_local_datetime(&last_east_minute).single()
    );
    assert_eq!(
        schedule.before(&east).next(),
        east_offset.from_local_datetime(&last_east_minute).single()
    );

    let west_offset = FixedOffset::west_opt(14 * 60 * 60).unwrap();
    let west = DateTime::<Utc>::MIN_UTC.with_timezone(&west_offset);
    let first_west_minute = NaiveDateTime::MIN.date().and_hms_opt(0, 0, 0).unwrap();
    assert!(!schedule.includes(&west));
    assert_eq!(
        schedule.next_after(&west),
        west_offset.from_local_datetime(&first_west_minute).single()
    );
    assert_eq!(
        schedule.after(&west).next(),
        west_offset.from_local_datetime(&first_west_minute).single()
    );
    assert!(schedule.previous_before(&west).is_none());
}

#[test]
fn schedule_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Schedule>();
}

proptest! {
    #[test]
    fn utc_queries_are_monotonic_and_match(
        minute in 0_u32..60,
        hour in 0_u32..24,
        timestamp in 1_577_836_800_i64..1_893_456_000_i64,
    ) {
        let expression = format!("{minute} {hour} * * *");
        let schedule = Schedule::from_str(&expression).unwrap();
        let cursor = Utc.timestamp_opt(timestamp, 0).single().unwrap();
        let next = schedule.next_after(&cursor).unwrap();
        prop_assert!(next > cursor);
        prop_assert!(schedule.includes(&next));

        let just_after = next + Duration::seconds(1);
        prop_assert_eq!(schedule.previous_before(&just_after), Some(next));
    }
}
