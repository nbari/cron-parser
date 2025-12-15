#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]
use chrono::{Datelike, TimeZone, Timelike, Utc};
use chrono_tz::{America::Chicago, US::Pacific};
use cron_parser::{parse, parse_field};
use std::collections::BTreeSet;

macro_rules! parse_field_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, min, max, expected) = $value;
                let mut expect = BTreeSet::<u32>::new();
                for i in expected {
                    expect.insert(i);
                }
                assert_eq!(parse_field(input, min, max).unwrap(), expect);
            }
        )*
    }
}

parse_field_tests! {
    parse_any:("*", 0, 0, vec![0]),
    parse_minutes_0: ("0", 0, 59, vec![0]),
    parse_minutes_1: ("1", 0, 59, vec![1]),
    parse_hours: ("23", 0, 23, vec![23]),
    parse_days: ("31", 0, 31, vec![31]),
    parse_day_week: ("6", 0, 6,  vec![6]),
    parse_every_30: ("*/30", 0, 59, vec![0,30]),
    parse_every_5_minutes: ("*/5", 0, 59, vec![0,5,10,15,20,25,30,35,40,45,50,55]),
    parse_every_minute: ("*", 0, 59, vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59]),
    parse_every_minute_step: ("*/1", 0, 59, vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59]),
    parse_every_hour: ("*", 0, 23, vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23]),
    parse_every_hour_step: ("*/1", 0, 23, vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23]),
    parse_every_day: ("*", 1, 31, vec![1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31]),
    parse_every_day_step: ("*/1", 1, 31, vec![1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31]),
    parse_every_month: ("*", 1, 12, vec![1,2,3,4,5,6,7,8,9,10,11,12]),
    parse_every_month_step: ("*/1", 1, 12, vec![1,2,3,4,5,6,7,8,9,10,11,12]),
    parse_every_dweek: ("*", 0, 6, vec![0,1,2,3,4,5,6]),
    parse_every_dweek_step: ("*/1", 0, 6, vec![0,1,2,3,4,5,6]),
    parse_range_5_10_minutes: ("5-10", 0, 59, vec![5,6,7,8,9,10]),
    parse_range_5_10_hours: ("5-10", 0, 12, vec![5,6,7,8,9,10]),
    parse_list_minutes: ("15,30,45,0", 0, 59, vec![0,15,30,45]),
    parse_1024: ("1024", 0, 1024, vec![1024]),
    parse_repeat_values:("1,1,1,1,2", 0, 59, vec![1,2]),
    parse_range_and_list1: ("1-8,11", 0, 23, vec![1,2,3,4,5,6,7,8,11]),
    parse_range_and_list2: ("1-8,11,9,4,5", 0, 23, vec![1,2,3,4,5,6,7,8,9,11]),
    parse_range_and_list3: ("*,1-8,11,9,4,5", 0, 23, vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23]),
    parse_range_and_list4: ("2-3,*/15", 0, 23, vec![0,2,3,15]),
    parse_range_and_list5: ("2-3,9,*/15,1-8,11,9,4,5", 0, 23, vec![0,1,2,3,4,5,6,7,8,9,11,15]),
    parse_range_list_step: ("*/30,40-45,57", 0, 59, vec![0,30,40,41,42,43,44,45,57]),
    parse_range_list_step_repeated_values: ("*/30,40-45,57,30,44,41-45", 0, 59, vec![0,30,40,41,42,43,44,45,57]),
    parse_start_step_minute: ("1/6", 0, 59, vec![1,7,13,19,25,31,37,43,49,55]),
    parse_start_step_hour: ("1/6", 0, 23, vec![1,7,13,19]),
    parse_start_step_day: ("1/6", 1, 31, vec![1,7,13,19,25,31]),
    parse_start_step_month: ("1/6", 1, 12, vec![1,7]),
    parse_start_step_dow: ("1/6", 0, 6, vec![1]),
    parse_range_with_step_minute: ("5-40/3", 0, 59, vec![5,8,11,14,17,20,23,26,29,32,35,38]),
    parse_range_with_step_hour: ("12-18/2", 0, 23, vec![12,14,16,18]),
    parse_range_with_step_hour_2: ("1-23/6", 0, 23, vec![1,7,13,19]),
    parse_range_with_step_hour_3: ("1/6", 0, 23, vec![1,7,13,19]),
    parse_range_with_step_hour_4: ("6/1", 0, 23, vec![6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23]),
    parse_range_with_step_day: ("1-31/5", 1, 31, vec![1,6,11,16,21,26,31]),
    parse_range_with_step_month: ("1-12/3", 1, 12, vec![1,4,7,10]),
}

