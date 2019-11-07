use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Timelike, Utc};
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
            ParseError::InvalidRange => write!(f, "invalid input"),
            ParseError::ParseIntError(ref err) => err.fmt(f),
            ParseError::TryFromIntError(ref err) => err.fmt(f),
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
///
/// ┌─────────────────────  minute (0 - 59)
/// │ ┌───────────────────  hour   (0 - 23)
/// │ │ ┌─────────────────  dom    (1 - 31) day of month
/// │ │ │ ┌───────────────  month  (1 - 12)
/// │ │ │ │ ┌─────────────  dow    (0 - 6)  day of week (Sunday to Saturday)
/// │ │ │ │ │
/// │ │ │ │ │
/// │ │ │ │ │
/// * * * * * command to execute
/// ```
///
/// Example
/// ```
/// use cron_parser::parse;
/// use chrono::Utc;
///
/// fn main() {
///     assert!(parse("*/5 * * * *", Utc::now()).is_ok());
/// }
/// ```
pub fn parse(cron: &str, dt: DateTime<Utc>) -> Result<DateTime<Utc>, ParseError> {
    let mut next = dt + Duration::minutes(1);
    let fields: Vec<&str> = cron.split_whitespace().collect();
    if fields.len() > 5 {
        return Err(ParseError::InvalidSyntax);
    }

    next = Utc
        .ymd(next.year(), next.month(), next.day())
        .and_hms(next.hour(), next.minute(), 0);
    loop {
        // * * * * <dow>
        let dow = parse_field(fields[4], 0, 6)?;
        if !dow.contains(&next.weekday().num_days_from_sunday()) {
            next = next + Duration::days(1);
            continue;
        }

        // * * * * <month>
        let month = parse_field(fields[3], 1, 12)?;
        if !month.contains(&next.month()) {
            if next.month() == 12 {
                next = Utc.ymd(next.year() + 1, 1, 1).and_hms(0, 0, 0);
            } else {
                next = Utc.ymd(next.year(), next.month() + 1, 1).and_hms(0, 0, 0);
            }
            continue;
        }

        // * * * <dom> *
        let dom = parse_field(fields[2], 1, 31)?;
        if !dom.contains(&next.day()) {
            next = next + Duration::days(1);
            next = Utc
                .ymd(next.year(), next.month(), next.day())
                .and_hms(0, 0, 0);
            // TODO prevent loop
            let days_in_month = if next.month() == 12 {
                NaiveDate::from_ymd(next.year() + 1, 1, 1)
            } else {
                NaiveDate::from_ymd(next.year(), next.month() + 1, 1)
            }
            .signed_duration_since(NaiveDate::from_ymd(next.year(), next.month(), 1))
            .num_days() as u32;
            if *dom.iter().nth(0).unwrap() > days_in_month {
                //return Err(ParseError::InvalidValue);
            }
            continue;
        }

        // * <hour> * * *
        let hour = parse_field(fields[1], 0, 23)?;
        if !hour.contains(&next.hour()) {
            next = next + Duration::hours(1);
            next = Utc
                .ymd(next.year(), next.month(), next.day())
                .and_hms(next.hour(), 0, 0);
            continue;
        }

        // <minute> * * * *
        let minute = parse_field(fields[0], 0, 59)?;
        if !minute.contains(&next.minute()) {
            next = next + Duration::minutes(1);
            continue;
        }

        break;
    }

    println!("{}\n{}", dt, next);
    Ok(next)
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
/// The field column can have a `*` or a list of elements separated by commas.
/// An element is either a number in the ranges or two numbers in the range
/// separated by a hyphen, slashes can be combined with ranges to specify
/// step values
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
    let fields: Vec<&str> = field.split(',').collect();
    for field in fields.into_iter() {
        match field {
            "*" => {
                for i in min..=max {
                    values.insert(i);
                }
            }
            // step values
            f if field.starts_with("*/") => {
                let f: u32 = f.trim_start_matches("*/").parse()?;
                if f > max {
                    return Err(ParseError::InvalidValue);
                }
                let step = usize::try_from(f)?;
                for i in (min..=max).step_by(step).collect::<Vec<u32>>() {
                    values.insert(i);
                }
            }
            // range of values
            f if f.contains('-') => {
                let fields: Vec<u32> = f
                    .split('-')
                    .map(|field| field.parse::<u32>())
                    .collect::<Result<_, _>>()?;
                if fields.len() != 2 || fields[0] > fields[1] || fields[1] > max {
                    return Err(ParseError::InvalidRange);
                }
                for i in (fields[0]..=fields[1]).collect::<Vec<u32>>() {
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
        }
    }
    Ok(values)
}
