# Show available recipes.
default:
    @just --list

# Format the whole workspace.
fmt:
    cargo fmt --all

# Check formatting without modifying files (as CI does).
fmt-check:
    cargo fmt --all -- --check

# Run clippy across the workspace and both puz-parse feature variants.
lint:
    cargo clippy --all --all-targets -- -D warnings -D clippy::all
    cargo clippy -p puz-parse --no-default-features -- -D warnings -D clippy::all
    cargo clippy -p puz-parse --features json -- -D warnings -D clippy::all

# Run the tests, including both puz-parse feature variants and the CLI smoke test.
test:
    cargo test --all
    cargo test -p puz-parse --no-default-features
    cargo test -p puz-parse --features json
    cargo run --bin puz -- --help

# Build the docs with warnings denied (as CI does).
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all --all-features

# Run everything CI checks. Run this before pushing.
check: fmt-check lint test docs