macro_rules! parse_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, ts, expected) = $value;
                let dt = Utc.timestamp_opt(ts, 0).unwrap();
                assert_eq!(parse(input, &dt).unwrap().timestamp(), expected);
                let dt = Pacific.from_local_datetime(&dt.naive_utc()).unwrap();
                let expected = Pacific
                    .from_local_datetime(&Utc.timestamp_opt(expected, 0).unwrap().naive_utc())
                    .unwrap()
                    .timestamp();
                assert_eq!(parse(input, &dt).unwrap().timestamp(), expected);
            }
        )*
    }
}

// https://play.rust-lang.org/
//
// extern crate chrono; // 0.4.9
// use chrono::{TimeZone, Utc};
// fn main() {
//     // 2019-11-05 15:56:35 UTC 1572969395
//     let dt = Utc.timestamp(1572969395, 0);
//     println!("{} {}", dt, dt.timestamp());
//     let next = Utc.ymd(2019, 11, 5).and_hms(16, 5, 0);
//     println!("{} {}", next, next.timestamp());
// }
parse_tests! {
    any_minute: ("* * * * *", 1_572_969_395, 1_572_969_420),
    any_minute2: ("*/5,* * * * *", 1_572_969_395, 1_572_969_420),
    every_5_mintues: ("*/5 * * * *", 1_572_969_395, 1_572_969_600),
    on_minute_5: ("5 * * * *", 1_572_969_395, 1_572_969_900),
    every_minute_every_2nd_hour: ("* */2 * * *", 1_572_969_395, 1_572_969_600),
    every_minute_in_october: ("* * * 10 *", 1_572_969_395, 1_601_510_400),
    every_minute_on_day_4_in_november: ("* * 4 11 *", 1_572_969_395, 1_604_448_000),
    daily_2am: ("0 2 * * *", 1_572_969_395, 1_573_005_600),
    twice_a_day_5_17: ("0 5,17 * * *", 1_572_969_395, 1_572_973_200),
    every_2nd_minute_every_hour_from_1_to_4_and_15:("*/2 1-4,15 * * *", 1_572_969_395,1_572_969_480),
    febraury_30_1: ("* * 30 */2 *", 1_572_969_395, 1_575_072_000),
    febraury_30_2: ("* * 30 * *", 1_548_892_800, 1_553_904_000),
    febraury_29: ("* * 29 2 *", 1_583_020_800, 1_709_164_800),
    day_31: ("* 5 31 * *", 1_548_936_000, 1_554_008_400),
    day_31_ever2months: ("* 5 31 */3 *", 1_548_936_000, 1_564_549_200),
    leap_year: ("* * 28-31 2 *", 1_583_020_800, 1_614_470_400),
    every_dow_0: ("0 0 * * 0", 1_573_151_292, 1_573_344_000),
    every_dow_1: ("0 0 * * 1", 1_573_151_292, 1_573_430_400),
    every_dow_2: ("0 0 * * 2", 1_573_151_292, 1_573_516_800),
    every_dow_3: ("0 0 * * 3", 1_573_151_292, 1_573_603_200),
    every_dow_4: ("0 0 * * 4", 1_573_151_292, 1_573_689_600),
    every_dow_5: ("0 0 * * 5", 1_573_151_292, 1_573_171_200),
    every_dow_6: ("0 0 * * 6", 1_573_151_292, 1_573_257_600),
    every_dow_sun: ("0 0 * * Sun", 1_573_151_292, 1_573_344_000),
    every_dow_mon: ("0 0 * * Mon", 1_573_151_292, 1_573_430_400),
    every_dow_tue: ("0 0 * * Tue", 1_573_151_292, 1_573_516_800),
    every_dow_wed: ("0 0 * * Wed", 1_573_151_292, 1_573_603_200),
    every_dow_thu: ("0 0 * * Thu", 1_573_151_292, 1_573_689_600),
    every_dow_fri: ("0 0 * * Fri", 1_573_151_292, 1_573_171_200),
    every_dow_sat: ("0 0 * * Sat", 1_573_151_292, 1_573_257_600),
    every_dow_wed_and_fri: ("0 0 * * Wed,Fri", 1_573_151_292, 1_573_171_200),
    dow_feb: ("0 0 29 2 6", 1_573_151_292, 1_582_934_400),
    every_dow_wed_2_fri: ("0 0 * * Wed-Fri", 1_573_151_292, 1_573_171_200),
}

