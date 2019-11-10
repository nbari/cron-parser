# cron parser

[![crates.io](https://img.shields.io/crates/v/cron-parser.svg)](https://crates.io/crates/cron-parser)
[![Build Status](https://travis-ci.org/nbari/cron-parser.svg?branch=master)](https://travis-ci.org/nbari/cron-parser)

Library for parsing cron syntax, returning the next available date.

Example:

    use chrono::{TimeZone, Utc};
    use cron_parser::parse;

    fn main() {
       if let Ok(next) = parse("*/5 * * * *", Utc::now()) {
            println!("when: {}", next);
       }

       // passing a custom timestamp
       if let Ok(next) = parse("0 0 29 2 *", Utc.timestamp(1893456000, 0)) {
            println!("next leap year: {}", next);
            assert_eq!(next.timestamp(), 1961625600);
       }

       assert!(parse("2-3,9,*/15,1-8,11,9,4,5 * * * *", Utc::now()).is_ok());
       assert!(parse("* * * * */Fri", Utc::now()).is_err());
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
# * * * * * command to execute
```

| Field        | Required | Allowed values | Allowed special characters |
| ------------ | -------- | -------------- | -------------------------- |
| Minutes      | Yes      | 0–59           | \* , - /                   |
| Hours        | Yes      | 0–23           | \* , - /                   |
| Day of month | Yes      | 1–31           | \* , - /                   |
| Month        | Yes      | 1–12           | \* , - /                   |
| Day of week  | Yes      | 0–6 or Sun-Sat | \* , - /                   |

> For the day of the week, when using a Weekday (Sun-Sat) the expression `*/Day` is not supported instead
> use the integer, reasons for this is that for example `*/Wed` = `*/3` translates
> to run every 3rd day of week, this means Sunday, Wednesday, Saturday.

* `*` any value
* `,` value list separator
* `-` range of values
* `/` step values


Depends on crate [chrono](https://crates.io/crates/chrono).

Example of `Cargo.toml`:

    [dependencies]
    chrono = "^0.4"
    cron-parser = "^0.3"


Getting the next 10 leap year iterations:

    use chrono::{DateTime, Utc};
    use cron_parser::parse;

    fn main() {
        let now = Utc::now();
        let mut crons = Vec::<DateTime<Utc>>::new();
        let mut next = parse("0 0 29 2 *", now).unwrap();
        for _ in 0..10 {
            next = parse("0 0 29 2 *", next).unwrap();
            crons.push(next);
        }
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
