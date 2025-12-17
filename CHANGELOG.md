# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
