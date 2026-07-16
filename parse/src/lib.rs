//! A library for parsing .puz crossword puzzle files.
//!
//! This library provides functionality to parse the binary .puz file format
//! used by crossword puzzle applications in the early-mid 2000s, like AcrossLite.
//! It supports all standard puzzle features including rebus squares, circled squares,
//! and various puzzle extensions.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use puz_parse::Puzzle;
//!
//! let puzzle = Puzzle::from_file("puzzle.puz")?;
//! println!("Title: {}", puzzle.info.title);
//! println!("Size: {}x{}", puzzle.info.width, puzzle.info.height);
//! # Ok::<(), puz_parse::PuzError>(())
//! ```
//!
//! # Advanced Usage
//!
//! For more control over parsing, configure a [`Puzzle::reader`]. The
//! `*_verbose` terminals also return any warnings collected during parsing:
//!
//! ```rust,no_run
//! use puz_parse::Puzzle;
//!
//! let parsed = Puzzle::reader().from_file_verbose("puzzle.puz")?;
//! let puzzle = &parsed.result;
//! println!("Title: {}", puzzle.info.title);
//!
//! for warning in &parsed.warnings {
//!     eprintln!("Warning: {}", warning);
//! }
//! # Ok::<(), puz_parse::PuzError>(())
//! ```
//!
//! # Writing
//!
//! The library can also serialize a [`Puzzle`] back into the binary `.puz`
//! format, computing all checksums so the output is accepted by other
//! crossword software:
//!
//! ```rust,no_run
//! use puz_parse::{Puzzle, to_bytes, write_file};
//!
//! let puzzle = Puzzle::from_file("puzzle.puz")?;
//! // in-memory bytes:
//! let bytes = to_bytes(&puzzle)?;
//! // or straight to a file:
//! write_file(&puzzle, "copy.puz")?;
//! # Ok::<(), puz_parse::PuzError>(())
//! ```
//!
//! # Validation
//!
//! [`Puzzle::from_file`] and friends record checksum mismatches as
//! [`PuzWarning::ChecksumMismatch`] and continue (many real-world files have
//! incorrect checksums). Use `Puzzle::reader().strict(true)` or
//! [`validate_bytes`] to treat a checksum mismatch as an error instead.
//!
//! # Building
//!
//! Build a puzzle by chaining setters from [`Puzzle::new`]. The grid uses `.`
//! for black squares; the chain is infallible and the puzzle is validated when
//! written:
//!
//! ```rust
//! use puz_parse::Puzzle;
//!
//! let puzzle = Puzzle::new()
//!     .title("Example")
//!     .author("Me")
//!     .grid(["AB.", "CDE"])
//!     .diagramless(true);
//! ```
//!
//! Read and write individual clues through the [`Clues`] API. Each direction is
//! a [`ClueSet`] keyed by clue number:
//!
//! ```rust
//! use puz_parse::Puzzle;
//!
//! let mut puzzle = Puzzle::new().grid(["AB", "CD"]);
//! puzzle.clues.across.set(1, "First across");
//! assert_eq!(puzzle.clues.across.get(1), Some("First across"));
//! ```
//!
//! # Features
//!
//! - **Complete .puz parsing**: Supports all standard puzzle features
//! - **Writing**: Serialize a `Puzzle` back to `.puz` with correct checksums
//! - **Diagramless**: Parses and writes diagramless puzzles (`:` black squares);
//!   [`PuzzleInfo::is_diagramless`] flags them
//! - **Validation**: Optional strict checksum verification
//! - **Error recovery**: Continues parsing with warnings for non-critical issues
//! - **Extensible**: Handles rebus squares, circles, and other puzzle extensions
//! - **JSON support**: Optional serde support via the `json` feature
//!
//! # Optional Features
//!
//! - `json`: Enables JSON serialization support via serde

mod checksums;
mod encoding;
mod error;
mod grid;
mod parser;
mod puzzle;
pub mod raw;
mod types;
mod writer;

pub use error::{ParseResult, PuzError, PuzWarning};
pub use puzzle::{Puzzle, PuzzleReader};
pub use types::{ClueAnswer, ClueSet, Clues, Direction, Extensions, Grid, PuzzleInfo, Rebus};

use std::io::Read;
use std::path::Path;

/// Parse a .puz file from any source that implements `Read`, returning the
/// puzzle and any warnings.
///
/// # Example
///
/// ```rust,no_run
/// use puz_parse::Puzzle;
///
/// let parsed = Puzzle::reader().from_file_verbose("puzzle.puz")?;
/// for warning in &parsed.warnings {
///     eprintln!("Warning: {}", warning);
/// }
/// # Ok::<(), puz_parse::PuzError>(())
/// ```
#[deprecated(
    since = "0.2.0",
    note = "use `Puzzle::from_reader` (or `Puzzle::reader().from_reader_verbose`) instead"
)]
pub fn parse<R: Read>(reader: R) -> Result<ParseResult<Puzzle>, PuzError> {
    parser::parse_puzzle(reader)
}

