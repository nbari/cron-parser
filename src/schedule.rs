use crate::{
    CronField, ParseError, ParseErrorKind, field::FieldSet, field::parse_expression_field,
};
use chrono::{
    DateTime, Datelike, Duration, LocalResult, NaiveDate, NaiveDateTime, Offset, TimeZone, Timelike,
};
use std::{fmt, hash::Hash, str::FromStr};

const GREGORIAN_CYCLE_YEARS: i32 = 400;

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
    pub fn includes<Tz: TimeZone>(&self, value: &DateTime<Tz>) -> bool {
        match local_time(value) {
            LocalTime::InRange(local) => {
                local.second() == 0 && local.nanosecond() == 0 && self.matches_naive(local)
            }
            LocalTime::BeforeMinimum | LocalTime::AfterMaximum => false,
        }
    }

    /// Find the first occurrence strictly after `after`.
    #[must_use]
    pub fn next_after<Tz: TimeZone>(&self, after: &DateTime<Tz>) -> Option<DateTime<Tz>> {
        let local = match local_time(after) {
            LocalTime::InRange(local) => {
                if let Some(delta) = first_fold_width(after, local)
                    && let Some(candidate) =
                        self.best_in_window(after, local, delta, Direction::Forward)
                {
                    return Some(candidate);
                }
                local
            }
            LocalTime::BeforeMinimum => NaiveDateTime::MIN,
            LocalTime::AfterMaximum => return None,
        };

        let start = floor_minute(local)?;
        let start_year = start.year();
        for offset in 0..=GREGORIAN_CYCLE_YEARS {
            let year = start_year.checked_add(offset)?;
            for month in self.months.values() {
                if year == start_year && month < start.month() {
                    continue;
                }
                for day in self.days_of_month.values() {
                    let Some(date) = NaiveDate::from_ymd_opt(year, month, day) else {
                        continue;
                    };
                    if date < start.date() || !self.matches_date(date) {
                        continue;
                    }
                    for hour in self.hours.values() {
                        for minute in self.minutes.values() {
                            let Some(naive) = date.and_hms_opt(hour, minute, 0) else {
                                continue;
                            };
                            if naive < start {
                                continue;
                            }
                            if let Some(candidate) =
                                earliest_after(after, after.timezone().from_local_datetime(&naive))
                            {
                                return Some(candidate);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Find the last occurrence strictly before `before`.
    #[must_use]
    pub fn previous_before<Tz: TimeZone>(&self, before: &DateTime<Tz>) -> Option<DateTime<Tz>> {
        let local = match local_time(before) {
            LocalTime::InRange(local) => {
                if let Some(delta) = second_fold_width(before, local)
                    && let Some(candidate) =
                        self.best_in_window(before, local, delta, Direction::Backward)
                {
                    return Some(candidate);
                }
                local
            }
            LocalTime::BeforeMinimum => return None,
            LocalTime::AfterMaximum => NaiveDateTime::MAX,
        };

        let start = floor_minute(local)?;
        let start_year = start.year();
        for offset in 0..=GREGORIAN_CYCLE_YEARS {
            let year = start_year.checked_sub(offset)?;
            for month in self.months.values().rev() {
                if year == start_year && month > start.month() {
                    continue;
                }
                for day in self.days_of_month.values().rev() {
                    let Some(date) = NaiveDate::from_ymd_opt(year, month, day) else {
                        continue;
                    };
                    if date > start.date() || !self.matches_date(date) {
                        continue;
                    }
                    for hour in self.hours.values().rev() {
                        for minute in self.minutes.values().rev() {
                            let Some(naive) = date.and_hms_opt(hour, minute, 0) else {
                                continue;
                            };
                            if naive > start {
                                continue;
                            }
                            if let Some(candidate) =
                                latest_before(before, before.timezone().from_local_datetime(&naive))
                            {
                                return Some(candidate);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Iterate over occurrences strictly after `after`.
    pub fn after<'a, Tz: TimeZone>(&'a self, after: &DateTime<Tz>) -> ScheduleIterator<'a, Tz> {
        ScheduleIterator::new(self, after.clone(), Direction::Forward)
    }

    /// Iterate over occurrences strictly before `before`.
    pub fn before<'a, Tz: TimeZone>(&'a self, before: &DateTime<Tz>) -> ScheduleIterator<'a, Tz> {
        ScheduleIterator::new(self, before.clone(), Direction::Backward)
    }

    /// Create an owned iterator over occurrences strictly after `after`.
    pub fn after_owned<Tz: TimeZone>(&self, after: DateTime<Tz>) -> OwnedScheduleIterator<Tz> {
        OwnedScheduleIterator::new(self.clone(), after, Direction::Forward)
    }

    /// Create an owned iterator over occurrences strictly before `before`.
    pub fn before_owned<Tz: TimeZone>(&self, before: DateTime<Tz>) -> OwnedScheduleIterator<Tz> {
        OwnedScheduleIterator::new(self.clone(), before, Direction::Backward)
    }

    fn matches_naive(&self, value: NaiveDateTime) -> bool {
        self.minutes.contains(value.minute())
            && self.hours.contains(value.hour())
            && self.matches_date(value.date())
    }

    fn matches_date(&self, date: NaiveDate) -> bool {
        self.days_of_month.contains(date.day())
            && self.months.contains(date.month())
            && self
                .days_of_week
                .contains(date.weekday().num_days_from_sunday())
    }

    fn is_calendar_feasible(&self) -> bool {
        (2000..2400).any(|year| {
            self.months.values().any(|month| {
                self.days_of_month.values().any(|day| {
                    NaiveDate::from_ymd_opt(year, month, day)
                        .is_some_and(|date| self.matches_date(date))
                })
            })
        })
    }

    fn best_in_window<Tz: TimeZone>(
        &self,
        boundary: &DateTime<Tz>,
        center: NaiveDateTime,
        delta: Duration,
        direction: Direction,
    ) -> Option<DateTime<Tz>> {
        let mut cursor = floor_minute(center.checked_sub_signed(delta)?)?;
        let end = floor_minute(center.checked_add_signed(delta)?)?;
        let mut best: Option<DateTime<Tz>> = None;

        while cursor <= end {
            if self.matches_naive(cursor) {
                let local = boundary.timezone().from_local_datetime(&cursor);
                let candidate = match direction {
                    Direction::Forward => earliest_after(boundary, local),
                    Direction::Backward => latest_before(boundary, local),
                };
                if let Some(candidate) = candidate {
                    let replace = match (&best, direction) {
                        (None, _) => true,
                        (Some(current), Direction::Forward) => candidate < *current,
                        (Some(current), Direction::Backward) => candidate > *current,
                    };
                    if replace {
                        best = Some(candidate);
                    }
                }
            }
            cursor = cursor.checked_add_signed(Duration::minutes(1))?;
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

#[derive(Clone, Copy)]
enum Direction {
    Forward,
    Backward,
}

/// An iterator borrowing a compiled schedule.
pub struct ScheduleIterator<'a, Tz: TimeZone> {
    schedule: &'a Schedule,
    cursor: Option<DateTime<Tz>>,
    direction: Direction,
}

impl<'a, Tz: TimeZone> ScheduleIterator<'a, Tz> {
    fn new(schedule: &'a Schedule, cursor: DateTime<Tz>, direction: Direction) -> Self {
        Self {
            schedule,
            cursor: Some(cursor),
            direction,
        }
    }
}

impl<Tz: TimeZone> Iterator for ScheduleIterator<'_, Tz> {
    type Item = DateTime<Tz>;

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
pub struct OwnedScheduleIterator<Tz: TimeZone> {
    schedule: Schedule,
    cursor: Option<DateTime<Tz>>,
    direction: Direction,
}

impl<Tz: TimeZone> OwnedScheduleIterator<Tz> {
    fn new(schedule: Schedule, cursor: DateTime<Tz>, direction: Direction) -> Self {
        Self {
            schedule,
            cursor: Some(cursor),
            direction,
        }
    }
}

impl<Tz: TimeZone> Iterator for OwnedScheduleIterator<Tz> {
    type Item = DateTime<Tz>;

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

fn floor_minute(value: NaiveDateTime) -> Option<NaiveDateTime> {
    value.with_second(0)?.with_nanosecond(0)
}

enum LocalTime {
    BeforeMinimum,
    InRange(NaiveDateTime),
    AfterMaximum,
}

fn local_time<Tz: TimeZone>(value: &DateTime<Tz>) -> LocalTime {
    let offset = i64::from(value.offset().fix().local_minus_utc());
    match value
        .naive_utc()
        .checked_add_signed(Duration::seconds(offset))
    {
        Some(local) => LocalTime::InRange(local),
        None if offset.is_negative() => LocalTime::BeforeMinimum,
        None => LocalTime::AfterMaximum,
    }
}

fn first_fold_width<Tz: TimeZone>(value: &DateTime<Tz>, local: NaiveDateTime) -> Option<Duration> {
    match value.timezone().from_local_datetime(&local) {
        LocalResult::Ambiguous(first, second) => {
            let (earlier, later) = chronological(first, second);
            (*value == earlier).then(|| later.signed_duration_since(earlier))
        }
        _ => None,
    }
}

fn second_fold_width<Tz: TimeZone>(value: &DateTime<Tz>, local: NaiveDateTime) -> Option<Duration> {
    match value.timezone().from_local_datetime(&local) {
        LocalResult::Ambiguous(first, second) => {
            let (earlier, later) = chronological(first, second);
            (*value == later).then(|| later.signed_duration_since(earlier))
        }
        _ => None,
    }
}

fn earliest_after<Tz: TimeZone>(
    boundary: &DateTime<Tz>,
    local: LocalResult<DateTime<Tz>>,
) -> Option<DateTime<Tz>> {
    match local {
        LocalResult::None => None,
        LocalResult::Single(candidate) => (candidate > *boundary).then_some(candidate),
        LocalResult::Ambiguous(first, second) => {
            let (earlier, later) = chronological(first, second);
            if earlier > *boundary {
                Some(earlier)
            } else if later > *boundary {
                Some(later)
            } else {
                None
            }
        }
    }
}

fn latest_before<Tz: TimeZone>(
    boundary: &DateTime<Tz>,
    local: LocalResult<DateTime<Tz>>,
) -> Option<DateTime<Tz>> {
    match local {
        LocalResult::None => None,
        LocalResult::Single(candidate) => (candidate < *boundary).then_some(candidate),
        LocalResult::Ambiguous(first, second) => {
            let (earlier, later) = chronological(first, second);
            if later < *boundary {
                Some(later)
            } else if earlier < *boundary {
                Some(earlier)
            } else {
                None
            }
        }
    }
}

fn chronological<Tz: TimeZone>(
    first: DateTime<Tz>,
    second: DateTime<Tz>,
) -> (DateTime<Tz>, DateTime<Tz>) {
    if first <= second {
        (first, second)
    } else {
        (second, first)
    }
}
