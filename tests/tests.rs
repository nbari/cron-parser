use cron_parser::parse_field;

macro_rules! parse_field_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, min, max, expected) = $value;
                    assert_eq!(expected, parse_field(input, min, max).unwrap());
                }
            )*
        }
    }

parse_field_tests! {
    parse_any:("*", 0, 0, Vec::<usize>::new()),
    parse_minutes_0: ("0", 0, 59, vec![0]),
    parse_minutes_1: ("1", 0, 59, vec![1]),
    parse_hours: ("23", 0, 23, vec![23]),
    parse_days: ("31", 0, 31, vec![31]),
    parse_day_week: ("6", 0, 6,  vec![6]),
    parse_every_30: ("*/30", 0, 59, vec![0,30]),
    parse_every_5_minutes: ("*/5", 0, 59, vec![0,5,10,15,20,25,30,35,40,45,50,55]),
    parse_every_minute: ("*/1", 0, 59, vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59]),
    parse_every_hour: ("*/1", 0, 23, vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23]),
    parse_every_day: ("*/1", 1, 31, vec![1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31]),
    parse_every_month: ("*/1", 1, 12, vec![1,2,3,4,5,6,7,8,9,10,11,12]),
    parse_every_dweek: ("*/1", 0, 6, vec![0,1,2,3,4,5,6]),
    parse_range_5_10_minutes: ("5-10", 0, 59, vec![5,6,7,8,9,10]),
    parse_range_5_10_hours: ("5-10", 0, 12, vec![5,6,7,8,9,10]),
    parse_list_minutes: ("15,30,45,0", 0, 59, vec![0,15,30,45]),
    parse_8: ("1024", 0, 1024, vec![1024]),
}

#[test]
fn parse_field_double_field() {
    assert!(
        parse_field("**", 0, 0).is_err(),
        "should thrown error ParseIntError, invalid digit"
    );
}

#[test]
fn parse_field_bad_range() {
    assert!(
        parse_field("1-2-3", 0, 0).is_err(),
        "should thrown error ParseIntError, invalid digit"
    );
    assert!(
        parse_field("8-5", 0, 0).is_err(),
        "should thrown error ParseIntError, invalid digit"
    );
}

#[test]
fn bad_minute_input() {
    assert!(
        parse_field("60", 0, 59).is_err(),
        "should thrown error ParseIntError, invalid digit"
    );
}

#[test]
fn bad_minute_input_range() {
    assert!(
        parse_field("5-60", 0, 59).is_err(),
        "should thrown error ParseIntError, invalid digit"
    );
}

#[test]
fn bad_minute_input_list() {
    assert!(
        parse_field("40,50,60", 0, 59).is_err(),
        "should thrown error ParseIntError, invalid digit"
    );
}
#[test]
fn bad_hour_input_step() {
    assert!(
        parse_field("*/30", 0, 23).is_err(),
        "should thrown error ParseIntError, invalid digit"
    );
}