/// Parse a .puz file, requiring all stored checksums to match.
///
/// Returns [`PuzError::InvalidChecksum`] on the first mismatch instead of
/// recording a warning.
#[deprecated(
    since = "0.2.0",
    note = "use `Puzzle::reader().strict(true).from_reader_verbose(..)` instead"
)]
pub fn parse_strict<R: Read>(reader: R) -> Result<ParseResult<Puzzle>, PuzError> {
    parser::parse_puzzle_strict(reader)
}

/// Validate the checksums of a .puz file without returning the puzzle.
///
/// Returns `Ok(())` if all stored checksums match, or
/// [`PuzError::InvalidChecksum`] on the first mismatch (or a parse error if the
/// data is malformed).
///
/// # Example
///
/// ```rust,no_run
/// use puz_parse::Puzzle;
///
/// let data = std::fs::read("puzzle.puz")?;
/// match Puzzle::reader().strict(true).from_bytes(&data) {
///     Ok(_) => println!("checksums valid"),
///     Err(e) => eprintln!("invalid: {e}"),
/// }
/// # Ok::<(), puz_parse::PuzError>(())
/// ```
#[deprecated(
    since = "0.2.0",
    note = "use `Puzzle::reader().strict(true).from_bytes(..)` instead"
)]
pub fn validate_bytes(data: &[u8]) -> Result<(), PuzError> {
    parser::parse_puzzle_strict(data).map(|_| ())
}

/// Parse a .puz file from a file path.
///
/// This is a convenience function that handles file opening and returns just the
/// puzzle data. Warnings are discarded. Use [`Puzzle::from_file`] for full control.
///
/// # Arguments
///
/// * `path` - Path to the .puz file
///
/// # Returns
///
/// Returns the parsed `Puzzle` or an error if parsing fails.
///
/// # Example
///
/// ```rust,no_run
/// use puz_parse::Puzzle;
///
/// let puzzle = Puzzle::from_file("puzzle.puz")?;
/// println!("Puzzle: {} by {}", puzzle.info.title, puzzle.info.author);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[deprecated(since = "0.2.0", note = "use `Puzzle::from_file` instead")]
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Puzzle, PuzError> {
    Puzzle::from_file(path)
}

/// Parse a .puz file from a byte slice.
///
/// Convenience function for parsing puzzle data already in memory.
///
/// # Arguments
///
/// * `data` - Byte slice containing .puz file data
///
/// # Returns
///
/// Returns the parsed `Puzzle` or an error if parsing fails.
///
/// # Example
///
/// ```rust,no_run
/// use puz_parse::Puzzle;
///
/// let data = std::fs::read("puzzle.puz")?;
/// let puzzle = Puzzle::from_bytes(&data)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[deprecated(since = "0.2.0", note = "use `Puzzle::from_bytes` instead")]
pub fn parse_bytes(data: &[u8]) -> Result<Puzzle, PuzError> {
    Puzzle::from_bytes(data)
}

/// Serialize a puzzle to an in-memory `.puz` byte buffer.
///
/// This is the core writing function. Use [`write()`] or [`write_file`] to send
/// the bytes to a sink or file.
///
/// # Example
///
/// ```rust,no_run
/// use puz_parse::{Puzzle, to_bytes};
///
/// let puzzle = Puzzle::from_file("puzzle.puz")?;
/// let bytes = to_bytes(&puzzle)?;
/// # Ok::<(), puz_parse::PuzError>(())
/// ```
pub fn to_bytes(puzzle: &Puzzle) -> Result<Vec<u8>, PuzError> {
    writer::write_puzzle(puzzle)
}

/// Write a puzzle to any type that implements `Write`.
pub fn write<W: std::io::Write>(puzzle: &Puzzle, mut writer: W) -> Result<(), PuzError> {
    let bytes = to_bytes(puzzle)?;
    writer.write_all(&bytes)?;
    Ok(())
}

/// Write a puzzle to a file path.
///
/// # Example
///
/// ```rust,no_run
/// use puz_parse::{Puzzle, write_file};
///
/// let puzzle = Puzzle::from_file("puzzle.puz")?;
/// write_file(&puzzle, "copy.puz")?;
/// # Ok::<(), puz_parse::PuzError>(())
/// ```
pub fn write_file<P: AsRef<Path>>(puzzle: &Puzzle, path: P) -> Result<(), PuzError> {
    let file = std::fs::File::create(path.as_ref())?;
    write(puzzle, file)
}
