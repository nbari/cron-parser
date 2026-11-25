#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use cron_parser::{CronField, ParseErrorKind, Schedule, parse, parse_field};
use jiff::{
    SignedDuration, Timestamp, ZonedDateTime,
    civil::{self, DateTime},
    tz::{AmbiguousOffset, Offset, TimeZone},
};
use proptest::prelude::*;
use std::{collections::HashSet, str::FromStr};

fn utc(year: i16, month: i8, day: i8, hour: i8, minute: i8) -> ZonedDateTime {
    civil::datetime(year, month, day, hour, minute, 0, 0)
        .to_zoned_datetime(TimeZone::UTC)
        .expect("fixed test date")
}

fn utc_at(seconds: i64) -> ZonedDateTime {
    Timestamp::from_second(seconds)
        .expect("fixed test timestamp")
        .to_zoned_datetime(TimeZone::UTC)
}

fn zone(name: &str) -> TimeZone {
    TimeZone::get(name).expect("known time zone")
}

fn new_york() -> TimeZone {
    zone("America/New_York")
}

fn seconds_after(value: &ZonedDateTime, seconds: i64) -> ZonedDateTime {
    value
        .checked_add(SignedDuration::from_secs(seconds))
        .expect("representable instant")
}

/// The UTC instants of a list of zoned datetimes, for exact comparisons.
fn instants(values: &[ZonedDateTime]) -> Vec<Timestamp> {
    values.iter().map(ZonedDateTime::timestamp).collect()
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
    assert!(!schedule.includes(&seconds_after(&monday, 1)));
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
    let before = utc(2024, 3, 10, 5, 0).with_time_zone(new_york());
    let next = schedule.next_after(&before).unwrap();
    assert_eq!(next, utc(2024, 3, 11, 6, 30));
    assert_eq!(next.datetime(), civil::datetime(2024, 3, 11, 2, 30, 0, 0));

    // Walking backwards skips the same nonexistent minute.
    let after = utc(2024, 3, 11, 0, 0).with_time_zone(new_york());
    assert_eq!(
        schedule.previous_before(&after).unwrap(),
        utc(2024, 3, 9, 7, 30)
    );
}

#[test]
fn fall_back_emits_both_instants_in_absolute_order() {
    let schedule = Schedule::from_str("30 1 * * *").unwrap();
    let before = utc(2024, 11, 3, 4, 0).with_time_zone(new_york());
    let actual = schedule.after(&before).take(3).collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            utc(2024, 11, 3, 5, 30),
            utc(2024, 11, 3, 6, 30),
            utc(2024, 11, 4, 6, 30),
        ]
    );
    // Both instants share the wall-clock time but not the UTC offset.
    assert_eq!(actual[0].datetime(), actual[1].datetime());
    assert_eq!(actual[0].offset(), Offset::constant(-4));
    assert_eq!(actual[1].offset(), Offset::constant(-5));
    assert!(actual.iter().all(|value| schedule.includes(value)));

    let second_fold = utc(2024, 11, 3, 6, 15).with_time_zone(new_york());
    assert_eq!(
        schedule.next_after(&second_fold).unwrap(),
        utc(2024, 11, 3, 6, 30)
    );

    let first_occurrence = utc(2024, 11, 3, 5, 30).with_time_zone(new_york());
    assert_eq!(
        schedule.next_after(&first_occurrence).unwrap(),
        utc(2024, 11, 3, 6, 30)
    );

    let second_occurrence = utc(2024, 11, 3, 6, 30).with_time_zone(new_york());
    assert_eq!(
        schedule.next_after(&second_occurrence).unwrap(),
        utc(2024, 11, 4, 6, 30)
    );
}

#[test]
fn dense_fall_back_sequence_remains_monotonic() {
    let schedule = Schedule::from_str("*/15 1 * * *").unwrap();
    let before = utc(2024, 11, 3, 4, 59).with_time_zone(new_york());
    let values = schedule.after(&before).take(8).collect::<Vec<_>>();
    assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(
        instants(&values),
        instants(&[
            utc(2024, 11, 3, 5, 0),
            utc(2024, 11, 3, 5, 15),
            utc(2024, 11, 3, 5, 30),
            utc(2024, 11, 3, 5, 45),
            utc(2024, 11, 3, 6, 0),
            utc(2024, 11, 3, 6, 15),
            utc(2024, 11, 3, 6, 30),
            utc(2024, 11, 3, 6, 45),
        ])
    );
}

