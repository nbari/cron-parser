# cron parser

[![crates.io](https://img.shields.io/crates/v/cron-parser.svg)](https://crates.io/crates/cron-parser)
[![Test](https://github.com/nbari/cron-parser/actions/workflows/test.yml/badge.svg)](https://github.com/nbari/cron-parser/actions/workflows/test.yml)
[![docs](https://docs.rs/cron-parser/badge.svg)](https://docs.rs/cron-parser)

Five-field cron expression parsing and recurrence calculation with timezone support.

## Compile once with `Schedule`

Use `Schedule` when an expression is evaluated repeatedly. Parsing validates
the expression and compiles its five fields into an immutable representation;
queries then reuse that representation without allocating.

```rust
use cron_parser::Schedule;
use jiff::ZonedDateTime;

fn main() -> Result<(), cron_parser::ParseError> {
    let schedule: Schedule = "*/5 * * * *".parse()?;
    let now = ZonedDateTime::now();

    let next = schedule.next_after(&now);
    let previous = schedule.previous_before(&now);
    let upcoming = schedule.after(&now).take(10).collect::<Vec<_>>();

    if let Some(next) = next {
        assert!(next > now);
        assert!(schedule.includes(&next));
    }

    Ok(())
}
```

The query `ZonedDateTime` supplies the timezone, and results are returned in
that same timezone. Nonexistent spring-forward minutes
are skipped, while both real instants in a repeated fall-back minute are
returned in chronological order. `next_after` and `previous_before` are always
strictly exclusive.

The one-shot `parse()` function remains available for applications that only
need one result.

Example:

    use cron_parser::parse;
    use jiff::{Timestamp, ZonedDateTime, tz::TimeZone};

    fn main() -> Result<(), Box<dyn std::error::Error>> {
        // the current time, in the system time zone
        let now = ZonedDateTime::now();
        if let Ok(next) = parse("*/5 * * * *", &now) {
            println!("when: {next}");
        }

        // passing a custom timestamp, in UTC
        let after = Timestamp::from_second(1893456000)?.to_zoned_datetime(TimeZone::UTC);
        if let Ok(next) = parse("0 0 29 2 *", &after) {
            println!("next leap year: {next}");
            assert_eq!(next.timestamp().as_second(), 1961625600);
        }

        assert!(parse("2-3,9,*/15,1-8,11,9,4,5 * * * *", &now).is_ok());
        assert!(parse("* * * * */Fri", &now).is_err());

        // use a custom time zone
        assert!(parse("*/5 * * * *", &now.in_tz("Europe/Lisbon")?).is_ok());

        Ok(())
    }


Cron table:

```
# ┌─────────────────────  minute (0 - 59)
# │ ┌───────────────────  hour   (0 - 23)
# │ │ ┌─────────────────  dom    (1 - 31) day of month
# │ │ │ ┌───────────────  month  (1 - 12)
# │ │ │ │ ┌─────────────  dow    (0 - 6 or Sun - Sat)  day of week (Sunday to Saturday)
# │ │ │ │ │
# │ │ │ │ │
# │ │ │ │ │
# * * * * * <command to execute>
```

| Field        | Required | Allowed values | Allowed special characters |
| ------------ | -------- | -------------- | -------------------------- |
| Minutes      | Yes      | 0–59           | \* , - /                   |
| Hours        | Yes      | 0–23           | \* , - /                   |
| Day of month | Yes      | 1–31           | \* , - /                   |
| Month        | Yes      | 1–12           | \* , - /                   |
| Day of week  | Yes      | 0–6 or Sun-Sat | \* , - /                   |

Day-of-month and day-of-week use **AND** semantics when both are restricted.
Sunday is `0`; `7` is not accepted. Month names, aliases, wrapping ranges, and
Quartz-specific syntax are not supported.

> For the day of the week, when using a Weekday (Sun-Sat) the expression `*/Day` is not supported instead
> use the integer, reasons for this is that for example `*/Wed` = `*/3` translates
> to run every 3rd day of week, this means Sunday, Wednesday, Saturday.

* `*` any value
* `,` value list separator
* `-` range of values
* `/` step values


## start-end/step

Ranges with steps are supported, for example:

```
0 12-18/3 * * *  # every 3 hours starting from 12 to 18
```

Or every 6 hours starting from 1:

```
0 1/6 * * *
```


## Examples

The library includes several example programs demonstrating different use cases:

### Parse Example
Interactive cron expression parser that shows next execution times:

```bash
cargo run --example parse -- "*/5 * * * *"
cargo run --example parse -- "0 9 * * 1-5" --count 10
cargo run --example parse -- "0 12-18/3 * * *" --count 10
```

### Timezone Example
Demonstrates how cron expressions work across different timezones:

```bash
cargo run --example timezone
```

### Patterns Example
Showcases common cron expression patterns:

```bash
cargo run --example patterns
```

See the [examples/](examples/) directory for the full source code.

## Dependencies

Depends on crate [jiff](https://crates.io/crates/jiff) for dates and time zones.

Example of `Cargo.toml`:

    [dependencies]
    cron-parser = "0.13"
    jiff = "1"

Time zone names are resolved by `jiff`. By default it reads the system time
zone database on Unix and bundles one on Windows, so a name such as
`America/New_York` is only available when the host provides it (for example
the `tzdata` package in a container image). Enable jiff's `tzdb-bundle-always`
feature to embed the database in your binary instead.

Getting the next 10 leap year iterations:

    use cron_parser::Schedule;
    use jiff::{Timestamp, ZonedDateTime, tz::TimeZone};

    fn main() {
        let now = Timestamp::now().to_zoned_datetime(TimeZone::UTC);
        let schedule: Schedule = "0 0 29 2 *".parse().unwrap();
        let crons = schedule.after(&now).take(10).collect::<Vec<ZonedDateTime>>();
        for x in crons {
            println!("{} - {}", x, x.timestamp().as_second());
        }
    }

It will print something like:

    2028-02-29T00:00:00+00:00[UTC] - 1835395200
    2032-02-29T00:00:00+00:00[UTC] - 1961625600
    2036-02-29T00:00:00+00:00[UTC] - 2087856000
    2040-02-29T00:00:00+00:00[UTC] - 2214086400
    2044-02-29T00:00:00+00:00[UTC] - 2340316800
    2048-02-29T00:00:00+00:00[UTC] - 2466547200
    2052-02-29T00:00:00+00:00[UTC] - 2592777600
    2056-02-29T00:00:00+00:00[UTC] - 2719008000
    2060-02-29T00:00:00+00:00[UTC] - 2845238400
    2064-02-29T00:00:00+00:00[UTC] - 2971468800

## Migrating from chrono

Versions up to 0.12 used [chrono](https://crates.io/crates/chrono), which is no
longer maintained. The functions and their behavior are the same; only the
date types changed:

| 0.12 (chrono)                         | 0.13 (jiff)                                           |
| ------------------------------------- | ----------------------------------------------------- |
| `&DateTime<Tz>` in, `DateTime<Tz>` out | `&ZonedDateTime` in, `ZonedDateTime` out              |
| `Utc::now()`                          | `Timestamp::now().to_zoned_datetime(TimeZone::UTC)`   |
| `Local::now()`                        | `ZonedDateTime::now()`                                |
| `value.with_timezone(&chrono_tz::Europe::Lisbon)` | `value.in_tz("Europe/Lisbon")?`           |
| `next.timestamp()`                    | `next.timestamp().as_second()`                        |
| `ScheduleIterator<'_, Tz>`            | `ScheduleIterator<'_>`                                |

If the rest of your application still uses chrono, convert at the boundary
through Unix timestamps:

    let cursor = jiff::Timestamp::new(value.timestamp(), value.timestamp_subsec_nanos() as i32)?
        .in_tz("Europe/Lisbon")?;
