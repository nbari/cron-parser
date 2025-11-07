//! Library for parsing cron expressions with timezone support.
//!
//! Example:
//! ```
//! use chrono::{TimeZone, Utc};
//! use chrono_tz::Europe::Lisbon;
//! use cron_parser::parse;
//!
//! if let Ok(next) = parse("*/5 * * * *", &Utc::now()) {
//!      println!("when: {}", next);
//! }
//!
//! // every 6 hours starting at 1:00
//! if let Ok(next) = parse("0 1/6 * * *", &Utc::now()) {
//!      println!("when: {}", next);
//! }
//!
//! // passing a custom timestamp
//! if let Ok(next) = parse("0 0 29 2 *", &Utc.timestamp_opt(1893456000, 0).unwrap()) {
//!      println!("next leap year: {}", next);
//!      assert_eq!(next.timestamp(), 1961625600);
//! }
//!
//! assert!(parse("2-3,9,*/15,1-8,11,9,4,5 * * * *", &Utc::now()).is_ok());
//! assert!(parse("* * * * */Fri", &Utc::now()).is_err());
//!
//! // use custom timezone
//! assert!(parse("*/5 * * * *", &Utc::now().with_timezone(&Lisbon)).is_ok());
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
    InvalidTimezone,
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
            Self::InvalidTimezone => write!(f, "invalid timezone"),
        }
    }
}

impl Error for ParseError {}

impl From<num::ParseIntError> for ParseError {
    fn from(err: num::ParseIntError) -> Self {
        Self::ParseIntError(err)
    }
}

