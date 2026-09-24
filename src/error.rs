use std::{error::Error, fmt, ops::Range};

/// A field in a five-field cron expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CronField {
    Minute,
    Hour,
    DayOfMonth,
    Month,
    DayOfWeek,
}

impl fmt::Display for CronField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Minute => "minute",
            Self::Hour => "hour",
            Self::DayOfMonth => "day of month",
            Self::Month => "month",
            Self::DayOfWeek => "day of week",
        })
    }
}

/// The reason a cron expression was rejected.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    WrongFieldCount { expected: usize, actual: usize },
    InvalidInteger,
    OutOfRange { min: u32, max: u32, actual: u32 },
    ZeroStep,
    StepOutOfRange { max: u32, actual: u32 },
    InvalidRange,
    ReversedRange,
    EmptyListElement,
    NameNotAllowed,
    InvalidName,
    ImpossibleSchedule,
    NoOccurrence,
}

/// A structured cron validation error.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    expression: String,
    field: Option<CronField>,
    span: Range<usize>,
    token: String,
    kind: ParseErrorKind,
}

impl ParseError {
    pub(crate) fn new(
        expression: &str,
        field: Option<CronField>,
        span: Range<usize>,
        token: &str,
        kind: ParseErrorKind,
    ) -> Self {
        Self {
            expression: expression.to_owned(),
            field,
            span,
            token: token.to_owned(),
            kind,
        }
    }

    pub(crate) fn no_occurrence(expression: &str) -> Self {
        Self::new(
            expression,
            None,
            expression.len()..expression.len(),
            "",
            ParseErrorKind::NoOccurrence,
        )
    }

    /// The original expression.
    #[must_use]
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// The field containing the error, when known.
    #[must_use]
    pub const fn field(&self) -> Option<CronField> {
        self.field
    }

    /// The byte range of the offending token.
    #[must_use]
    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    /// The offending token.
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    /// The machine-readable reason.
    #[must_use]
    pub const fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(field) = self.field {
            write!(formatter, "invalid {field} field")?;
        } else {
            formatter.write_str("invalid cron expression")?;
        }

        match &self.kind {
            ParseErrorKind::WrongFieldCount { expected, actual } => {
                write!(formatter, ": expected {expected} fields, found {actual}")
            }
            ParseErrorKind::InvalidInteger => {
                write!(formatter, ": `{}` is not an integer", self.token)
            }
            ParseErrorKind::OutOfRange { min, max, actual } => {
                write!(formatter, ": {actual} is outside {min}..={max}")
            }
            ParseErrorKind::ZeroStep => formatter.write_str(": step cannot be zero"),
            ParseErrorKind::StepOutOfRange { max, actual } => {
                write!(formatter, ": step {actual} exceeds {max}")
            }
            ParseErrorKind::InvalidRange => write!(formatter, ": invalid range `{}`", self.token),
            ParseErrorKind::ReversedRange => write!(formatter, ": reversed range `{}`", self.token),
            ParseErrorKind::EmptyListElement => formatter.write_str(": empty list element"),
            ParseErrorKind::NameNotAllowed => {
                write!(formatter, ": name `{}` is not allowed here", self.token)
            }
            ParseErrorKind::InvalidName => write!(formatter, ": unknown name `{}`", self.token),
            ParseErrorKind::ImpossibleSchedule => formatter.write_str(": schedule can never occur"),
            ParseErrorKind::NoOccurrence => formatter.write_str(": no representable occurrence"),
        }
    }
}

impl Error for ParseError {}
