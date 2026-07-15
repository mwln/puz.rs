# puz

A command-line tool for reading and inspecting `.puz` crossword files. It parses
puzzles to JSON and provides raw-structure views for debugging. It's built on
the [`puz-parse`](https://crates.io/crates/puz-parse) library.

## Contents

- [Installation](#installation)
- [Commands](#commands)
- [Parsing to JSON](#parsing-to-json)
- [Validating a directory](#validating-a-directory)
- [Inspecting a file](#inspecting-a-file)
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

## Commands

```text
puz [FILES]...              parse puzzles to JSON (default)
puz parse [FILES]...        parse puzzles to JSON (explicit form)
puz validate <DIR>          bulk-validate every .puz file under a directory
puz dump header <FILE>      declared dimensions, clue count, bitmask, version
puz dump grid <FILE>        the solution and blank grids, with any mismatches
puz dump strings <FILE>     title, author, copyright, the clue list, and notes
puz dump clues <FILE>       clue numbering vs. the file's declared/provided clues
puz inspect sections <FILE> extension sections (GRBS, RTBL, GEXT, ...)
```

The `dump` and `inspect` commands read the file bytes directly rather than fully
parsing, so they still produce useful output for files that fail to parse.

## Parsing to JSON

Running `puz` with file arguments (no subcommand) parses them to JSON, the same
as `puz parse`. Parse warnings are printed to stderr.

Parse a file and print JSON to stdout:

```sh
puz puzzle.puz
```

Pretty-print the output:

```sh
puz puzzle.puz --pretty
```

Parse several files at once (returned as a JSON array):

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

### Options (parse)

| Option | Description |
| --- | --- |
| `<FILES>...` | One or more `.puz` files to parse. Supports shell globs. |
| `-o, --output <FILE>` | Write output to a file instead of stdout. |
| `-p, --pretty` | Indent the JSON for readability. |
| `-s, --single` | For a single file, output the puzzle object directly instead of wrapping it in an array. |

## Validating a directory

Recursively parse every `.puz` file under a directory and print a summary of
parse errors and warnings:

```sh
puz validate ./puzzles
```

| Option | Description |
| --- | --- |
| `<DIR>` | Directory to scan recursively for `.puz` files. |
| `--verbose` | Print a line for every file, including clean ones. |
| `--errors-only` | Print only hard parse failures, not warnings. |

## Inspecting a file

The `dump` and `inspect` commands show a file's raw structure. They are useful
for understanding an unusual puzzle or debugging one that does not parse.

```sh
puz dump header  puzzle.puz    # dimensions, clue count, bitmask, version
puz dump grid    puzzle.puz    # solution + blank grids, black-square mismatches
puz dump strings puzzle.puz    # title/author/copyright, numbered clues, notes
puz dump clues   puzzle.puz    # computed clue numbering vs. the file's clue list
puz inspect sections puzzle.puz  # GRBS / RTBL / GEXT extension sections
```

`dump clues` is handy for puzzles whose declared clue count does not match the
grid geometry: it shows the across/down slot counts, the declared `num_clues`,
the number of clue strings in the file, and any extras.

## Output format

The `parse` command (and the bare `puz FILES...` default) prints a JSON array of
parsed puzzles, one entry per input file. With `--single` and exactly one file,
it prints that puzzle object on its own.

Each puzzle object mirrors the `puz-parse` data model:

- `info`: metadata (title, author, copyright, notes, width, height, version,
  scrambled flag, diagramless flag)
- `grid`: the blank and solution grids, each an array of row strings
- `clues`: across and down clues keyed by clue number, plus the raw clue list
- `extensions`: rebus, circled, and given squares, when the puzzle has them

See the [`puz-parse` README](../parse/README.md) for what each field contains.

## License

Licensed under the [MIT License](../LICENSE).
