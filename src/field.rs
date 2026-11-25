use crate::{CronField, ParseError, ParseErrorKind};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct FieldSet {
    min: u32,
    max: u32,
    bits: u64,
}

impl FieldSet {
    pub(crate) const fn contains(self, value: u32) -> bool {
        value >= self.min && value <= self.max && self.bits & (1_u64 << (value - self.min)) != 0
    }

    /// All values, in ascending order.
    pub(crate) const fn values(self) -> Values {
        Values {
            min: self.min,
            bits: self.bits,
        }
    }

    /// The values greater than or equal to `lower`, in ascending order.
    pub(crate) const fn values_from(self, lower: u32) -> Values {
        let skipped = lower.saturating_sub(self.min);
        let bits = if skipped < u64::BITS {
            self.bits & (u64::MAX << skipped)
        } else {
            0
        };
        Values {
            min: self.min,
            bits,
        }
    }

    /// The values strictly less than `upper`, in ascending order.
    pub(crate) const fn values_before(self, upper: u32) -> Values {
        let kept = upper.saturating_sub(self.min);
        let bits = if kept < u64::BITS {
            self.bits & ((1_u64 << kept) - 1)
        } else {
            self.bits
        };
        Values {
            min: self.min,
            bits,
        }
    }

    pub(crate) fn to_btree_set(self) -> BTreeSet<u32> {
        self.values().collect()
    }
}

/// The values of a [`FieldSet`], visiting only the bits that are set.
#[derive(Clone, Debug)]
pub(crate) struct Values {
    min: u32,
    bits: u64,
}

impl Iterator for Values {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }
        let index = self.bits.trailing_zeros();
        self.bits &= self.bits - 1;
        Some(self.min + index)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.bits.count_ones() as usize;
        (remaining, Some(remaining))
    }
}

impl DoubleEndedIterator for Values {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }
        let index = u64::BITS - 1 - self.bits.leading_zeros();
        self.bits &= !(1_u64 << index);
        Some(self.min + index)
    }
}

impl ExactSizeIterator for Values {}

pub(crate) fn parse_expression_field(
    expression: &str,
    token: &str,
    span_start: usize,
    field: CronField,
) -> Result<FieldSet, ParseError> {
    let (min, max) = field_bounds(field);
    parse_set(expression, token, span_start, min, max, Some(field))
}

pub(crate) fn parse_public_field(
    input: &str,
    min: u32,
    max: u32,
) -> Result<BTreeSet<u32>, ParseError> {
    let field = match (min, max) {
        (0, 59) => Some(CronField::Minute),
        (0, 23) => Some(CronField::Hour),
        (1, 31) => Some(CronField::DayOfMonth),
        (1, 12) => Some(CronField::Month),
        (0, 6) => Some(CronField::DayOfWeek),
        _ => None,
    };
    if max.saturating_sub(min) >= 64 {
        parse_large_public_set(input, min, max, field)
    } else {
        parse_set(input, input, 0, min, max, field).map(FieldSet::to_btree_set)
    }
}

const fn field_bounds(field: CronField) -> (u32, u32) {
    match field {
        CronField::Minute => (0, 59),
        CronField::Hour => (0, 23),
        CronField::DayOfMonth => (1, 31),
        CronField::Month => (1, 12),
        CronField::DayOfWeek => (0, 6),
    }
}

fn parse_set(
    expression: &str,
    input: &str,
    span_start: usize,
    min: u32,
    max: u32,
    field: Option<CronField>,
) -> Result<FieldSet, ParseError> {
    let mut bits = 0_u64;
    let mut offset = 0;

    for item in input.split(',') {
        let item_start = span_start + offset;
        let item_end = item_start + item.len();
        if item.is_empty() {
            return Err(ParseError::new(
                expression,
                field,
                item_start..item_end,
                item,
                ParseErrorKind::EmptyListElement,
            ));
        }
        parse_item(expression, item, item_start, min, max, field, &mut bits)?;
        offset += item.len() + 1;
    }

    Ok(FieldSet { min, max, bits })
}

#[allow(clippy::too_many_arguments)]
fn parse_item(
    expression: &str,
    item: &str,
    item_start: usize,
    min: u32,
    max: u32,
    field: Option<CronField>,
    bits: &mut u64,
) -> Result<(), ParseError> {
    let mut slash = item.split('/');
    let base = slash.next().unwrap_or_default();
    let step_text = slash.next();
    if slash.next().is_some() || base.is_empty() {
        return Err(error(
            expression,
            field,
            item_start,
            item,
            ParseErrorKind::InvalidRange,
        ));
    }

    let step = if let Some(step_text) = step_text {
        parse_number(expression, step_text, item_start + base.len() + 1, field)?
    } else {
        1
    };
    if step == 0 {
        return Err(error(
            expression,
            field,
            item_start,
            item,
            ParseErrorKind::ZeroStep,
        ));
    }
    if step_text.is_some() && step > max {
        return Err(error(
            expression,
            field,
            item_start,
            item,
            ParseErrorKind::StepOutOfRange { max, actual: step },
        ));
    }

    let (start, end) = if base == "*" {
        (min, max)
    } else if base.contains('-') {
        let mut range = base.split('-');
        let start_text = range.next().unwrap_or_default();
        let end_text = range.next().unwrap_or_default();
        if start_text.is_empty() || end_text.is_empty() || range.next().is_some() {
            return Err(error(
                expression,
                field,
                item_start,
                item,
                ParseErrorKind::InvalidRange,
            ));
        }
        let start = parse_value(expression, start_text, item_start, min, max, field)?;
        let end = parse_value(
            expression,
            end_text,
            item_start + start_text.len() + 1,
            min,
            max,
            field,
        )?;
        if start > end {
            return Err(error(
                expression,
                field,
                item_start,
                item,
                ParseErrorKind::ReversedRange,
            ));
        }
        (start, end)
    } else {
        let start = parse_value(expression, base, item_start, min, max, field)?;
        (start, if step_text.is_some() { max } else { start })
    };

    for value in (start..=end).step_by(step as usize) {
        *bits |= 1_u64 << (value - min);
    }
    Ok(())
}