#[test]
fn reverse_fall_back_emits_both_instants() {
    let schedule = Schedule::from_str("30 1 * * *").unwrap();
    let after = utc(2024, 11, 3, 8, 0).with_time_zone(new_york());
    let actual = schedule.before(&after).take(3).collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            utc(2024, 11, 3, 6, 30),
            utc(2024, 11, 3, 5, 30),
            utc(2024, 11, 2, 5, 30),
        ]
    );

    // Strict boundaries hold from inside either occurrence of the fold.
    let second_occurrence = utc(2024, 11, 3, 6, 30).with_time_zone(new_york());
    assert_eq!(
        schedule.previous_before(&second_occurrence).unwrap(),
        utc(2024, 11, 3, 5, 30)
    );
    let first_occurrence = utc(2024, 11, 3, 5, 30).with_time_zone(new_york());
    assert_eq!(
        schedule.previous_before(&first_occurrence).unwrap(),
        utc(2024, 11, 2, 5, 30)
    );
}

#[test]
fn half_hour_fold_is_handled() {
    // Australia/Lord_Howe falls back by 30 minutes: on 2024-04-07 the local
    // half hour 01:30..02:00 happens twice (+11:00, then +10:30).
    let lord_howe = zone("Australia/Lord_Howe");
    let repeated = civil::datetime(2024, 4, 7, 1, 45, 0, 0);
    assert!(matches!(
        lord_howe.to_ambiguous_timestamp(repeated).offset(),
        AmbiguousOffset::Fold { .. }
    ));

    let schedule = Schedule::from_str("45 1 * * *").unwrap();
    let before = utc(2024, 4, 6, 12, 0).with_time_zone(lord_howe.clone());
    let actual = schedule.after(&before).take(3).collect::<Vec<_>>();
    assert_eq!(
        instants(&actual),
        instants(&[
            utc(2024, 4, 6, 14, 45),
            utc(2024, 4, 6, 15, 15),
            utc(2024, 4, 7, 15, 15),
        ])
    );
    assert!(actual.iter().all(|value| value.time_zone() == &lord_howe));

    let reverse = schedule
        .before(&utc(2024, 4, 7, 0, 0).with_time_zone(lord_howe))
        .take(3)
        .collect::<Vec<_>>();
    assert_eq!(
        instants(&reverse),
        instants(&[
            utc(2024, 4, 6, 15, 15),
            utc(2024, 4, 6, 14, 45),
            utc(2024, 4, 5, 14, 45),
        ])
    );
}

#[test]
fn whole_skipped_day_is_not_emitted() {
    // Pacific/Apia crossed the date line at the end of 2011: local
    // 2011-12-30 never happened.
    let apia = zone("Pacific/Apia");
    let schedule = Schedule::from_str("0 12 30 12 *").unwrap();
    let before = civil::datetime(2011, 12, 1, 0, 0, 0, 0)
        .to_zoned_datetime(apia.clone())
        .unwrap();
    let next = schedule.next_after(&before).unwrap();
    assert_eq!(next.datetime(), civil::datetime(2012, 12, 30, 12, 0, 0, 0));

    let daily = Schedule::from_str("0 12 * * *").unwrap();
    let days = daily
        .after(
            &civil::datetime(2011, 12, 28, 13, 0, 0, 0)
                .to_zoned_datetime(apia)
                .unwrap(),
        )
        .take(3)
        .map(|value| value.date())
        .collect::<Vec<_>>();
    assert_eq!(
        days,
        [
            civil::date(2011, 12, 29),
            civil::date(2011, 12, 31),
            civil::date(2012, 1, 1),
        ]
    );
}

#[test]
fn results_keep_the_query_time_zone() {
    let schedule = Schedule::from_str("0 9 * * *").unwrap();
    let cursor = utc(2026, 6, 1, 0, 0);

    for time_zone in [
        TimeZone::UTC,
        TimeZone::fixed(Offset::constant(-3)),
        new_york(),
        zone("Asia/Kolkata"),
    ] {
        let local = cursor.with_time_zone(time_zone.clone());
        let next = schedule.next_after(&local).unwrap();
        let previous = schedule.previous_before(&local).unwrap();
        for value in [&next, &previous] {
            assert_eq!(value.time_zone(), &time_zone);
            assert_eq!(value.time(), civil::time(9, 0, 0, 0));
        }
        assert!(previous < local && local < next);
        assert_eq!(parse("0 9 * * *", &local).unwrap(), next);
        assert_eq!(schedule.after(&local).next().unwrap(), next);
        assert_eq!(schedule.before(&local).next().unwrap(), previous);
        assert_eq!(schedule.after_owned(local.clone()).next().unwrap(), next);
        assert_eq!(schedule.before_owned(local).next().unwrap(), previous);
    }

    // The same instant yields a different occurrence in a different zone.
    let kolkata = schedule
        .next_after(&cursor.with_time_zone(zone("Asia/Kolkata")))
        .unwrap();
    assert_eq!(kolkata, utc(2026, 6, 1, 3, 30));
    assert_eq!(schedule.next_after(&cursor).unwrap(), utc(2026, 6, 1, 9, 0));
}

