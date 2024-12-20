# cron parser

[![crates.io](https://img.shields.io/crates/v/cron-parser.svg)](https://crates.io/crates/cron-parser)
[![Test](https://github.com/nbari/cron-parser/actions/workflows/test.yml/badge.svg)](https://github.com/nbari/cron-parser/actions/workflows/test.yml)
[![docs](https://docs.rs/cron-parser/badge.svg)](https://docs.rs/cron-parser)

Library for parsing cron expressions with timezone support.

Example:

    use chrono::{TimeZone, Utc};
    use chrono_tz::Europe::Lisbon;
    use cron_parser::parse;

    fn main() {
       if let Ok(next) = parse("*/5 * * * *", &Utc::now()) {
            println!("when: {}", next);
       }

       // passing a custom timestamp
       if let Ok(next) = parse("0 0 29 2 *", &Utc.timestamp(1893456000, 0)) {
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


Depends on crate [chrono](https://crates.io/crates/chrono).

Example of `Cargo.toml`:

    [dependencies]
    chrono = "^0.4"
    cron-parser = "*"


Getting the next 10 leap year iterations:

    use chrono::{DateTime, Utc};
    use cron_parser::parse;

    fn main() {
        let now = Utc::now();
        let mut crons = Vec::<DateTime<Utc>>::new();
        let mut next = parse("0 0 29 2 *", &now).unwrap();
        for _ in 0..10 {
            next = parse("0 0 29 2 *", &next).unwrap();
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
