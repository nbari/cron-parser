//! Five-field cron expression parsing and recurrence calculation.
//!
//! [`Schedule`] parses an expression once and can then be reused for forward,
//! backward, membership, and iterator queries. [`parse`] remains available for
//! convenient one-shot next-occurrence queries.
//!
//! ```
//! use chrono::Utc;
//! use cron_parser::Schedule;
//!
//! let schedule: Schedule = "*/5 * * * *".parse()?;
//! let now = Utc::now();
//! let next = schedule.next_after(&now).expect("future occurrence");
//!
//! assert!(next > now);
//! assert!(schedule.includes(&next));
//! # Ok::<(), cron_parser::ParseError>(())
//! ```

mod error;
mod field;
mod schedule;

pub use error::{CronField, ParseError, ParseErrorKind};
pub use schedule::{OwnedScheduleIterator, Schedule, ScheduleIterator};

use chrono::{DateTime, TimeZone};
use std::collections::BTreeSet;

/// Parse an expression and return its first occurrence strictly after `after`.
///
/// Prefer compiling a [`Schedule`] once when evaluating an expression more
/// than once.
///
/// # Errors
///
/// Returns a structured [`ParseError`] when the expression is invalid or no
/// occurrence is representable after `after`.
pub fn parse<Tz: TimeZone>(
    expression: &str,
    after: &DateTime<Tz>,
) -> Result<DateTime<Tz>, ParseError> {
    let schedule = Schedule::parse(expression)?;
    schedule
        .next_after(after)
        .ok_or_else(|| ParseError::no_occurrence(expression))
}

/// Parse one field into its allowed values.
///
/// Weekday names are accepted only for the standard `0..=6` weekday domain.
/// Empty list elements are rejected.
///
/// # Errors
///
/// Returns a structured [`ParseError`] when the field is malformed or contains
/// values outside `min..=max`.
pub fn parse_field(field: &str, min: u32, max: u32) -> Result<BTreeSet<u32>, ParseError> {
    field::parse_public_field(field, min, max)
}