#[test]
fn parse_field_double_field() {
    assert!(parse_field("**", 0, 0).is_err());
}

#[test]
fn parse_field_bad_range() {
    assert!(parse_field("1-2-3", 0, 0).is_err(),);
    assert!(parse_field("8-5", 0, 0).is_err(),);
}

#[test]
fn bad_minute_input() {
    assert!(parse_field("60", 0, 59).is_err(),);
}

#[test]
fn bad_minute_input_range() {
    assert!(parse_field("5-60", 0, 59).is_err());
}

#[test]
fn bad_minute_input_list() {
    assert!(parse_field("40,50,60", 0, 59).is_err());
}

#[test]
fn bad_hour_input_step() {
    assert!(parse_field("*/30", 0, 23).is_err());
}

#[test]
fn february_30() {
    assert!(parse("* * 30 2 *", &Utc::now()).is_err());
}

#[test]
fn test_parse() {
    assert!(parse("*/5 * * * *", &Utc::now()).is_ok());
    assert!(parse("0 0 29 2 5", &Utc.timestamp_opt(1_573_151_292, 0).unwrap()).is_err());
    assert!(parse("0 0 * * */Wed", &Utc::now()).is_err());
    assert!(parse("0 0 * * */2-5", &Utc::now()).is_err());
}

#[test]
fn test_bad_input() {
    assert!(parse("2-3,9,*/15,1-8,11,9,4,5, * * * *", &Utc::now()).is_ok());
    assert!(parse("2-3,9,*/15,1-8,11,9,4,5,,,, * * * *", &Utc::now()).is_ok());
}

#[test]
fn test_next_100_iterations() {
    let now = Utc.timestamp_opt(1_573_239_864, 0).unwrap();
    let mut next = parse("0 23 */2 * *", &now).unwrap();
    assert_eq!(next.timestamp(), 1_573_340_400);
    for _ in 0..100 {
        next = parse("0 23 */2 * *", &next).unwrap();
    }
    assert_eq!(next.timestamp(), 1_590_274_800);
}

#[test]
fn test_timezone() {
    let utc = Utc.timestamp_opt(1_573_405_861, 0).unwrap();
    let pacific_time = utc.with_timezone(&Pacific);
    let next_pt = parse("*/5 * * * *", &pacific_time).unwrap();
    assert_eq!(next_pt.timestamp(), 1_573_406_100);
    let next_utc = parse("*/5 * * * *", &utc).unwrap();
    assert_eq!(next_utc.timestamp(), 1_573_406_100);
    assert_ne!(next_pt.to_string(), next_utc.to_string());
}

#[test]
fn test_timezone_dst() {
    // 2_018-11-04 1:30
    let utc = Utc.timestamp_opt(1_541_309_400, 0).unwrap();
    let cst = utc.with_timezone(&Chicago);
    let mut next = parse("*/15 * * * *", &cst).unwrap();
    for _ in 0..10 {
        next = parse("*/15 * * * *", &next).unwrap();
    }
    assert_eq!(next.timestamp(), 1_541_322_900);
}

