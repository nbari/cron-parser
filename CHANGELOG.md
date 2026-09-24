# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.12.0] - 2026-09-24

### Added
- Reusable, immutable `Schedule` that parses an expression once and answers `next_after`, `previous_before`, and `includes` queries; also implements `FromStr` and `Display`
- Borrowed (`after`, `before`) and owned (`after_owned`, `before_owned`) recurrence iterators
- Semantic `Eq`/`Hash` for `Schedule`: expressions matching the same times compare equal (e.g. `*/15` and `0,15,30,45`)
- Structured `ParseError` exposing the field (`CronField`), offending token, byte span, and a machine-readable `ParseErrorKind`
- `examples/schedule.rs` touring the `Schedule` API (`just run-schedule-example`)
- Property tests, allocation tests, parser/evaluator fuzz targets, and compiled-query benchmarks

### Changed
- Compile cron fields into fixed bitsets; compiled queries do not allocate
- Search the complete 400-year Gregorian cycle instead of stopping after four years, so rare schedules such as `0 0 29 2 0` (Feb 29 on a Sunday) now resolve instead of returning an error
- Reject schedules that can never match a calendar date (e.g. `0 0 31 2 *`) at parse time with `ParseErrorKind::ImpossibleSchedule`
- CI: least-privilege workflow permissions, `persist-credentials: false`, current action versions, and a job running release-mode tests and checking the fuzz targets

### Fixed
- Day-of-week schedules no longer skip earlier matching times on the matching day; previously `* * * * fri` evaluated on Tuesday at 10:07 returned Friday 10:08 instead of Friday 00:00. Jobs restricted by day of week may now fire at different (correct) times after upgrading
- Return both real instants of a repeated fall-back time, in chronological order, instead of only the first
- Preserve strict next/previous boundaries when starting inside either occurrence of a DST fold
- Skip nonexistent spring-forward local times
- Return `None` instead of panicking at the `DateTime` minimum/maximum, including for non-UTC offsets
- Find leap-day occurrences across non-leap centuries such as 2100

### Migration
- `ParseError` is now a structured, non-exhaustive struct instead of an enum; match on `ParseError::kind()` instead of the old variants
- The `From<std::num::ParseIntError>` and `From<std::num::TryFromIntError>` impls for `ParseError` were removed; code relying on `?` to convert those errors must map them explicitly
- Repeated, leading, and trailing commas are no longer accepted (`1,,2`, `,5`, `5,`)
- Weekday names (`Mon`–`Sun`) are only accepted in the day-of-week field, including in `parse_field()`, which accepts them only for the `0..=6` range
- Numeric values must contain only ASCII digits; a leading `+` such as `+5` is no longer accepted
- `parse()` and `parse_field()` keep their signatures; `parse()` now delegates to `Schedule`. Prefer compiling a `Schedule` once when evaluating an expression repeatedly

## [0.11.2] - 2025-12-17

### Changed
- Improved `examples/parse.rs` argument handling (`--help`, `--count`/`-n`) and usage output
- Updated `just run-example` recipe to pass through extra CLI args (e.g., `--count 10`)
- Refactored `examples/timezone.rs` to reduce repetition and show parse errors per timezone

### Fixed
- Fixed `examples/patterns.rs` header underline (removed the accidental `PPPP...` line)

## [0.11.1] - 2025-12-15

### Fixed
- Fixed clippy warnings for `indexing_slicing`, `unwrap_used`, `expect_used`, and `panic`
- Fixed `clippy::similar_names` warnings in `src/lib.rs`
- Fixed `clippy::uninlined_format_args` warnings

## [0.11.0] - 2025-11-08

### Added
- Three comprehensive example programs (`parse`, `timezone`, `patterns`)
- Comprehensive unit tests for `make_utc_datetime()` helper function
- Extensive test coverage for range-step patterns (e.g., "0 12-18/3 * * *")
- Additional edge case tests for cron expression parsing (11 new tests)
- GitHub Actions workflows for CI, releases, and publishing to crates.io
- Dependabot configuration for automatic dependency updates
- Justfile with 25+ recipes for common development tasks
- Issue and PR templates for better contributor experience
- Comprehensive release documentation (.github/RELEASE.md)

### Changed
- **Breaking**: Upgraded to Rust edition 2024
- GitHub Actions: Updated all actions to latest versions (checkout@v5, cache@v4, codecov@v4)
- GitHub Actions: Replaced cargo-tarpaulin with grcov for more reliable code coverage
- Improved code organization with helper function `make_utc_datetime()`
- Added derives (`Clone`, `Copy`, `PartialEq`, `Eq`) to internal `Dow` enum
- Enhanced Cargo.toml categories to include "date-and-time"
- Release tags now use pure semver format (0.11.0 instead of v0.11.0)

### Fixed
- Reduced code duplication in datetime creation logic

## [0.10.0] - 2024-12-14

### Added
- Support for start-end/step pattern, e.g. "0 12-18/3 * * *"

## Previous Versions

See git history for changes in versions prior to 0.10.0.
