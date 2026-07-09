# puz-parse

A Rust library for reading and writing the binary `.puz` crossword format used
by AcrossLite and most crossword apps. It parses metadata, the solution and
blank grids, clues, and extensions like rebus and circled squares — and
serializes them back to a spec-correct `.puz` file.

## Contents

- [Installation](#installation)
- [Quick start](#quick-start)
- [Parsing API](#parsing-api)
- [Writing API](#writing-api)
- [Validation](#validation)
- [Data model](#data-model)
- [Warnings and errors](#warnings-and-errors)
- [Feature flags](#feature-flags)
- [Examples](#examples)
- [License](#license)

## Installation

```toml
[dependencies]
puz-parse = "0.1"
```

To derive serde `Serialize`/`Deserialize` on the puzzle types, enable the
`json` feature:

```toml
[dependencies]
puz-parse = { version = "0.1", features = ["json"] }
```

## Quick start

```rust
use puz_parse::parse_file;

fn main() -> Result<(), puz_parse::PuzError> {
    let puzzle = parse_file("puzzle.puz")?;

    println!("{} by {}", puzzle.info.title, puzzle.info.author);
    println!("{}x{}", puzzle.info.width, puzzle.info.height);

    for (number, clue) in &puzzle.clues.across {
        println!("{number} across: {clue}");
    }

    Ok(())
}
```

## Parsing API

There are three entry points, depending on what you're parsing and how much you
care about warnings:

- `parse_file(path)` opens a file and returns a `Puzzle`. This is the
  convenience path and discards any non-fatal warnings.
- `parse(reader)` parses from anything implementing `Read` and returns a
  `ParseResult<Puzzle>`, which carries both the `Puzzle` and any warnings
  collected during parsing.
- `parse_bytes(&[u8])` parses puzzle data already in memory and returns a
  `Puzzle`.

Reach for `parse` when you want to see warnings (for example, a recovered
encoding issue or a skipped extension); use `parse_file` or `parse_bytes` when
you just want the puzzle.

```rust
use puz_parse::parse;
use std::fs::File;

fn main() -> Result<(), puz_parse::PuzError> {
    let file = File::open("puzzle.puz").expect("open puzzle.puz");
    let result = parse(file)?;
    let puzzle = result.result;

    for warning in &result.warnings {
        eprintln!("warning: {warning}");
    }

    println!("parsed {}", puzzle.info.title);

    Ok(())
}
```

## Writing API

The library can serialize a `Puzzle` back into the binary `.puz` format,
computing all checksums (overall, CIB, and the masked "ICHEATED" checksums) plus
per-extension-section checksums, so the output is accepted by other crossword
software:

- `to_bytes(&puzzle)` returns the `.puz` file as a `Vec<u8>`.
- `write(&puzzle, writer)` writes to anything implementing `Write`.
- `write_file(&puzzle, path)` writes straight to a file.

```rust
use puz_parse::{parse_file, write_file};

fn main() -> Result<(), puz_parse::PuzError> {
    let puzzle = parse_file("puzzle.puz")?;
    write_file(&puzzle, "copy.puz")?;
    Ok(())
}
```

Writing validates the puzzle first and returns an error rather than producing a
corrupt file: grids must match the declared dimensions, clue counts must match
the grid, and every string must be encodable in Windows-1252. Scrambled puzzles
are rejected with `PuzError::UnsupportedFeature` (writing the scramble algorithm
is not currently supported).

## Validation

`parse` is lenient about checksums — many real-world `.puz` files have incorrect
ones — so a mismatch is reported as a `PuzWarning::ChecksumMismatch` rather than
an error. When you need to enforce integrity, use the strict entry points, which
recompute all checksums and return `PuzError::InvalidChecksum` on the first
mismatch:

- `parse_strict(reader)` parses but fails on a checksum mismatch.
- `validate_bytes(&[u8])` checks a file's checksums without returning the puzzle.

```rust
use puz_parse::validate_bytes;

fn main() {
    let data = std::fs::read("puzzle.puz").expect("read file");
    match validate_bytes(&data) {
        Ok(()) => println!("checksums valid"),
        Err(e) => eprintln!("invalid: {e}"),
    }
}
```

## Data model

`parse_file` (and the others) give you a `Puzzle`:

```text
Puzzle
├── info: PuzzleInfo    title, author, copyright, notes, width, height,
│                       version, is_scrambled
├── grid: Grid          blank + solution, each a Vec<String> of rows
├── clues: Clues        across + down, each a HashMap<u16, String> keyed by
│                       clue number
└── extensions: Extensions   rebus, circles, given (all optional)
```

Grid rows are strings of single-character cells:

- `.` is a black/blocked square.
- `-` is an empty square (in the blank grid).
- Any letter or number is cell content.

`extensions` is where the less common features live, and each is `Option`:

- `rebus` holds a `Rebus` with a `grid: Vec<Vec<u8>>` marking rebus cells
  (`0` means none) and a `table: HashMap<u8, String>` mapping each key to its
  multi-character value.
- `circles` is a `Vec<Vec<bool>>` marking circled cells, if the puzzle has any.
- `given` is a `Vec<Vec<bool>>` marking cells that were pre-filled for the
  solver, if any.

## Warnings and errors

Parsing distinguishes between problems it can recover from and problems it
can't.

Recoverable problems come back as `PuzWarning` values in
`ParseResult::warnings` (only visible through `parse`). They cover cases like a
skipped extension section, a recovered text-encoding issue, partial data
recovery, or a scrambled puzzle.

Fatal problems return `Err(PuzError)`. Variants describe what went wrong,
including an invalid magic header, a checksum mismatch, bad dimensions, a
section size mismatch, and I/O errors, so you can match on the specific case:

```rust
use puz_parse::{parse_file, PuzError};

fn main() {
    match parse_file("puzzle.puz") {
        Ok(puzzle) => println!("parsed: {}", puzzle.info.title),
        Err(PuzError::InvalidMagic { .. }) => eprintln!("not a .puz file"),
        Err(e) => eprintln!("parse error: {e}"),
    }
}
```

## Feature flags

- `json` (off by default) derives serde `Serialize`/`Deserialize` on `Puzzle`
  and its component types, so a parsed puzzle can be serialized directly.

## Examples

The [`examples/`](examples/) directory has runnable programs. Run one with:

```sh
cargo run --example read-with-file
```

## License

Licensed under the [MIT License](../LICENSE).
