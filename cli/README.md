# puz

A command-line tool for reading `.puz` crossword files and printing their
contents as JSON. It's built on the
[`puz-parse`](https://crates.io/crates/puz-parse) library.

## Contents

- [Installation](#installation)
- [Usage](#usage)
- [Options](#options)
- [Output format](#output-format)
- [License](#license)

## Installation

```sh
cargo install puz
```

Or build from source:

```sh
git clone https://github.com/mwln/puz.rs.git
cd puz.rs
cargo install --path cli
```

Prebuilt binaries for common platforms are also attached to each
[release](https://github.com/mwln/puz.rs/releases).

## Usage

Parse a file and print JSON to stdout:

```sh
puz puzzle.puz
```

Pretty-print the output:

```sh
puz puzzle.puz --pretty
```

Parse several files at once (they're returned as a JSON array):

```sh
puz puzzle1.puz puzzle2.puz --pretty
```

For a single file, drop the surrounding array and print just the object:

```sh
puz puzzle.puz --single --pretty
```

Write to a file instead of stdout:

```sh
puz puzzle.puz --output output.json
```

## Options

| Option | Description |
| --- | --- |
| `<FILES>...` | One or more `.puz` files to parse. Supports shell globs. |
| `-o, --output <FILE>` | Write output to a file instead of stdout. |
| `-p, --pretty` | Indent the JSON for readability. |
| `-s, --single` | For a single file, output the puzzle object directly instead of wrapping it in an array. |

## Output format

By default the tool prints a JSON array of parsed puzzles, one entry per input
file. With `--single` and exactly one file, it prints that puzzle object on its
own.

Each puzzle object mirrors the `puz-parse` data model:

- `info`: metadata (title, author, copyright, notes, width, height, version,
  scrambled flag)
- `grid`: the blank and solution grids, each an array of row strings
- `clues`: across and down clues, keyed by clue number
- `extensions`: rebus, circled, and given squares, when the puzzle has them

See the [`puz-parse` README](../parse/README.md) for what each field contains.

## License

Licensed under the [MIT License](../LICENSE).
