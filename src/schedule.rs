use crate::{
    CronField, ParseError, ParseErrorKind, field::FieldSet, field::parse_expression_field,
};
use jiff::{
    SignedDuration, Timestamp, ZonedDateTime,
    civil::{Date, DateTime, Time},
    tz::{AmbiguousOffset, TimeZone},
};
use std::{fmt, hash::Hash, str::FromStr};

const GREGORIAN_CYCLE_YEARS: i16 = 400;

/// A parsed, immutable five-field cron schedule.
#[derive(Clone, Debug)]
pub struct Schedule {
    source: String,
    minutes: FieldSet,
    hours: FieldSet,
    days_of_month: FieldSet,
    months: FieldSet,
    days_of_week: FieldSet,
}

impl Schedule {
    /// Parse a five-field cron expression.
    ///
    /// # Errors
    ///
    /// Returns a structured [`ParseError`] when the expression is malformed or
    /// can never match a Gregorian calendar date.
    pub fn parse(expression: &str) -> Result<Self, ParseError> {
        expression.parse()
    }

    /// Return the original expression.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Return whether this exact, minute-aligned instant matches the schedule.
    #[must_use]
    pub fn includes(&self, value: &ZonedDateTime) -> bool {
        let local = value.datetime();
        local.second() == 0 && local.subsec_nanosecond() == 0 && self.matches_civil(local)
    }

