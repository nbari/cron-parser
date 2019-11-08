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

|Field|Required|Allowed values|Allowed special characters|
|-----|--------|--------------|--------------------------|
|Minutes| Yes | 0–59 | * , - / |
|Hours  | Yes | 0–23 | * , - / |
|Day of month| Yes | 1–31 | * , - / |
|Month | Yes | 1–12 | * , - / |
|Day of week | Yes | 0–6 or Sun-Sat | * , - / |

> For the day of the week, when using a Weekday (Sun-Sat) the expression `*/Day` is not supported instead
use the integer, reasons for this is that for example `*/Wed` = `*/3` translates
to run every 3rd day of week, this means Sunday, Wednesday, Saturday.

* `*` any value
* `,` value list separator
* `-` range of values
* `/` step values
