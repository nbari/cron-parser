//! Library for parsing cron expressions with timezone support.
//!
//! Example:
//! ```
//! use chrono::{TimeZone, Utc};
//! use chrono_tz::Europe::Lisbon;
//! use cron_parser::parse;
//!
//! fn main() {
//!    if let Ok(next) = parse("*/5 * * * *", &Utc::now()) {
//!         println!("when: {}", next);
//!    }
//!
//!    // passing a custom timestamp
//!    if let Ok(next) = parse("0 0 29 2 *", &Utc.timestamp(1893456000, 0)) {
//!         println!("next leap year: {}", next);
//!         assert_eq!(next.timestamp(), 1961625600);
//!    }
//!
//!    assert!(parse("2-3,9,*/15,1-8,11,9,4,5 * * * *", &Utc::now()).is_ok());
//!    assert!(parse("* * * * */Fri", &Utc::now()).is_err());
//!
//!    // use custom timezone
//!    assert!(parse("*/5 * * * *", &Utc::now().with_timezone(&Lisbon)).is_ok());
//! }
//! ```
use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use std::{collections::BTreeSet, error::Error, fmt, num, str::FromStr};

#[derive(Debug)]
pub enum ParseError {
    InvalidCron,
    InvalidRange,
    InvalidValue,
    ParseIntError(num::ParseIntError),
    TryFromIntError(num::TryFromIntError),
}

enum Dow {
    Sun = 0,
    Mon = 1,
    Tue = 2,
    Wed = 3,
    Thu = 4,
    Fri = 5,
    Sat = 6,
}

impl FromStr for Dow {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*s.to_uppercase() {
            "SUN" => Ok(Self::Sun),
            "MON" => Ok(Self::Mon),
            "TUE" => Ok(Self::Tue),
            "WED" => Ok(Self::Wed),
            "THU" => Ok(Self::Thu),
            "FRI" => Ok(Self::Fri),
            "SAT" => Ok(Self::Sat),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::InvalidCron => write!(f, "invalid cron"),
            Self::InvalidRange => write!(f, "invalid input"),
            Self::InvalidValue => write!(f, "invalid value"),
            Self::ParseIntError(ref err) => err.fmt(f),
            Self::TryFromIntError(ref err) => err.fmt(f),
        }
    }
}
impl Error for ParseError {}

impl From<num::ParseIntError> for ParseError {
    #[must_use]
    fn from(err: num::ParseIntError) -> Self {
        Self::ParseIntError(err)
    }
}

impl From<num::TryFromIntError> for ParseError {
    #[must_use]
    fn from(err: num::TryFromIntError) -> Self {
        Self::TryFromIntError(err)
    }
}

