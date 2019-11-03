# cron parser

[![crates.io](https://img.shields.io/crates/v/cron-parser.svg)](https://crates.io/crates/cron-parser)
[![Build Status](https://travis-ci.org/nbari/cron-parser.svg?branch=master)](https://travis-ci.org/nbari/cron-parser)

Returns the next timestamp

```
# ┌───────────── minute (0 - 59)
# │ ┌───────────── hour (0 - 23)
# │ │ ┌───────────── day of the month (1 - 31)
# │ │ │ ┌───────────── month (1 - 12)
# │ │ │ │ ┌───────────── day of the week (0 - 6)
# │ │ │ │ │
# │ │ │ │ │
# │ │ │ │ │
# * * * * * command to execute
```

|Field|Required|Allowed values|Allowed special characters|
|-----|--------|--------------|--------------------------|
|Minutes| Yes | 0–59 | * / , - |
|Hours  | Yes | 0–23 | * / , - |
|Day of month| Yes | 1–31 | * / , - |
|Month | Yes | 1–12 | * / , -  |
|Day of week | Yes | 0–6 | * / , - |


For fixed intervals

    @every <duration>

For example, `@every 2h15m30s` would be every 2 hours, 15 minutes and 30
seconds.
