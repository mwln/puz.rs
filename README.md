# puz.rs - A Rust-powered crossword toolkit for working with .puz files

A parsing library and CLI for the binary `.puz` format used by AcrossLite and
most crossword apps.

## Contents

- [Workspace layout](#workspace-layout)
- [Features](#features)
- [Getting started](#getting-started)
- [Usage](#usage)
- [File format](#file-format)
- [Contributing](#contributing)
- [Releasing](#releasing)
- [License](#license)

## Workspace layout

The repository is a Cargo workspace with two crates:

- [`parse/`](parse/) is the parsing library, published as
  [`puz-parse`](https://crates.io/crates/puz-parse).
- [`cli/`](cli/) is a command-line tool built on top of it, published as
  [`puz`](https://crates.io/crates/puz).

The `cli` crate depends on `parse`.

## Features

- Parses the full `.puz` binary format: metadata, grids, and clues.
- Handles extensions like rebus squares and circled cells.
- Validates checksums while parsing and surfaces problems as warnings.
- Optional JSON serialization behind the `json` feature.
- Pure, memory-safe Rust with zero-copy parsing where it's practical.

## Getting started

Clone the repo:

```sh
git clone https://github.com/mwln/puz.rs.git
cd puz.rs
```

The toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml), so
`rustup` picks the right version automatically the first time you run a `cargo`
command here. That keeps local builds in step with CI.

Build and test the whole workspace:

```sh
cargo build
cargo test
```

If you plan to contribute, see [CONTRIBUTING.md](CONTRIBUTING.md) for the
checks CI runs and how to submit changes.

## Usage

### Library

Add `puz-parse` to your `Cargo.toml`, then parse a file:

```rust
use puz_parse::parse_file;

fn main() -> Result<(), puz_parse::PuzError> {
    let puzzle = parse_file("puzzle.puz")?;
    println!("{} by {}", puzzle.info.title, puzzle.info.author);
    Ok(())
}
```

Enable the `json` feature for serde support. See
[parse/README.md](parse/README.md) for the full API, error handling, and more
examples.

### CLI

The `puz` command reads one or more `.puz` files and prints their contents as
JSON:

```sh
puz puzzle.puz --pretty
```

See [cli/README.md](cli/README.md) for the available flags and output format.

## File format

`.puz` is a binary format. [`PUZ.md`](PUZ.md) documents the layout the parser
implements, from the header and checksums through the grid and extension
sections.

## Contributing

Contributions are welcome. [CONTRIBUTING.md](CONTRIBUTING.md) covers how to set
up, the checks to run before opening a pull request, and the review process.

## Releasing

Releases are tag-driven and handled in CI. The process is documented in
[RELEASING.md](RELEASING.md).

## License

Licensed under the [MIT License](LICENSE).
