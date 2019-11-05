use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use std::{collections::BTreeSet, convert::TryFrom, error::Error, fmt, num};

#[derive(Debug)]
pub enum ParseError {
    InvalidMinute,
    InvalidSyntax,
    InvalidRange,
    InvalidValue,
    ParseIntError(num::ParseIntError),
    TryFromIntError(num::TryFromIntError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::InvalidMinute => write!(f, "invalid minute"),
            ParseError::InvalidSyntax => write!(f, "invalid minute"),
            ParseError::InvalidValue => write!(f, "invalid value"),
            ParseError::InvalidRange => write!(f, "wrong input. Hyphens define ranges. For example, 2000–2010 indicates every year between 2000 and 2010, inclusive."),
            ParseError::ParseIntError(ref err) => err.fmt(f),
            ParseError::TryFromIntError(ref err)  => err.fmt(f),
        }
    }
}
impl Error for ParseError {}

impl From<num::ParseIntError> for ParseError {
    fn from(err: num::ParseIntError) -> Self {
        ParseError::ParseIntError(err)
    }
}

impl From<num::TryFromIntError> for ParseError {
    fn from(err: num::TryFromIntError) -> Self {
        ParseError::TryFromIntError(err)
    }
}

/// Parse cron syntax
/// ```text
///                                                   index
/// ┌───────────── minute (0 - 59)                    0
/// │ ┌───────────── hour (0 - 23)                    1
/// │ │ ┌───────────── day of the month (1 - 31)      2
/// │ │ │ ┌───────────── month (1 - 12)               3
/// │ │ │ │ ┌───────────── day of the week (0 - 6)    4
/// │ │ │ │ │
/// │ │ │ │ │
/// │ │ │ │ │
/// * * * * * command to execute
/// ```
///
/// Example
/// ```
/// use cron_parser::parse;
///
/// fn main() {
///     assert!(parse("*/5 * * * *").is_ok(), "todo");
/// }
/// ```
pub fn parse(cron: &str) -> Result<DateTime<Local>, ParseError> {
    let mut next = chrono::Local::now() + chrono::Duration::minutes(1);
    let fields: Vec<&str> = cron.split_whitespace().collect();
    if fields.len() > 5 {
        return Err(ParseError::InvalidSyntax);
    }

    // get month
    let month = parse_field(fields[3], 1, 12)?;
    if !month.is_empty() {
        println!("----> NO EMPTY{:?} {}", month, next);
    }

    // get day
    let day = parse_field(fields[2], 1, 31)?;
    if !day.is_empty() {
        println!("----> NO EMPTY{:?} {}", day, next);
    }

    // get hour
    let hour = parse_field(fields[1], 0, 23)?;
    if !hour.is_empty() {
        println!("----> NO EMPTY{:?} {}", hour, next);
    }

    // get minute
    let minutes = parse_field(fields[0], 0, 59)?;
    if !minutes.is_empty() {
        next = exp_minute(minutes, next)?;
    }

    println!("{}", next);
    Ok(next)
}

pub fn exp_minute(
    minutes: BTreeSet<u32>,
    dt: DateTime<Local>,
) -> Result<DateTime<Local>, ParseError> {
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
        chrono::Local
            .ymd(dt.year(), dt.month(), dt.day())
            .and_hms(dt.hour(), next_minute, 0);
    if next_minute != dt.minute() {
        if dt.minute() > next_minute {
            next_dt = next_dt + chrono::Duration::hours(1);
        }
    }
    Ok(next_dt)
}

/// parse_field
/// Allowed special characters:
/// * `*` any value
/// * `,` value list separator
/// * `-` range of values
/// * `/` step values
///
/// ```text
/// minutes min: 0, max: 59
/// hours   min: 0, max: 23
/// days    min: 1, max: 31
/// month   min: 1, max: 12
/// dweek   min: 0, max: 6
/// ```
///
/// Example
/// ```
/// use cron_parser::parse_field;
/// use std::collections::BTreeSet;
///
/// fn main() {
///      // every 3 months
///      // assert_eq!(parse_field("*/3", 1, 12).unwrap(), vec![1,4,7,10]);
///      assert_eq!(parse_field("*/3", 1, 12).unwrap(),
///      BTreeSet::<u32>::from([1,4,7,10].iter().cloned().collect()));
///      // day 31
///      assert_eq!(parse_field("31", 1, 31).unwrap(),
///      BTreeSet::<u32>::from([31].iter().cloned().collect()));
///      // every minute from 40 through 50
///      assert_eq!(parse_field("40-50", 0, 59).unwrap(),
///      BTreeSet::<u32>::from([40,41,42,43,44,45,46,47,48,49,50].iter().cloned().collect()));
///      // at hour 3,15,23
///      assert_eq!(parse_field("15,3,23", 0, 23).unwrap(),
///      BTreeSet::<u32>::from([3,15,23].iter().cloned().collect()));
/// }
/// ```
pub fn parse_field(field: &str, min: u32, max: u32) -> Result<BTreeSet<u32>, ParseError> {
    // set of integers
    let mut values = BTreeSet::<u32>::new();

    // The field column can have a * or a list of elements separated by commas.
    // An element is either a number in the ranges or two numbers in the range
    // separated by a hyphen. slashes can be combined with ranges to specify
    // step values
    let fields: Vec<&str> = field.split(",").collect();

    let mut iterator = fields.into_iter();
    loop {
        match iterator.next() {
            Some(field) => match field {
                "*" => continue,
                // step values
                f if field.starts_with("*/") => {
                    let f: u32 = f.trim_start_matches("*/").parse()?;
                    if f > max {
                        return Err(ParseError::InvalidValue);
                    }
                    let step = usize::try_from(f)?;
                    for i in (min..max + 1).step_by(step).collect::<Vec<u32>>() {
                        values.insert(i);
                    }
                }
                // range of values
                f if f.contains("-") => {
                    let fields: Vec<u32> = f
                        .split('-')
                        .map(|field| field.parse::<u32>())
                        .collect::<Result<_, _>>()?;
                    if fields.len() != 2 || fields[0] > fields[1] || fields[1] > max {
                        return Err(ParseError::InvalidRange);
                    }
                    for i in (fields[0]..fields[1] + 1).collect::<Vec<u32>>() {
                        values.insert(i);
                    }
                }
                _ => {
                    let f = field.parse::<u32>()?;
                    if f > max {
                        return Err(ParseError::InvalidValue);
                    }
                    values.insert(f);
                }
            },
            None => break,
        }
    }
    Ok(values)
}
