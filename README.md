# cron parser

[![crates.io](https://img.shields.io/crates/v/cron-parser.svg)](https://crates.io/crates/cron-parser)
[![Build Status](https://travis-ci.org/nbari/cron-parser.svg?branch=master)](https://travis-ci.org/nbari/cron-parser)

```
# ┌─────────────────────  minute (0 - 59)
# │ ┌───────────────────  hour   (0 - 23)
# │ │ ┌─────────────────  dom    (1 - 31) day of month
# │ │ │ ┌───────────────  month  (1 - 12)
# │ │ │ │ ┌─────────────  dow    (0 - 6)  day of week (Sunday to Saturday)
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
|Day of week | Yes | 0–6 | * , - / |

* `*` any value
* `,` value list separator
* `-` range of values
* `/` step values
