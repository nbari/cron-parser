use chrono;
use std::{error::Error, fmt, num};

#[derive(Debug)]
pub enum ParseError {
    InvalidMinute,
    InvalidSyntax,
    InvalidRange,
    ParseIntError(num::ParseIntError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::InvalidMinute => write!(f, "invalid minute"),
            ParseError::InvalidSyntax => write!(f, "invalid minute"),
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

fn parse_field(field: &str, min: isize, max: isize) -> Result<Vec<isize>, ParseError> {
    match field {
        "*" => Ok(vec![-1]),
        f if f.starts_with("*/") => {
            let f: usize = field.trim_start_matches("*/").parse()?;
            Ok((min..max).step_by(f).collect::<Vec<isize>>())
        }
        f if f.contains(",") => {
            let fields: Vec<isize> = field
                .split(',')
                .map(|field| field.parse::<isize>())
                .collect::<Result<_, _>>()?;
            Ok(fields)
        }
        f if f.contains("-") => {
            let fields: Vec<isize> = field
                .split('-')
                .map(|field| field.parse::<isize>())
                .collect::<Result<_, _>>()?;
            if fields.len() != 2 || fields[0] > fields[1] {
                return Err(ParseError::InvalidRange);
            }
            Ok((fields[0]..fields[1]).collect::<Vec<isize>>())
        }
        _ => {
            // single int field
            let f = field.parse::<isize>()?;
            Ok(vec![f])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        parse_0: ("1", 0, 0, vec![1]),
        parse_1: ("*", 0, 0, vec![-1]),
        parse_2: ("*/5", 0, 60, vec![0,5,10,15,20,25,30,35,40,45,50,55]),
        parse_3: ("*/30", 0, 60, vec![0,30]),
        parse_4: ("*/5", 0, 0, Vec::<isize>::new()),
        parse_5: ("5-10", 0, 0, vec![5,6,7,8,9]),
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
}
