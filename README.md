# puz.rs

A rust workspace for interacting with `.puz` crossword puzzle files.

## Structure

- **[`parse/`](parse/)** - core parsing library (`puz-parse` crate)
- **[`cli/`](cli/)** - cli tool for processing files (`puz` crate)

## Quick Start

For detailed installation and usage instructions, see the individual package READMEs:

- **library**: See [parse/README.md](parse/README.md) for complete API documentation and examples
- **cli**: See [cli/README.md](cli/README.md) for command-line interface options

## Features

- Complete `.puz` file format parsing
- Support for rebus squares, circles, and other extensions
- JSON serialization support
- Memory-safe, zero-copy parsing where possible
- Comprehensive error handling with warnings

## File Format

The library parses the binary `.puz` format used by crossword applications like AcrossLite. See `PUZ.md` for complete format documentation.
