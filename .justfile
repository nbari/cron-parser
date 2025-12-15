# Default recipe to display help information
default: test
    @just --list

# Build the project
build:
    cargo build

# Build the project in release mode
build-release:
    cargo build --release

# Run all tests
test: clippy format-check
    cargo test -- --nocapture

# Run clippy for linting
clippy:
    cargo clippy --all-targets --all-features

# Check formatting without making changes
format-check:
    cargo fmt -- --check

# Run benchmarks
bench:
    cargo bench

# Build documentation
doc:
    cargo doc --no-deps --open

# Check the project for errors without building
check:
    cargo check --all-targets --all-features

# Clean build artifacts
clean:
    cargo clean

# Generate code coverage report
coverage:
    cargo llvm-cov --all-features --workspace

# Run all quality checks (format, lint, test)
ci: format-check clippy test
    @echo "All CI checks passed!"

# Update dependencies
update:
    cargo update

# Audit dependencies for security vulnerabilities
audit:
    cargo audit

# Run example with a cron expression (usage: just run-example "*/5 * * * *")
run-example cron:
    @echo "Parsing cron expression: {{cron}}"
    @cargo run --example parse -- "{{cron}}"

# Run timezone example showing cron parsing across different timezones
run-timezone-example:
    @cargo run --example timezone

# Run patterns example showing common cron expression patterns
run-patterns-example:
    @cargo run --example patterns

# List all available examples
list-examples:
    @echo "Available examples:"
    @echo "  parse    - Parse cron expressions and show next execution times"
    @echo "  timezone - Demonstrate timezone-aware cron parsing"
    @echo "  patterns - Show common cron expression patterns"
    @echo ""
    @echo "Usage:"
    @echo "  just run-example \"*/5 * * * *\""
    @echo "  just run-timezone-example"
    @echo "  just run-patterns-example"

# Publish to crates.io (dry-run)
publish-dry:
    cargo publish --dry-run

# Publish to crates.io
publish:
    cargo publish