    /// Find the first occurrence strictly after `after`.
    ///
    /// The result is in the time zone of `after`. Returns `None` when no
    /// occurrence is representable after `after`.
    #[must_use]
    pub fn next_after(&self, after: &ZonedDateTime) -> Option<ZonedDateTime> {
        let time_zone = after.time_zone();
        let boundary = after.timestamp();
        let local = after.datetime();
        if let Some(width) = first_fold_width(time_zone, boundary, local)
            && let Some(candidate) =
                self.best_in_window(time_zone, boundary, local, width, Direction::Forward)
        {
            return Some(candidate.to_zoned_datetime(time_zone.clone()));
        }

        // Occurrences are minute-aligned, so those after `after` are exactly
        // the ones whose local time is later than its minute. An earlier local
        // time can only be a later instant inside a fold, handled above.
        let start = floor_minute(local)?;
        let start_year = start.year();
        let start_month = unsigned(start.month());
        let start_day = unsigned(start.day());
        let start_hour = unsigned(start.hour());
        let start_minute = unsigned(start.minute());
        for offset in 0..=GREGORIAN_CYCLE_YEARS {
            let year = start_year.checked_add(offset)?;
            if year > Date::MAX.year() {
                return None;
            }
            let first_year = offset == 0;
            let months = if first_year {
                self.months.values_from(start_month)
            } else {
                self.months.values()
            };
            for month in months {
                let month_length = days_in_month(year, month);
                let first_month = first_year && month == start_month;
                let days = if first_month {
                    self.days_of_month.values_from(start_day)
                } else {
                    self.days_of_month.values()
                };
                for day in days {
                    if day > month_length {
                        break;
                    }
                    let Some(date) = civil_date(year, month, day) else {
                        continue;
                    };
                    if !self.matches_weekday(date) {
                        continue;
                    }
                    let first_day = first_month && day == start_day;
                    let hours = if first_day {
                        self.hours.values_from(start_hour)
                    } else {
                        self.hours.values()
                    };
                    for hour in hours {
                        let minutes = if first_day && hour == start_hour {
                            self.minutes.values_from(start_minute + 1)
                        } else {
                            self.minutes.values()
                        };
                        for minute in minutes {
                            let Some(civil) = civil_minute(date, hour, minute) else {
                                continue;
                            };
                            if let Some(candidate) = earliest_after(time_zone, boundary, civil) {
                                return Some(candidate.to_zoned_datetime(time_zone.clone()));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Find the last occurrence strictly before `before`.
    ///
    /// The result is in the time zone of `before`. Returns `None` when no
    /// occurrence is representable before `before`.
    #[must_use]
    pub fn previous_before(&self, before: &ZonedDateTime) -> Option<ZonedDateTime> {
        let time_zone = before.time_zone();
        let boundary = before.timestamp();
        let local = before.datetime();
        if let Some(width) = second_fold_width(time_zone, boundary, local)
            && let Some(candidate) =
                self.best_in_window(time_zone, boundary, local, width, Direction::Backward)
        {
            return Some(candidate.to_zoned_datetime(time_zone.clone()));
        }

        // Occurrences are minute-aligned, so those before `before` have a
        // local time up to its minute, or strictly before it when `before` is
        // itself minute-aligned. A later local time can only be an earlier
        // instant inside a fold, handled above.
        let start = floor_minute(local)?;
        let start_year = start.year();
        let start_month = unsigned(start.month());
        let start_day = unsigned(start.day());
        let start_hour = unsigned(start.hour());
        let minute_limit = unsigned(start.minute()) + u32::from(local > start);
        for offset in 0..=GREGORIAN_CYCLE_YEARS {
            let year = start_year.checked_sub(offset)?;
            if year < Date::MIN.year() {
                return None;
            }
            let last_year = offset == 0;
            let months = if last_year {
                self.months.values_before(start_month + 1)
            } else {
                self.months.values()
            };
            for month in months.rev() {
                let month_length = days_in_month(year, month);
                let last_month = last_year && month == start_month;
                let days = if last_month {
                    self.days_of_month.values_before(start_day + 1)
                } else {
                    self.days_of_month.values()
                };
                for day in days.rev() {
                    if day > month_length {
                        continue;
                    }
                    let Some(date) = civil_date(year, month, day) else {
                        continue;
                    };
                    if !self.matches_weekday(date) {
                        continue;
                    }
                    let last_day = last_month && day == start_day;
                    let hours = if last_day {
                        self.hours.values_before(start_hour + 1)
                    } else {
                        self.hours.values()
                    };
                    for hour in hours.rev() {
                        let minutes = if last_day && hour == start_hour {
                            self.minutes.values_before(minute_limit)
                        } else {
                            self.minutes.values()
                        };
                        for minute in minutes.rev() {
                            let Some(civil) = civil_minute(date, hour, minute) else {
                                continue;
                            };
                            if let Some(candidate) = latest_before(time_zone, boundary, civil) {
                                return Some(candidate.to_zoned_datetime(time_zone.clone()));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Iterate over occurrences strictly after `after`.
    #[must_use]
    pub fn after(&self, after: &ZonedDateTime) -> ScheduleIterator<'_> {
        ScheduleIterator::new(self, after.clone(), Direction::Forward)
    }

    /// Iterate over occurrences strictly before `before`.
    #[must_use]
    pub fn before(&self, before: &ZonedDateTime) -> ScheduleIterator<'_> {
        ScheduleIterator::new(self, before.clone(), Direction::Backward)
    }

    /// Create an owned iterator over occurrences strictly after `after`.
    #[must_use]
    pub fn after_owned(&self, after: ZonedDateTime) -> OwnedScheduleIterator {
        OwnedScheduleIterator::new(self.clone(), after, Direction::Forward)
    }

    /// Create an owned iterator over occurrences strictly before `before`.
    #[must_use]
    pub fn before_owned(&self, before: ZonedDateTime) -> OwnedScheduleIterator {
        OwnedScheduleIterator::new(self.clone(), before, Direction::Backward)
    }

    fn matches_civil(&self, value: DateTime) -> bool {
        self.minutes.contains(unsigned(value.minute()))
            && self.hours.contains(unsigned(value.hour()))
            && self.matches_date(value.date())
    }

    fn matches_date(&self, date: Date) -> bool {
        self.days_of_month.contains(unsigned(date.day()))
            && self.months.contains(unsigned(date.month()))
            && self.matches_weekday(date)
    }

    fn matches_weekday(&self, date: Date) -> bool {
        self.days_of_week
            .contains(unsigned(date.weekday().to_sunday_zero_offset()))
    }

    fn is_calendar_feasible(&self) -> bool {
        (2000..2400).any(|year| {
            self.months.values().any(|month| {
                let month_length = days_in_month(year, month);
                self.days_of_month
                    .values()
                    .take_while(|day| *day <= month_length)
                    .any(|day| {
                        civil_date(year, month, day).is_some_and(|date| self.matches_date(date))
                    })
            })
        })
    }

    fn best_in_window(
        &self,
        time_zone: &TimeZone,
        boundary: Timestamp,
        center: DateTime,
        width: SignedDuration,
        direction: Direction,
    ) -> Option<Timestamp> {
        let mut cursor = floor_minute(center.checked_sub(width).ok()?)?;
        let end = floor_minute(center.checked_add(width).ok()?)?;
        let mut best: Option<Timestamp> = None;

        while cursor <= end {
            if self.matches_civil(cursor) {
                let candidate = match direction {
                    Direction::Forward => earliest_after(time_zone, boundary, cursor),
                    Direction::Backward => latest_before(time_zone, boundary, cursor),
                };
                if let Some(candidate) = candidate {
                    let replace = match (best, direction) {
                        (None, _) => true,
                        (Some(current), Direction::Forward) => candidate < current,
                        (Some(current), Direction::Backward) => candidate > current,
                    };
                    if replace {
                        best = Some(candidate);
                    }
                }
            }
            cursor = next_minute(cursor)?;
        }
        best
    }
}

impl FromStr for Schedule {
    type Err = ParseError;

    fn from_str(expression: &str) -> Result<Self, Self::Err> {
        let tokens = expression_tokens(expression);
        if tokens.len() != 5 {
            return Err(ParseError::new(
                expression,
                None,
                expression.len()..expression.len(),
                "",
                ParseErrorKind::WrongFieldCount {
                    expected: 5,
                    actual: tokens.len(),
                },
            ));
        }

        let fields = [
            CronField::Minute,
            CronField::Hour,
            CronField::DayOfMonth,
            CronField::Month,
            CronField::DayOfWeek,
        ];
        let mut parsed = tokens
            .into_iter()
            .zip(fields)
            .map(|((token, start), field)| parse_expression_field(expression, token, start, field));
        let minutes = next_parsed(&mut parsed, expression)?;
        let hours = next_parsed(&mut parsed, expression)?;
        let days_of_month = next_parsed(&mut parsed, expression)?;
        let months = next_parsed(&mut parsed, expression)?;
        let days_of_week = next_parsed(&mut parsed, expression)?;

        let schedule = Self {
            source: expression.to_owned(),
            minutes,
            hours,
            days_of_month,
            months,
            days_of_week,
        };
        if !schedule.is_calendar_feasible() {
            return Err(ParseError::new(
                expression,
                None,
                0..expression.len(),
                expression,
                ParseErrorKind::ImpossibleSchedule,
            ));
        }
        Ok(schedule)
    }
}

fn next_parsed<I>(parsed: &mut I, expression: &str) -> Result<FieldSet, ParseError>
where
    I: Iterator<Item = Result<FieldSet, ParseError>>,
{
    parsed.next().unwrap_or_else(|| {
        Err(ParseError::new(
            expression,
            None,
            expression.len()..expression.len(),
            "",
            ParseErrorKind::WrongFieldCount {
                expected: 5,
                actual: 0,
            },
        ))
    })
}

impl fmt::Display for Schedule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.source)
    }
}

impl PartialEq for Schedule {
    fn eq(&self, other: &Self) -> bool {
        self.minutes == other.minutes
            && self.hours == other.hours
            && self.days_of_month == other.days_of_month
            && self.months == other.months
            && self.days_of_week == other.days_of_week
    }
}

impl Eq for Schedule {}

impl Hash for Schedule {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.minutes.hash(state);
        self.hours.hash(state);
        self.days_of_month.hash(state);
        self.months.hash(state);
        self.days_of_week.hash(state);
    }
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Forward,
    Backward,
}

/// An iterator borrowing a compiled schedule.
#[derive(Clone, Debug)]
pub struct ScheduleIterator<'a> {
    schedule: &'a Schedule,
    cursor: Option<ZonedDateTime>,
    direction: Direction,
}

impl<'a> ScheduleIterator<'a> {
    const fn new(schedule: &'a Schedule, cursor: ZonedDateTime, direction: Direction) -> Self {
        Self {
            schedule,
            cursor: Some(cursor),
            direction,
        }
    }
}

impl Iterator for ScheduleIterator<'_> {
    type Item = ZonedDateTime;

    fn next(&mut self) -> Option<Self::Item> {
        let cursor = self.cursor.take()?;
        let next = match self.direction {
            Direction::Forward => self.schedule.next_after(&cursor),
            Direction::Backward => self.schedule.previous_before(&cursor),
        }?;
        self.cursor = Some(next.clone());
        Some(next)
    }
}

/// An iterator owning its compiled schedule.
#[derive(Clone, Debug)]
pub struct OwnedScheduleIterator {
    schedule: Schedule,
    cursor: Option<ZonedDateTime>,
    direction: Direction,
}

impl OwnedScheduleIterator {
    const fn new(schedule: Schedule, cursor: ZonedDateTime, direction: Direction) -> Self {
        Self {
            schedule,
            cursor: Some(cursor),
            direction,
        }
    }
}

impl Iterator for OwnedScheduleIterator {
    type Item = ZonedDateTime;

    fn next(&mut self) -> Option<Self::Item> {
        let cursor = self.cursor.take()?;
        let next = match self.direction {
            Direction::Forward => self.schedule.next_after(&cursor),
            Direction::Backward => self.schedule.previous_before(&cursor),
        }?;
        self.cursor = Some(next.clone());
        Some(next)
    }
}

fn expression_tokens(expression: &str) -> Vec<(&str, usize)> {
    let mut tokens = Vec::new();
    let mut start = None;
    for (index, character) in expression.char_indices() {
        if character.is_whitespace() {
            if let Some(token_start) = start.take() {
                tokens.push((&expression[token_start..index], token_start));
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(token_start) = start {
        tokens.push((&expression[token_start..], token_start));
    }
    tokens
}

fn unsigned(value: i8) -> u32 {
    u32::from(value.unsigned_abs())
}

/// The number of days in a month of the proleptic Gregorian calendar.
const fn days_in_month(year: i16, month: u32) -> u32 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 31,
    }
}

fn civil_date(year: i16, month: u32, day: u32) -> Option<Date> {
    Date::new(year, i8::try_from(month).ok()?, i8::try_from(day).ok()?).ok()
}

fn civil_minute(date: Date, hour: u32, minute: u32) -> Option<DateTime> {
    let time = Time::new(i8::try_from(hour).ok()?, i8::try_from(minute).ok()?, 0, 0).ok()?;
    Some(DateTime::from_parts(date, time))
}

/// The minute after a minute-aligned civil datetime.
fn next_minute(value: DateTime) -> Option<DateTime> {
    let (date, hour, minute) = (value.date(), value.hour(), value.minute());
    let (date, hour, minute) = if minute < 59 {
        (date, hour, minute + 1)
    } else if hour < 23 {
        (date, hour + 1, 0)
    } else {
        (date.tomorrow().ok()?, 0, 0)
    };
    Some(DateTime::from_parts(
        date,
        Time::new(hour, minute, 0, 0).ok()?,
    ))
}

fn floor_minute(value: DateTime) -> Option<DateTime> {
    let time = Time::new(value.hour(), value.minute(), 0, 0).ok()?;
    Some(DateTime::from_parts(value.date(), time))
}

/// Both instants of a repeated (fall-back) local time, earliest first.
fn fold_instants(time_zone: &TimeZone, local: DateTime) -> Option<(Timestamp, Timestamp)> {
    match time_zone.to_ambiguous_timestamp(local).offset() {
        AmbiguousOffset::Fold { before, after } => Some((
            before.to_timestamp(local).ok()?,
            after.to_timestamp(local).ok()?,
        )),
        AmbiguousOffset::Unambiguous { .. } | AmbiguousOffset::Gap { .. } => None,
    }
}

fn first_fold_width(
    time_zone: &TimeZone,
    value: Timestamp,
    local: DateTime,
) -> Option<SignedDuration> {
    let (earlier, later) = fold_instants(time_zone, local)?;
    (value == earlier).then(|| later.duration_since(earlier))
}

fn second_fold_width(
    time_zone: &TimeZone,
    value: Timestamp,
    local: DateTime,
) -> Option<SignedDuration> {
    let (earlier, later) = fold_instants(time_zone, local)?;
    (value == later).then(|| later.duration_since(earlier))
}

/// The real instants a local time maps to: none inside a spring-forward gap,
/// two (earliest first) inside a fall-back fold, and one otherwise.
fn local_instants(time_zone: &TimeZone, local: DateTime) -> (Option<Timestamp>, Option<Timestamp>) {
    match time_zone.to_ambiguous_timestamp(local).offset() {
        AmbiguousOffset::Unambiguous { offset } => (offset.to_timestamp(local).ok(), None),
        AmbiguousOffset::Gap { .. } => (None, None),
        AmbiguousOffset::Fold { before, after } => (
            before.to_timestamp(local).ok(),
            after.to_timestamp(local).ok(),
        ),
    }
}

fn earliest_after(time_zone: &TimeZone, boundary: Timestamp, local: DateTime) -> Option<Timestamp> {
    let (earlier, later) = local_instants(time_zone, local);
    earlier
        .filter(|candidate| *candidate > boundary)
        .or_else(|| later.filter(|candidate| *candidate > boundary))
}

fn latest_before(time_zone: &TimeZone, boundary: Timestamp, local: DateTime) -> Option<Timestamp> {
    let (earlier, later) = local_instants(time_zone, local);
    later
        .filter(|candidate| *candidate < boundary)
        .or_else(|| earlier.filter(|candidate| *candidate < boundary))
}
