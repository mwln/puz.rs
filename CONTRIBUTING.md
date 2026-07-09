# Contributing

Thanks for your interest in improving `puz.rs`. This guide covers how to get
set up, run the checks, and open a pull request.

## Contents

- [Setup](#setup)
- [Project layout](#project-layout)
- [Running the checks](#running-the-checks)
- [Testing](#testing)
- [Opening a pull request](#opening-a-pull-request)
- [Releases](#releases)

## Setup

You'll need Rust. Install it with [`rustup`](https://rustup.rs), which manages
the toolchain for you.

Clone the repo and let `rustup` handle the rest. The pinned version in
[`rust-toolchain.toml`](rust-toolchain.toml) is installed automatically the
first time you run a `cargo` command, so your build matches CI.

```sh
git clone https://github.com/mwln/puz.rs.git
cd puz.rs
cargo build
```

From inside the repo, confirm `rustup` is using the pinned toolchain:

```sh
rustup show active-toolchain
```

You should see it reported as overridden by `rust-toolchain.toml`, for example
`stable-aarch64-apple-darwin (overridden by '.../rust-toolchain.toml')`. If it
isn't, run any `cargo` command in the repo once and `rustup` will install and
select it.

Most of the workflow runs through [`just`](https://github.com/casey/just), so
install it once:

```sh
cargo install just
```

Run `just` with no arguments to see the available recipes.

## Project layout

This is a Cargo workspace with two crates:

- [`parse/`](parse/) is the `puz-parse` library that does the actual `.puz`
  parsing.
- [`cli/`](cli/) is the `puz` command-line tool, which depends on `puz-parse`.

## Running the checks

Before pushing, run the same checks CI runs:

```sh
just check
```

That covers formatting, clippy, tests, and the docs build. You can also run the
pieces individually:

```sh
just fmt       # format the code
just lint      # clippy
just test      # tests
just docs      # build the docs
```

One thing worth knowing: `puz-parse` has an optional `json` feature, and CI
checks the library both with and without it. `just lint` and `just test` run
those variants for you, so `just check` catches feature-related breakage that a
plain `cargo test` would miss.

Clippy warnings are denied in CI, so anything `just lint` reports has to be
resolved before a PR can merge.

## Testing

Run the full suite with `just test`, or target a single test the usual way:

```sh
cargo test test_name
```

## Opening a pull request

- Make sure `just check` passes.
- Keep commits focused; a PR that does one thing is easier to review.
- The minimum supported Rust version is **1.78.0**. CI tests against it, so
  avoid standard-library APIs newer than that.
- If you add or change behavior, add a test for it.

CI runs the same checks as `just check` across stable, beta, and the 1.78.0
MSRV, plus a docs build and cross-compilation checks.

## Releases

Releases are maintainer-only and handled through CI. If you're curious how it
works, see [RELEASING.md](RELEASING.md).