impl From<num::TryFromIntError> for ParseError {
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
/// assert!(parse("*/5 * * * *", &Utc::now()).is_ok());
///
/// // use custom timezone
/// use chrono_tz::US::Pacific;
/// assert!(parse("*/5 * * * *", &Utc::now().with_timezone(&Pacific)).is_ok());
/// ```
/// # Errors
/// [`ParseError`](enum.ParseError.html)
pub fn parse<TZ: TimeZone>(cron: &str, dt: &DateTime<TZ>) -> Result<DateTime<TZ>, ParseError> {
    let tz = dt.timezone();

    let fields: Vec<&str> = cron.split_whitespace().collect();

    if fields.len() != 5 {
        return Err(ParseError::InvalidCron);
    }

    let mut next = match Utc.from_local_datetime(&dt.naive_local()) {
        chrono::LocalResult::Single(datetime) => datetime + Duration::minutes(1),
        chrono::LocalResult::Ambiguous(earlier, _later) => earlier + Duration::minutes(1),
        chrono::LocalResult::None => return Err(ParseError::InvalidTimezone),
    };

    next = match Utc.with_ymd_and_hms(
        next.year(),
        next.month(),
        next.day(),
        next.hour(),
        next.minute(),
        0,
    ) {
        chrono::LocalResult::Single(datetime) => datetime,
        chrono::LocalResult::Ambiguous(earlier, _later) => earlier,
        chrono::LocalResult::None => return Err(ParseError::InvalidTimezone),
    };

    let result = loop {
        // only try until next leap year
        if next.year() - dt.year() > 4 {
            return Err(ParseError::InvalidCron);
        }

        // * * * <month> *
        let month = parse_field(fields[3], 1, 12)?;
        if !month.contains(&next.month()) {
            next = match Utc.with_ymd_and_hms(
                if next.month() == 12 {
                    next.year() + 1
                } else {
                    next.year()
                },
                if next.month() == 12 {
                    1
                } else {
                    next.month() + 1
                },
                1,
                0,
                0,
                0,
            ) {
                chrono::LocalResult::Single(datetime) => datetime,
                chrono::LocalResult::Ambiguous(earlier, _later) => earlier,
                chrono::LocalResult::None => return Err(ParseError::InvalidTimezone),
            };
            continue;
        }

        // * * <dom> * *
        let do_m = parse_field(fields[2], 1, 31)?;
        if !do_m.contains(&next.day()) {
            next += Duration::days(1);
            next = match Utc.with_ymd_and_hms(next.year(), next.month(), next.day(), 0, 0, 0) {
                chrono::LocalResult::Single(datetime) => datetime,
                chrono::LocalResult::Ambiguous(earlier, _later) => earlier,
                chrono::LocalResult::None => return Err(ParseError::InvalidTimezone),
            };
            continue;
        }

        // * <hour> * * *
        let hour = parse_field(fields[1], 0, 23)?;
        if !hour.contains(&next.hour()) {
            next += Duration::hours(1);
            next = match Utc.with_ymd_and_hms(
                next.year(),
                next.month(),
                next.day(),
                next.hour(),
                0,
                0,
            ) {
                chrono::LocalResult::Single(datetime) => datetime,
                chrono::LocalResult::Ambiguous(earlier, _later) => earlier,
                chrono::LocalResult::None => return Err(ParseError::InvalidTimezone),
            };
            continue;
        }

        // <minute> * * * *
        let minute = parse_field(fields[0], 0, 59)?;
        if !minute.contains(&next.minute()) {
            next += Duration::minutes(1);
            continue;
        }

        // * * * * <dow>
        let do_w = parse_field(fields[4], 0, 6)?;
        if !do_w.contains(&next.weekday().num_days_from_sunday()) {
            next += Duration::days(1);
            continue;
        }

        // Valid datetime for the timezone
        match tz.from_local_datetime(&next.naive_local()) {
            chrono::LocalResult::Single(dt) => break dt,
            chrono::LocalResult::Ambiguous(earlier, _later) => break earlier,
            chrono::LocalResult::None => {
                next += Duration::minutes(1);
                continue;
            }
        }
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
///  // every 3 months
///  assert_eq!(parse_field("*/3", 1, 12).unwrap(),
///  BTreeSet::<u32>::from([1,4,7,10].iter().cloned().collect::<BTreeSet<u32>>()));
///
///  // day 31
///  assert_eq!(parse_field("31", 1, 31).unwrap(),
///  BTreeSet::<u32>::from([31].iter().cloned().collect::<BTreeSet<u32>>()));
///
///  // every minute from 40 through 50
///  assert_eq!(parse_field("40-50", 0, 59).unwrap(),
///  BTreeSet::<u32>::from([40,41,42,43,44,45,46,47,48,49,50].iter().cloned().collect::<BTreeSet<u32>>()));
///
///  // at hour 3,15,23
///  assert_eq!(parse_field("15,3,23", 0, 23).unwrap(),
///  BTreeSet::<u32>::from([3,15,23].iter().cloned().collect::<BTreeSet<u32>>()));
/// ```
///
/// Parses a cron field, supporting formats like:
/// `*/N`, `<start>/<step>`, ranges (`min-max`), and lists (`1,2,3`).
///
/// # Errors
/// [`ParseError`](enum.ParseError.html)
pub fn parse_field(field: &str, min: u32, max: u32) -> Result<BTreeSet<u32>, ParseError> {
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
            f if f.starts_with("*/") => {
                let step: u32 = f.trim_start_matches("*/").parse()?;

                if step == 0 || step > max {
                    return Err(ParseError::InvalidValue);
                }

                for i in (min..=max).step_by(step as usize) {
                    values.insert(i);
                }
            }

            // step with range, eg: 12-18/2
            f if f.contains('/') => {
                let tmp_fields: Vec<&str> = f.split('/').collect();

                if tmp_fields.len() != 2 {
                    return Err(ParseError::InvalidRange);
                }

                // get the step, eg: 2 from 12-18/2
                let step: u32 = tmp_fields[1].parse()?;

                if step == 0 || step > max {
                    return Err(ParseError::InvalidValue);
                }

                // check for range, eg: 12-18
                if tmp_fields[0].contains('-') {
                    let tmp_range: Vec<&str> = tmp_fields[0].split('-').collect();

                    if tmp_range.len() != 2 {
                        return Err(ParseError::InvalidRange);
                    }

                    let start = parse_cron_value(tmp_range[0], min, max)?;

                    let end = parse_cron_value(tmp_range[1], min, max)?;

                    if start > end {
                        return Err(ParseError::InvalidRange);
                    }

                    for i in (start..=end).step_by(step as usize) {
                        values.insert(i);
                    }
                } else {
                    let start = parse_cron_value(tmp_fields[0], min, max)?;

                    for i in (start..=max).step_by(step as usize) {
                        values.insert(i);
                    }
                }
            }

            // range of values, it can have days of week like Wed-Fri
            f if f.contains('-') => {
                let tmp_fields: Vec<&str> = f.split('-').collect();

                if tmp_fields.len() != 2 {
                    return Err(ParseError::InvalidRange);
                }

                let start = parse_cron_value(tmp_fields[0], min, max)?;

                let end = parse_cron_value(tmp_fields[1], min, max)?;

                if start > end {
                    return Err(ParseError::InvalidRange);
                }
                for i in start..=end {
                    values.insert(i);
                }
            }

            // integers or days of week any other will return an error
            _ => {
                let value = parse_cron_value(field, min, max)?;
                values.insert(value);
            }
        }
    }

    Ok(values)
}

// helper function to parse cron values
fn parse_cron_value(value: &str, min: u32, max: u32) -> Result<u32, ParseError> {
    if let Ok(dow) = Dow::from_str(value) {
        Ok(dow as u32)
    } else {
        let v: u32 = value.parse()?;
        if v < min || v > max {
            return Err(ParseError::InvalidValue);
        }
        Ok(v)
    }
}
