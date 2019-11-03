use chrono;
use std::{error::Error, fmt, num};

#[derive(Debug)]
pub enum ParseError {
    InvalidMinute,
    InvalidSyntax,
    ParseIntError(num::ParseIntError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::InvalidMinute => write!(f, "invalid minute"),
            ParseError::InvalidSyntax => write!(f, "invalid minute"),
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
        "*/" => Ok((min..max).collect::<Vec<isize>>()),
        _ => {
            if field.contains(",") {
                let fields: Vec<isize> = field
                    .split(',')
                    .map(|field| field.parse::<isize>())
                    .collect::<Result<_, _>>()?;
                Ok(fields)
            } else if field.contains("-") {
                let fields: Vec<isize> = field
                    .split('-')
                    .map(|field| field.parse::<isize>())
                    .collect::<Result<_, _>>()?;
                Ok(fields)
            } else {
                // single int field
                let f = field.parse::<isize>()?;
                Ok(vec![f])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let _ = parse("1 0 * * *").unwrap();
    }
}