#[test]
// check if input filds are < 5
fn parse_needs_5_fields() {
    assert!(parse("*/5 * * * *", &Utc::now()).is_ok());
    assert!(parse("*/5 * * *", &Utc::now()).is_err());
    assert!(parse("*/5 * *", &Utc::now()).is_err());
    assert!(parse("*/5 *", &Utc::now()).is_err());
    assert!(parse("*/5", &Utc::now()).is_err());
    assert!(parse("* * * * * *", &Utc::now()).is_err());
}

#[test]
fn parse_start_step() {
    assert!(parse("* 1/6 * * *", &Utc::now()).is_ok());
    assert!(parse("* 0/5 * * *", &Utc::now()).is_ok());
    assert!(parse("* */5 * * *", &Utc::now()).is_ok());
    assert!(parse("* */0 * * *", &Utc::now()).is_err());
}

#[test]
fn combine_ranges_with_steps() {
    assert!(parse("* 12-18/2 * * *", &Utc::now()).is_ok());
    // Test the exact pattern from CHANGELOG
    assert!(parse("0 12-18/3 * * *", &Utc::now()).is_ok());
}

#[test]
fn test_range_step_patterns() {
    // Verify the CHANGELOG example works correctly
    let now = Utc.timestamp_opt(1_573_239_864, 0).unwrap(); // 2019-11-08 17:44:24 UTC
    let result = parse("0 12-18/3 * * *", &now);
    assert!(result.is_ok());
    let next = result.unwrap();
    // Next should be at 18:00 same day or 12:00 next day
    assert!(next.hour() == 18 || (next.hour() == 12 && next.day() == 9));

    // Test various range-step patterns
    assert!(parse("0 0-23/6 * * *", &Utc::now()).is_ok()); // Every 6 hours
    assert!(parse("*/10 9-17 * * *", &Utc::now()).is_ok()); // Every 10 min during 9-5
    assert!(parse("0 1-12/2 * * *", &Utc::now()).is_ok()); // Every 2 hours from 1-12
    assert!(parse("30 8-20/4 * * 1-5", &Utc::now()).is_ok()); // Complex pattern with range-step
}

#[test]
fn test_changelog_example_12_18_step_3() {
    // Test the exact CHANGELOG example: "0 12-18/3 * * *"
    // This should execute at 12:00, 15:00, and 18:00 each day

    // Starting before first execution (11:00)
    let before_first = Utc.timestamp_opt(1_573_210_800, 0).unwrap(); // 2019-11-08 11:00:00 UTC
    let next = parse("0 12-18/3 * * *", &before_first).unwrap();
    assert_eq!(next.hour(), 12);
    assert_eq!(next.minute(), 0);

    // Starting after first execution (13:00)
    let after_first = Utc.timestamp_opt(1_573_218_000, 0).unwrap(); // 2019-11-08 13:00:00 UTC
    let next = parse("0 12-18/3 * * *", &after_first).unwrap();
    assert_eq!(next.hour(), 15);
    assert_eq!(next.minute(), 0);

    // Starting after second execution (16:00)
    let after_second = Utc.timestamp_opt(1_573_228_800, 0).unwrap(); // 2019-11-08 16:00:00 UTC
    let next = parse("0 12-18/3 * * *", &after_second).unwrap();
    assert_eq!(next.hour(), 18);
    assert_eq!(next.minute(), 0);

    // Starting after last execution (19:00) - should go to next day
    let after_last = Utc.timestamp_opt(1_573_239_600, 0).unwrap(); // 2019-11-08 19:00:00 UTC
    let next = parse("0 12-18/3 * * *", &after_last).unwrap();
    assert_eq!(next.hour(), 12);
    assert_eq!(next.minute(), 0);
    assert_eq!(next.day(), 9); // Next day
}

#[test]
fn test_range_step_edge_cases() {
    // Test edge cases for range-step
    let now = Utc::now();

    // Step equals range
    assert!(parse("0 12-12/1 * * *", &now).is_ok());

    // Large step that only hits start
    assert!(parse("0 12-18/10 * * *", &now).is_ok());

    // Step of 1 (same as regular range)
    assert!(parse("0 12-18/1 * * *", &now).is_ok());

    // Multiple range-step patterns
    assert!(parse("0 1-5/2,10-14/2 * * *", &now).is_ok());
}

