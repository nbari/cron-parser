use chrono;
use std::{error::Error, fmt, num};

#[derive(Debug)]
pub enum ParseError {
    InvalidMinute,
    InvalidSyntax,
    InvalidRange,
    InvalidValue,
    ParseIntError(num::ParseIntError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::InvalidMinute => write!(f, "invalid minute"),
            ParseError::InvalidSyntax => write!(f, "invalid minute"),
            ParseError::InvalidValue => write!(f, "invalid value"),
            ParseError::InvalidRange => write!(f, "wrong input. Hyphens define ranges. For example, 2000–2010 indicates every year between 2000 and 2010, inclusive."),
            ParseError::ParseIntError(ref err) => err.fmt(f),
        }
    }
}
impl Error for ParseError {}

impl From<num::ParseIntError> for ParseError {
    fn from(err: num::ParseIntError) -> Self {
        ParseError::ParseIntError(err)
    }
}

/// Parse cron syntax
/// ```text
/// ┌───────────── minute (0 - 59)
/// │ ┌───────────── hour (0 - 23)
/// │ │ ┌───────────── day of the month (1 - 31)
/// │ │ │ ┌───────────── month (1 - 12)
/// │ │ │ │ ┌───────────── day of the week (0 - 6)
/// │ │ │ │ │
/// │ │ │ │ │
/// │ │ │ │ │
/// * * * * * command to execute
/// ```
pub fn parse(cron: &str) -> Result<(), ParseError> {
    let next = chrono::Local::now() + chrono::Duration::minutes(1);
    let fields: Vec<&str> = cron.split_whitespace().collect();
    if fields.len() > 5 {
        return Err(ParseError::InvalidSyntax);
    }

    Ok(())
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
///
/// fn main() {
///      // every 3 months
///      assert_eq!(parse_field("*/3", 1, 12).unwrap(), vec![1,4,7,10]);
///      // day 31
///      assert_eq!(parse_field("31", 1, 31).unwrap(), vec![31]);
///      // every minute from 40 through 50
///      assert_eq!(parse_field("40-50", 0, 59).unwrap(), vec![40,41,42,43,44,45,46,47,48,49,50]);
///      // at hour 3,15,23
///      assert_eq!(parse_field("15,3,23", 0, 23).unwrap(), vec![3,15,23]);
/// }
/// ```
pub fn parse_field(field: &str, min: usize, max: usize) -> Result<Vec<usize>, ParseError> {
    match field {
        // any value
        "*" => Ok(Vec::<usize>::new()),
        // step values
        f if f.starts_with("*/") => {
            let f: usize = field.trim_start_matches("*/").parse()?;
            if f > max {
                return Err(ParseError::InvalidValue);
            }
            Ok((min..max + 1).step_by(f).collect::<Vec<usize>>())
        }
        // value list separator
        f if f.contains(",") => {
            let mut fields: Vec<usize> = field
                .split(',')
                .map(|field| field.parse::<usize>())
                .collect::<Result<_, _>>()?;
            fields.sort();
            if fields[fields.len() - 1] > max {
                return Err(ParseError::InvalidValue);
            }
            Ok(fields)
        }
        // range of values
        f if f.contains("-") => {
            let fields: Vec<usize> = field
                .split('-')
                .map(|field| field.parse::<usize>())
                .collect::<Result<_, _>>()?;
            if fields.len() != 2 || fields[0] > fields[1] || fields[1] > max {
                return Err(ParseError::InvalidRange);
            }
            Ok((fields[0]..fields[1] + 1).collect::<Vec<usize>>())
        }
        _ => {
            // single int field
            let f = field.parse::<usize>()?;
            if f > max {
                return Err(ParseError::InvalidValue);
            }
            Ok(vec![f])
        }
    }
}