#[test]
fn sub_minute_cursors_are_strict() {
    let schedule = Schedule::from_str("*/5 * * * *").unwrap();
    let slot = utc(2026, 1, 1, 10, 0);
    let just_after = slot.checked_add(SignedDuration::from_nanos(1)).unwrap();
    let just_before = slot.checked_sub(SignedDuration::from_nanos(1)).unwrap();

    assert!(!schedule.includes(&just_after));
    assert_eq!(schedule.next_after(&just_before).unwrap(), slot);
    assert_eq!(schedule.previous_before(&just_after).unwrap(), slot);
    assert_eq!(
        schedule.next_after(&just_after).unwrap(),
        utc(2026, 1, 1, 10, 5)
    );
    assert_eq!(
        schedule.previous_before(&just_before).unwrap(),
        utc(2026, 1, 1, 9, 55)
    );
}

#[test]
fn representable_boundaries_do_not_panic() {
    let schedule = Schedule::from_str("* * * * *").unwrap();
    let latest = Timestamp::MAX.to_zoned_datetime(TimeZone::UTC);
    let earliest = Timestamp::MIN.to_zoned_datetime(TimeZone::UTC);
    assert!(schedule.next_after(&latest).is_none());
    assert!(schedule.after(&latest).next().is_none());
    assert!(schedule.previous_before(&earliest).is_none());
    assert!(schedule.before(&earliest).next().is_none());
    assert!(parse("* * * * *", &latest).is_err());

    // Away from the edge the neighbouring minutes are still found.
    assert_eq!(
        schedule.previous_before(&latest).unwrap().datetime(),
        civil::datetime(9999, 12, 30, 22, 0, 0, 0)
    );
    assert_eq!(
        schedule.next_after(&earliest).unwrap().datetime(),
        civil::datetime(-9999, 1, 2, 2, 0, 0, 0)
    );

    // The largest offset puts the last instant on the last civil datetime.
    let east_zone = TimeZone::fixed(Offset::MAX);
    let east = Timestamp::MAX.to_zoned_datetime(east_zone.clone());
    assert_eq!(east.datetime(), DateTime::MAX);
    assert!(!schedule.includes(&east));
    assert!(schedule.next_after(&east).is_none());
    let last_east_minute = schedule.previous_before(&east).unwrap();
    assert_eq!(
        last_east_minute.datetime(),
        civil::datetime(9999, 12, 31, 23, 59, 0, 0)
    );
    assert_eq!(last_east_minute.time_zone(), &east_zone);
    assert_eq!(schedule.before(&east).next().unwrap(), last_east_minute);

    // The smallest offset puts the first instant on the first civil datetime.
    let west_zone = TimeZone::fixed(Offset::MIN);
    let west = Timestamp::MIN.to_zoned_datetime(west_zone.clone());
    assert_eq!(west.datetime(), DateTime::MIN);
    assert!(schedule.includes(&west));
    assert!(schedule.previous_before(&west).is_none());
    let first_west_minute = schedule.next_after(&west).unwrap();
    assert_eq!(
        first_west_minute.datetime(),
        civil::datetime(-9999, 1, 1, 0, 1, 0, 0)
    );
    assert_eq!(first_west_minute.time_zone(), &west_zone);
    assert_eq!(schedule.after(&west).next().unwrap(), first_west_minute);

    // Sparse schedules give up cleanly instead of searching past the range.
    let leap_day = Schedule::from_str("0 0 29 2 *").unwrap();
    assert!(leap_day.next_after(&latest).is_none());
    assert!(leap_day.previous_before(&earliest).is_none());
    assert_eq!(
        leap_day.previous_before(&latest).unwrap().date(),
        civil::date(9996, 2, 29)
    );
}

#[test]
fn schedule_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Schedule>();
}