#[test]
fn parse_field_start_stop_0() {
    assert!(parse_field("*/0", 0, 24).is_err());
}

#[test]
fn test_field_parsing() {
    // Valid: */5
    let result = parse_field("*/5", 0, 59).unwrap();
    assert_eq!(result, (0..=59).step_by(5).collect::<BTreeSet<u32>>());

    // Valid: 1/6
    let result = parse_field("1/6", 0, 59).unwrap();
    assert_eq!(result, (1..=59).step_by(6).collect::<BTreeSet<u32>>());

    // Valid: 12-18/2
    let result = parse_field("12-18/2", 0, 23).unwrap();
    assert_eq!(result, BTreeSet::from([12, 14, 16, 18]));

    // Invalid: step 0
    assert!(parse_field("*/0", 0, 59).is_err());

    // Invalid: out-of-range values
    assert!(parse_field("60", 0, 59).is_err());
    assert!(parse_field("24", 0, 23).is_err());
    assert!(parse_field("13-25", 1, 12).is_err());

    // Invalid: invalid range
    assert!(parse_field("10-5", 0, 59).is_err()); // Reverse range
}

// Additional edge case tests
#[test]
fn test_invalid_dow_name() {
    // Invalid day-of-week names should error
    assert!(parse("0 0 * * InvalidDay", &Utc::now()).is_err());
    assert!(parse("0 0 * * Monday", &Utc::now()).is_err()); // Full name not supported
}

#[test]
fn test_step_greater_than_max() {
    // Step value greater than max should error
    assert!(parse_field("*/60", 0, 59).is_err());
    assert!(parse_field("*/100", 0, 23).is_err());
    assert!(parse_field("1/60", 0, 59).is_err());
}

#[test]
fn test_empty_field_parts() {
    // Empty parts after comma should be ignored
    // Currently returns empty set - could be improved to error
    assert_eq!(parse_field(",,,", 0, 59).unwrap(), BTreeSet::new());
    assert_eq!(parse_field("1,,,2", 0, 59).unwrap(), BTreeSet::from([1, 2]));
}

#[test]
fn test_invalid_range_step_format() {
    // Multiple slashes should error
    assert!(parse_field("1/2/3", 0, 59).is_err());
    assert!(parse_field("*/5/2", 0, 59).is_err());
}

#[test]
fn test_range_with_invalid_step_zero() {
    // Range with step 0 should error
    assert!(parse_field("1-10/0", 0, 59).is_err());
    assert!(parse_field("0-23/0", 0, 23).is_err());
}

#[test]
fn test_dow_range() {
    // Test day-of-week ranges with names
    assert!(parse_field("Mon-Fri", 0, 6).is_ok());
    assert_eq!(
        parse_field("Mon-Fri", 0, 6).unwrap(),
        BTreeSet::from([1, 2, 3, 4, 5])
    );
    // Sat-Sun should error because Sat (6) > Sun (0) is an invalid range
    assert!(parse_field("Sat-Sun", 0, 6).is_err());
}

#[test]
fn test_very_large_numbers() {
    // Numbers way beyond max should error
    assert!(parse_field("9999", 0, 59).is_err());
    assert!(parse_field("100-200", 0, 59).is_err());
}

#[test]
fn test_mixed_dow_formats() {
    // Mixed numeric and named days
    assert!(parse_field("0,Mon,5,Fri", 0, 6).is_ok());
    assert_eq!(
        parse_field("0,Mon,5,Fri", 0, 6).unwrap(),
        BTreeSet::from([0, 1, 5])
    );
}

