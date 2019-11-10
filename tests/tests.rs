use chrono::{TimeZone, Utc};
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
}

macro_rules! parse_tests {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, ts, expected) = $value;
                let dt = Utc.timestamp(ts, 0);
                assert_eq!(parse(input, dt).unwrap().timestamp(), expected);
                use chrono_tz::US::Pacific;
                let dt = Pacific.from_local_datetime(&dt.naive_utc()).unwrap();
                let expected = Pacific
                    .from_local_datetime(&Utc.timestamp(expected, 0).naive_utc())
                    .unwrap()
                    .timestamp();
                assert_eq!(parse(input, dt).unwrap().timestamp(), expected);
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
    any_minute: ("* * * * *", 1572969395, 1572969420),
    any_minute2: ("*/5,* * * * *", 1572969395, 1572969420),
    every_5_mintues: ("*/5 * * * *", 1572969395, 1572969600),
    on_minute_5: ("5 * * * *", 1572969395, 1572969900),
    every_minute_every_2nd_hour: ("* */2 * * *", 1572969395, 1572969600),
    every_minute_in_october: ("* * * 10 *", 1572969395, 1601510400),
    every_minute_on_day_4_in_november: ("* * 4 11 *", 1572969395, 1604448000),
    daily_2am: ("0 2 * * *", 1572969395, 1573005600),
    twice_a_day_5_17: ("0 5,17 * * *", 1572969395, 1572973200),
    every_2nd_minute_every_hour_from_1_to_4_and_15:("*/2 1-4,15 * * *", 1572969395,1572969480),
    febraury_30_1: ("* * 30 */2 *", 1572969395, 1575072000),
    febraury_30_2: ("* * 30 * *", 1548892800, 1553904000),
    febraury_29: ("* * 29 2 *", 1583020800, 1709164800),
    day_31: ("* 5 31 * *", 1548936000, 1554008400),
    day_31_ever2months: ("* 5 31 */3 *", 1548936000, 1564549200),
    leap_year: ("* * 28-31 2 *", 1583020800, 1614470400),
    every_dow_0: ("0 0 * * 0", 1573151292, 1573344000),
    every_dow_1: ("0 0 * * 1", 1573151292, 1573430400),
    every_dow_2: ("0 0 * * 2", 1573151292, 1573516800),
    every_dow_3: ("0 0 * * 3", 1573151292, 1573603200),
    every_dow_4: ("0 0 * * 4", 1573151292, 1573689600),
    every_dow_5: ("0 0 * * 5", 1573151292, 1573171200),
    every_dow_6: ("0 0 * * 6", 1573151292, 1573257600),
    every_dow_sun: ("0 0 * * Sun", 1573151292, 1573344000),
    every_dow_mon: ("0 0 * * Mon", 1573151292, 1573430400),
    every_dow_tue: ("0 0 * * Tue", 1573151292, 1573516800),
    every_dow_wed: ("0 0 * * Wed", 1573151292, 1573603200),
    every_dow_thu: ("0 0 * * Thu", 1573151292, 1573689600),
    every_dow_fri: ("0 0 * * Fri", 1573151292, 1573171200),
    every_dow_sat: ("0 0 * * Sat", 1573151292, 1573257600),
    every_dow_wed_and_fri: ("0 0 * * Wed,Fri", 1573151292, 1573171200),
    dow_feb: ("0 0 29 2 6", 1573151292, 1582934400),
    every_dow_wed_2_fri: ("0 0 * * Wed-Fri", 1573151292, 1573171200),
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
    assert!(parse("* * 30 2 *", Utc::now()).is_err());
}

#[test]
fn test_parse() {
    assert!(parse("*/5 * * * *", Utc::now()).is_ok());
    assert!(parse("0 0 29 2 5", Utc.timestamp(1573151292, 0)).is_err());
    assert!(parse("0 0 * * */Wed", Utc::now()).is_err());
    assert!(parse("0 0 * * */2-5", Utc::now()).is_err());
}

#[test]
fn test_bad_input() {
    assert!(parse("2-3,9,*/15,1-8,11,9,4,5, * * * *", Utc::now()).is_ok());
    assert!(parse("2-3,9,*/15,1-8,11,9,4,5,,,, * * * *", Utc::now()).is_ok());
}

#[test]
fn test_next_100_iterations() {
    let now = Utc.timestamp(1573239864, 0);
    let mut next = parse("0 23 */2 * *", now).unwrap();
    assert_eq!(next.timestamp(), 1573340400);
    for _ in 0..100 {
        next = parse("0 23 */2 * *", next).unwrap();
    }
    assert_eq!(next.timestamp(), 1590274800);
}

#[test]
fn test_timezone() {
    use chrono_tz::US::Pacific;
    let utc = Utc.timestamp(1573405861, 0);
    let pacific_time = utc.with_timezone(&Pacific);
    let next_pt = parse("*/5 * * * *", pacific_time).unwrap();
    assert_eq!(next_pt.timestamp(), 1573406100);
    let next_utc = parse("*/5 * * * *", utc).unwrap();
    assert_eq!(next_utc.timestamp(), 1573406100);
    assert_ne!(next_pt.to_string(), next_utc.to_string());
}

#[test]
fn test_timezone_dst() {
    // 2018-11-04 1:30
    use chrono_tz::America::Chicago;
    let utc = Utc.timestamp(1541309400, 0);
    let cst = utc.with_timezone(&Chicago);
    let mut next = parse("*/15 * * * *", cst).unwrap();
    for _ in 0..10 {
        next = parse("*/15 * * * *", next).unwrap();
    }
    assert_eq!(next.timestamp(), 1541322900);
}
