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
use chrono::Utc;
use cron_parser::Schedule;

fn main() -> Result<(), cron_parser::ParseError> {
    let schedule: Schedule = "*/5 * * * *".parse()?;
    let now = Utc::now();

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

The query timestamp supplies the timezone. Nonexistent spring-forward minutes
are skipped, while both real instants in a repeated fall-back minute are
returned in chronological order. `next_after` and `previous_before` are always
strictly exclusive.

The one-shot `parse()` function remains available for applications that only
need one result.

Example:

    use chrono::{TimeZone, Utc};
    use chrono_tz::Europe::Lisbon;
    use cron_parser::parse;

    fn main() {
       if let Ok(next) = parse("*/5 * * * *", &Utc::now()) {
            println!("when: {}", next);
       }

       // passing a custom timestamp
       if let Ok(next) = parse("0 0 29 2 *", &Utc.timestamp_opt(1893456000, 0).unwrap()) {
            println!("next leap year: {}", next);
            assert_eq!(next.timestamp(), 1961625600);
       }

       assert!(parse("2-3,9,*/15,1-8,11,9,4,5 * * * *", &Utc::now()).is_ok());
       assert!(parse("* * * * */Fri", &Utc::now()).is_err());

       // use custom timezone
       assert!(parse("*/5 * * * *", &Utc::now().with_timezone(&Lisbon)).is_ok());
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

Depends on crate [chrono](https://crates.io/crates/chrono).

Example of `Cargo.toml`:

    [dependencies]
    chrono = "^0.4"
    cron-parser = "0.12"


Getting the next 10 leap year iterations:

    use chrono::{DateTime, Utc};
    use cron_parser::Schedule;

    fn main() {
        let now = Utc::now();
        let schedule: Schedule = "0 0 29 2 *".parse().unwrap();
        let crons = schedule.after(&now).take(10).collect::<Vec<DateTime<Utc>>>();
        for x in crons {
            println!("{} - {}", x, x.timestamp());
        }
    }

It will print something like:

    2024-02-29 00:00:00 UTC - 1709164800
    2028-02-29 00:00:00 UTC - 1835395200
    2032-02-29 00:00:00 UTC - 1961625600
    2036-02-29 00:00:00 UTC - 2087856000
    2040-02-29 00:00:00 UTC - 2214086400
    2044-02-29 00:00:00 UTC - 2340316800
    2048-02-29 00:00:00 UTC - 2466547200
    2052-02-29 00:00:00 UTC - 2592777600
    2056-02-29 00:00:00 UTC - 2719008000
    2060-02-29 00:00:00 UTC - 2845238400