#[test]
fn test_cron_edge_cases() {
    // Test some edge cases with full cron expressions
    // April only has 30 days, so April 31 will error after 4-year lookahead
    assert!(parse("0 0 31 4 *", &Utc::now()).is_err());
    // Feb 31 doesn't exist, will error after 4-year lookahead
    assert!(parse("0 0 31 2 *", &Utc::now()).is_err());

    // Invalid: step in dow field
    assert!(parse("0 0 * * */8", &Utc::now()).is_err()); // Step > 6 for dow
}

#[test]
fn test_whitespace_handling() {
    // Multiple spaces should be handled correctly
    assert!(parse("*  *  *  *  *", &Utc::now()).is_ok());
    assert!(parse("*/5    *    *    *    *", &Utc::now()).is_ok());
}

#[test]
fn test_case_insensitive_dow() {
    // Day names should be case insensitive
    assert_eq!(
        parse_field("mon", 0, 6).unwrap(),
        parse_field("MON", 0, 6).unwrap()
    );
    assert_eq!(
        parse_field("mon", 0, 6).unwrap(),
        parse_field("Mon", 0, 6).unwrap()
    );
}

// Test for invalid range in step with range format (e.g., "1-2-3/5")
#[test]
fn test_invalid_range_in_step() {
    // Multiple dashes in range with step
    assert!(parse_field("1-2-3/5", 0, 59).is_err());
    assert!(parse_field("10-20-30/2", 0, 59).is_err());
}

// Test for invalid range where start > end in step format
#[test]
fn test_reverse_range_with_step() {
    // Reverse range with step should error
    assert!(parse_field("20-10/2", 0, 59).is_err());
    assert!(parse_field("50-40/5", 0, 59).is_err());
}

// Test cron expression that will never match within 4 years
#[test]
fn test_cron_never_matches() {
    // Invalid combination: February 30 will never occur
    let now = Utc.timestamp_opt(1_573_151_292, 0).unwrap();
    assert!(parse("0 0 30 2 *", &now).is_err());
}

// Test with DST transition (spring forward) - skipped hour
#[test]
fn test_dst_spring_forward_skipped_time() {
    // 2024-03-10 02:00 AM PST does not exist (spring forward to 3:00 AM PDT)
    // Use Pacific timezone which has DST
    let before_dst = Pacific.with_ymd_and_hms(2024, 3, 10, 1, 30, 0).unwrap();

    // Schedule at 2:30 AM which doesn't exist due to DST
    // The parser should skip to the next valid time
    let result = parse("30 2 * * *", &before_dst);
    assert!(result.is_ok());

    // The result should be after the DST transition
    let next = result.unwrap();
    // Should skip the non-existent 2:30 AM and go to the next day
    assert!(next.day() >= 10);
}

// Test with fall back DST - ambiguous time
#[test]
fn test_dst_fall_back_ambiguous_time() {
    // November 3, 2024: 1:00 AM happens twice (fall back)
    // Create a time during the ambiguous period
    match Pacific.with_ymd_and_hms(2024, 11, 3, 1, 30, 0) {
        chrono::LocalResult::Ambiguous(earlier, _later) => {
            // Use the earlier (PDT) time
            let result = parse("*/15 * * * *", &earlier);
            assert!(result.is_ok());
        }
        chrono::LocalResult::Single(dt) => {
            // If single, just test it works
            let result = parse("*/15 * * * *", &dt);
            assert!(result.is_ok());
        }
        chrono::LocalResult::None => {
            panic!("1:30 AM on Nov 3 should exist");
        }
    }
}

// Test with extremely restrictive cron that takes long to find next match
#[test]
fn test_very_restrictive_cron() {
    // Feb 29 only on leap years that fall on Friday
    let now = Utc.timestamp_opt(1_577_836_800, 0).unwrap(); // 2020-01-01

    // This should work as 2020-02-29 is on Saturday (day 6)
    // But if we look for Sunday (day 0), it won't match in 4 years
    let result = parse("0 0 29 2 0", &now);
    // Feb 29 on Sunday doesn't occur in the next 4 years from 2020
    assert!(result.is_err());
}

// 1541322900 -> 1_541_322_900
// vim :%s/\(\d\)\(\(\d\d\d\)\+\d\@!\)\@=/\1_/g