/// Zones and local dates around which an offset transition happens.
const TRANSITIONS: [(&str, i16, i8, i8); 9] = [
    ("America/New_York", 2024, 3, 10),
    ("America/New_York", 2024, 11, 3),
    ("Europe/Lisbon", 2024, 3, 31),
    ("Europe/Lisbon", 2024, 10, 27),
    ("Australia/Lord_Howe", 2024, 4, 7),
    ("Australia/Lord_Howe", 2024, 10, 6),
    ("Pacific/Apia", 2011, 12, 30),
    ("Asia/Kolkata", 2024, 6, 15),
    ("UTC", 2024, 2, 29),
];

/// A cursor within two days of one of the `TRANSITIONS`.
fn transition_cursor(index: usize, offset_seconds: i64) -> ZonedDateTime {
    let (name, year, month, day) = TRANSITIONS[index];
    let midnight = civil::date(year, month, day)
        .at(0, 0, 0, 0)
        .to_zoned_datetime(TimeZone::UTC)
        .unwrap();
    seconds_after(&midnight, offset_seconds).with_time_zone(zone(name))
}

/// Every minute-aligned instant strictly between two instants.
fn minutes_between(
    start: &ZonedDateTime,
    end: &ZonedDateTime,
) -> impl Iterator<Item = ZonedDateTime> {
    let time_zone = start.time_zone().clone();
    let first = start.timestamp().as_second().div_euclid(60) + 1;
    let last = (end.timestamp().as_second() - 1).div_euclid(60);
    (first..=last).map(move |minute| {
        Timestamp::from_second(minute * 60)
            .unwrap()
            .to_zoned_datetime(time_zone.clone())
    })
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
        let cursor = utc_at(timestamp);
        let next = schedule.next_after(&cursor).unwrap();
        prop_assert!(next > cursor);
        prop_assert!(schedule.includes(&next));

        let just_after = seconds_after(&next, 1);
        prop_assert_eq!(schedule.previous_before(&just_after), Some(next));
    }

    /// Around DST gaps, folds, and a skipped day, `next_after` returns exactly
    /// the first matching minute and `previous_before` exactly the last one.
    #[test]
    fn zoned_queries_find_the_nearest_match_around_transitions(
        index in 0..TRANSITIONS.len(),
        offset_seconds in -172_800_i64..172_800,
        minute_step in 1_u32..60,
        hours in prop::sample::select(vec!["*", "0-3", "1", "2", "1,2", "*/6"]),
    ) {
        let expression = format!("*/{minute_step} {hours} * * *");
        let schedule = Schedule::from_str(&expression).unwrap();
        let cursor = transition_cursor(index, offset_seconds);

        let next = schedule.next_after(&cursor).unwrap();
        prop_assert!(next > cursor);
        prop_assert!(schedule.includes(&next));
        prop_assert_eq!(next.time_zone(), cursor.time_zone());
        for skipped in minutes_between(&cursor, &next) {
            prop_assert!(!schedule.includes(&skipped), "{} skipped {}", expression, skipped);
        }

        let previous = schedule.previous_before(&cursor).unwrap();
        prop_assert!(previous < cursor);
        prop_assert!(schedule.includes(&previous));
        prop_assert_eq!(previous.time_zone(), cursor.time_zone());
        for skipped in minutes_between(&previous, &cursor) {
            prop_assert!(!schedule.includes(&skipped), "{} skipped {}", expression, skipped);
        }

        // Forward and backward queries are inverses of each other.
        prop_assert_eq!(schedule.previous_before(&seconds_after(&next, 1)), Some(next.clone()));
        prop_assert_eq!(schedule.next_after(&seconds_after(&previous, -1)), Some(previous));
        if schedule.includes(&cursor) {
            prop_assert_eq!(schedule.previous_before(&next), Some(cursor));
        }
    }

    /// Iterators are strictly monotonic and visit the same instants in both
    /// directions, including across transitions.
    #[test]
    fn iterators_agree_in_both_directions(
        index in 0..TRANSITIONS.len(),
        offset_seconds in -172_800_i64..172_800,
        minute in 0_u32..60,
        hours in prop::sample::select(vec!["*", "0-3", "1", "2"]),
    ) {
        let expression = format!("{minute} {hours} * * *");
        let schedule = Schedule::from_str(&expression).unwrap();
        let cursor = transition_cursor(index, offset_seconds);

        let forward = schedule.after(&cursor).take(12).collect::<Vec<_>>();
        prop_assert_eq!(forward.len(), 12);
        prop_assert!(forward.windows(2).all(|pair| pair[0] < pair[1]));

        let end = seconds_after(&forward[11], 1);
        let mut backward = schedule.before(&end).take(12).collect::<Vec<_>>();
        backward.reverse();
        prop_assert_eq!(instants(&forward), instants(&backward));
    }
}