/// Parse cron expression
/// ```text
///
/// ┌─────────────────────  minute (0 - 59)
/// │ ┌───────────────────  hour   (0 - 23)
/// │ │ ┌─────────────────  dom    (1 - 31) day of month
/// │ │ │ ┌───────────────  month  (1 - 12)
/// │ │ │ │ ┌─────────────  dow    (0 - 6 or Sun - Sat) day of week (Sunday to Saturday)
/// │ │ │ │ │
/// │ │ │ │ │
/// │ │ │ │ │
/// * * * * * <command to execute>
/// ```
///
/// Example
/// ```
/// use cron_parser::parse;
/// use chrono::Utc;
///
/// fn main() {
///     assert!(parse("*/5 * * * *", &Utc::now()).is_ok());
///
///     // use custom timezone
///     use chrono_tz::US::Pacific;
///     assert!(parse("*/5 * * * *", &Utc::now().with_timezone(&Pacific)).is_ok());
/// }
/// ```
pub fn parse<TZ: TimeZone>(cron: &str, dt: &DateTime<TZ>) -> Result<DateTime<TZ>, ParseError> {
    let tz = dt.timezone();
    // TODO handle unwrap
    let mut next = Utc.from_local_datetime(&dt.naive_local()).unwrap() + Duration::minutes(1);
    let fields: Vec<&str> = cron.split_whitespace().collect();
    if fields.len() > 5 {
        return Err(ParseError::InvalidCron);
    }

    next = Utc
        .ymd(next.year(), next.month(), next.day())
        .and_hms(next.hour(), next.minute(), 0);

    let result = loop {
        // only try until next leap year
        if next.year() - dt.year() > 4 {
            return Err(ParseError::InvalidCron);
        }

        // * * * <month> *
        let month = parse_field(fields[3], 1, 12)?;
        if !month.contains(&next.month()) {
            if next.month() == 12 {
                next = Utc.ymd(next.year() + 1, 1, 1).and_hms(0, 0, 0);
            } else {
                next = Utc.ymd(next.year(), next.month() + 1, 1).and_hms(0, 0, 0);
            }
            continue;
        }

        // * * <dom> * *
        let do_m = parse_field(fields[2], 1, 31)?;
        if !do_m.contains(&next.day()) {
            next = next + Duration::days(1);
            next = Utc
                .ymd(next.year(), next.month(), next.day())
                .and_hms(0, 0, 0);
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

        // * * * * <dow>
        let do_w = parse_field(fields[4], 0, 6)?;
        if !do_w.contains(&next.weekday().num_days_from_sunday()) {
            next = next + Duration::days(1);
            continue;
        }

        // Valid datetime for the timezone
        if let Some(dt) = tz.from_local_datetime(&next.naive_local()).latest() {
            break dt;
        }

        next = next + Duration::minutes(1);
    };

    Ok(result)
}

/// `parse_field`
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
/// dow     min: 0, max: 6 or min: Sun, max Sat
///
/// Day of week (dow):
///    Sun = 0
///    Mon = 1
///    Tue = 2
///    Wed = 3
///    Thu = 4
///    Fri = 5
///    Sat = 6
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
///      assert_eq!(parse_field("*/3", 1, 12).unwrap(),
///      BTreeSet::<u32>::from([1,4,7,10].iter().cloned().collect()));
///
///      // day 31
///      assert_eq!(parse_field("31", 1, 31).unwrap(),
///      BTreeSet::<u32>::from([31].iter().cloned().collect()));
///
///      // every minute from 40 through 50
///      assert_eq!(parse_field("40-50", 0, 59).unwrap(),
///      BTreeSet::<u32>::from([40,41,42,43,44,45,46,47,48,49,50].iter().cloned().collect()));
///
///      // at hour 3,15,23
///      assert_eq!(parse_field("15,3,23", 0, 23).unwrap(),
///      BTreeSet::<u32>::from([3,15,23].iter().cloned().collect()));
/// }
/// ```
pub fn parse_field(field: &str, min: u32, max: u32) -> Result<BTreeSet<u32>, ParseError> {
    // set of integers
    let mut values = BTreeSet::<u32>::new();

    // split fields by ','
    let fields: Vec<&str> = field.split(',').filter(|s| !s.is_empty()).collect();

    // iterate over the fields and match against allowed characters
    for field in fields {
        match field {
            // any
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
                for i in (min..=max).step_by(f as usize).collect::<Vec<u32>>() {
                    values.insert(i);
                }
            }
            // range of values, it can have days of week like Wed-Fri
            f if f.contains('-') => {
                let tmp_fields: Vec<&str> = f.split('-').collect();
                if tmp_fields.len() != 2 {
                    return Err(ParseError::InvalidRange);
                }

                let mut fields: Vec<u32> = Vec::new();

                if let Ok(dow) = Dow::from_str(tmp_fields[0]) {
                    fields.push(dow as u32);
                } else {
                    fields.push(tmp_fields[0].parse::<u32>()?);
                };

                if let Ok(dow) = Dow::from_str(tmp_fields[1]) {
                    fields.push(dow as u32);
                } else {
                    fields.push(tmp_fields[1].parse::<u32>()?);
                }

                if fields[0] > fields[1] || fields[1] > max {
                    return Err(ParseError::InvalidRange);
                }
                for i in (fields[0]..=fields[1]).collect::<Vec<u32>>() {
                    values.insert(i);
                }
            }
            // integers or days of week any other will return an error
            _ => {
                if let Ok(dow) = Dow::from_str(field) {
                    values.insert(dow as u32);
                } else {
                    let f = field.parse::<u32>()?;
                    if f > max {
                        return Err(ParseError::InvalidValue);
                    }
                    values.insert(f);
                }
            }
        }
    }
    Ok(values)
}