fn parse_large_public_set(
    input: &str,
    min: u32,
    max: u32,
    field: Option<CronField>,
) -> Result<BTreeSet<u32>, ParseError> {
    let mut values = BTreeSet::new();
    let mut offset = 0;
    for item in input.split(',') {
        if item.is_empty() {
            return Err(error(
                input,
                field,
                offset,
                item,
                ParseErrorKind::EmptyListElement,
            ));
        }
        let mut slash = item.split('/');
        let base = slash.next().unwrap_or_default();
        let step_text = slash.next();
        if slash.next().is_some() || base.is_empty() {
            return Err(error(
                input,
                field,
                offset,
                item,
                ParseErrorKind::InvalidRange,
            ));
        }
        let step = step_text.map_or(Ok(1), |text| {
            parse_number(input, text, offset + base.len() + 1, field)
        })?;
        if step == 0 {
            return Err(error(input, field, offset, item, ParseErrorKind::ZeroStep));
        }
        if step_text.is_some() && step > max {
            return Err(error(
                input,
                field,
                offset,
                item,
                ParseErrorKind::StepOutOfRange { max, actual: step },
            ));
        }
        let (start, end) = if base == "*" {
            (min, max)
        } else if let Some((start, end)) = base.split_once('-') {
            if start.is_empty() || end.is_empty() || end.contains('-') {
                return Err(error(
                    input,
                    field,
                    offset,
                    item,
                    ParseErrorKind::InvalidRange,
                ));
            }
            let start_value = parse_value(input, start, offset, min, max, field)?;
            let end_value = parse_value(input, end, offset + start.len() + 1, min, max, field)?;
            if start_value > end_value {
                return Err(error(
                    input,
                    field,
                    offset,
                    item,
                    ParseErrorKind::ReversedRange,
                ));
            }
            (start_value, end_value)
        } else {
            let start = parse_value(input, base, offset, min, max, field)?;
            (start, if step_text.is_some() { max } else { start })
        };
        values.extend((start..=end).step_by(step as usize));
        offset += item.len() + 1;
    }
    Ok(values)
}

fn parse_value(
    expression: &str,
    token: &str,
    token_start: usize,
    min: u32,
    max: u32,
    field: Option<CronField>,
) -> Result<u32, ParseError> {
    if token.bytes().all(|byte| byte.is_ascii_alphabetic()) {
        if field != Some(CronField::DayOfWeek) {
            return Err(error(
                expression,
                field,
                token_start,
                token,
                ParseErrorKind::NameNotAllowed,
            ));
        }
        return weekday(token).ok_or_else(|| {
            error(
                expression,
                field,
                token_start,
                token,
                ParseErrorKind::InvalidName,
            )
        });
    }

    let value = parse_number(expression, token, token_start, field)?;
    if value < min || value > max {
        return Err(error(
            expression,
            field,
            token_start,
            token,
            ParseErrorKind::OutOfRange {
                min,
                max,
                actual: value,
            },
        ));
    }
    Ok(value)
}

fn parse_number(
    expression: &str,
    token: &str,
    token_start: usize,
    field: Option<CronField>,
) -> Result<u32, ParseError> {
    if token.is_empty() || !token.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(error(
            expression,
            field,
            token_start,
            token,
            ParseErrorKind::InvalidInteger,
        ));
    }

    token.parse::<u32>().map_err(|_| {
        error(
            expression,
            field,
            token_start,
            token,
            ParseErrorKind::InvalidInteger,
        )
    })
}

fn weekday(token: &str) -> Option<u32> {
    match token.to_ascii_uppercase().as_str() {
        "SUN" => Some(0),
        "MON" => Some(1),
        "TUE" => Some(2),
        "WED" => Some(3),
        "THU" => Some(4),
        "FRI" => Some(5),
        "SAT" => Some(6),
        _ => None,
    }
}

fn error(
    expression: &str,
    field: Option<CronField>,
    start: usize,
    token: &str,
    kind: ParseErrorKind,
) -> ParseError {
    ParseError::new(expression, field, start..start + token.len(), token, kind)
}
