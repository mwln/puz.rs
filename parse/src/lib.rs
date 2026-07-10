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
//! use puz_parse::parse_file;
//!
//! let puzzle = parse_file("puzzle.puz")?;
//! println!("Title: {}", puzzle.info.title);
//! println!("Size: {}x{}", puzzle.info.width, puzzle.info.height);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Advanced Usage
//!
//! For more control over parsing and error handling:
//!
//! ```rust,no_run
//! use std::fs::File;
//! use puz_parse::parse;
//!
//! let file = File::open("puzzle.puz")?;
//! let result = parse(file)?;
//! let puzzle = result.result;
//!
//! // Handle any warnings that occurred during parsing
//! for warning in &result.warnings {
//!     eprintln!("Warning: {}", warning);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Writing
//!
//! The library can also serialize a [`Puzzle`] back into the binary `.puz`
//! format, computing all checksums so the output is accepted by other
//! crossword software:
//!
//! ```rust,no_run
//! use puz_parse::{parse_file, to_bytes, write_file};
//!
//! let puzzle = parse_file("puzzle.puz")?;
//! // in-memory bytes:
//! let bytes = to_bytes(&puzzle)?;
//! // or straight to a file:
//! write_file(&puzzle, "copy.puz")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Validation
//!
//! [`parse`] records checksum mismatches as [`PuzWarning::ChecksumMismatch`]
//! and continues (many real-world files have incorrect checksums). Use
//! [`parse_strict`] or [`validate_bytes`] to treat a checksum mismatch as an
//! error instead.
//!
//! # Building
//!
//! Construct a puzzle from solution rows with [`Puzzle::new`] (using `.` for
//! black squares) and refine it with chained setters:
//!
//! ```rust
//! use puz_parse::Puzzle;
//!
//! let puzzle = Puzzle::new(["AB.", "CDE"])?
//!     .title("Example")
//!     .diagramless(true);
//! # Ok::<(), puz_parse::PuzError>(())
//! ```
//!
//! Read and write individual clues through the [`Clues`] API. Each direction is
//! a [`ClueSet`] keyed by clue number:
//!
//! ```rust
//! use puz_parse::Puzzle;
//!
//! let mut puzzle = Puzzle::new(["AB", "CD"])?;
//! puzzle.clues.across.set(1, "First across");
//! assert_eq!(puzzle.clues.across.get(1), Some("First across"));
//! # Ok::<(), puz_parse::PuzError>(())
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
mod types;
mod writer;

pub use error::{ParseResult, PuzError, PuzWarning};
pub use puzzle::Puzzle;
pub use types::*;

use std::io::Read;
use std::path::Path;

/// Parse a .puz file from any source that implements `Read`.
///
/// This is the core parsing function that provides full control over error handling
/// and warnings. Use [`parse_file`] for a simpler API when parsing from files.
///
/// # Arguments
///
/// * `reader` - Any type that implements `Read`, such as a `File` or `&[u8]`
///
/// # Returns
///
/// Returns a `Result<ParseResult<Puzzle>, PuzError>` containing the parsed puzzle data
/// along with any warnings, or an error if parsing fails.
///
/// # Example
///
/// ```rust,no_run
/// use std::fs::File;
/// use puz_parse::parse;
///
/// let file = File::open("puzzle.puz")?;
/// let result = parse(file)?;
/// let puzzle = result.result;
/// for warning in &result.warnings {
///     eprintln!("Warning: {}", warning);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse<R: Read>(reader: R) -> Result<ParseResult<Puzzle>, PuzError> {
    parser::parse_puzzle(reader)
}

/// Parse a .puz file, requiring all stored checksums to match.
///
/// Unlike [`parse`], which records checksum mismatches as
/// [`PuzWarning::ChecksumMismatch`] and continues, this returns
/// [`PuzError::InvalidChecksum`] on the first mismatch. Use this when you need
/// to reject files whose integrity checks fail.
pub fn parse_strict<R: Read>(reader: R) -> Result<ParseResult<Puzzle>, PuzError> {
    parser::parse_puzzle_strict(reader)
}

/// Validate the checksums of a .puz file without returning the puzzle.
///
/// Returns `Ok(())` if all stored checksums match the recomputed values, or
/// [`PuzError::InvalidChecksum`] on the first mismatch (or a parse error if the
/// data is malformed).
///
/// # Example
///
/// ```rust,no_run
/// use puz_parse::validate_bytes;
///
/// let data = std::fs::read("puzzle.puz")?;
/// match validate_bytes(&data) {
///     Ok(()) => println!("checksums valid"),
///     Err(e) => eprintln!("invalid: {e}"),
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn validate_bytes(data: &[u8]) -> Result<(), PuzError> {
    parser::parse_puzzle_strict(data).map(|_| ())
}

/// Parse a .puz file from a file path.
///
/// This is a convenience function that handles file opening and returns just the
/// puzzle data. Warnings are discarded. Use [`parse`] for full control.
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
/// use puz_parse::parse_file;
///
/// let puzzle = parse_file("puzzle.puz")?;
/// println!("Puzzle: {} by {}", puzzle.info.title, puzzle.info.author);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Puzzle, PuzError> {
    let file = std::fs::File::open(path.as_ref()).map_err(|e| PuzError::IoError {
        message: format!("Failed to open file: {e}"),
        kind: e.kind(),
        position: None,
    })?;

    let result = parse(file)?;
    Ok(result.result)
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
/// use puz_parse::parse_bytes;
///
/// let data = std::fs::read("puzzle.puz")?;
/// let puzzle = parse_bytes(&data)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_bytes(data: &[u8]) -> Result<Puzzle, PuzError> {
    let result = parse(data)?;
    Ok(result.result)
}

/// Serialize a puzzle to an in-memory `.puz` byte buffer.
///
/// This is the core writing function. Use [`write()`] or [`write_file`] to send
/// the bytes to a sink or file.
///
/// # Example
///
/// ```rust,no_run
/// use puz_parse::{parse_file, to_bytes};
///
/// let puzzle = parse_file("puzzle.puz")?;
/// let bytes = to_bytes(&puzzle)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn to_bytes(puzzle: &Puzzle) -> Result<Vec<u8>, PuzError> {
    writer::write_puzzle(puzzle)
}

/// Write a puzzle to any type that implements `Write`.
pub fn write<W: std::io::Write>(puzzle: &Puzzle, mut writer: W) -> Result<(), PuzError> {
    let bytes = to_bytes(puzzle)?;
    writer.write_all(&bytes).map_err(|e| PuzError::IoError {
        message: format!("Failed to write puzzle: {e}"),
        kind: e.kind(),
        position: None,
    })
}

/// Write a puzzle to a file path.
///
/// # Example
///
/// ```rust,no_run
/// use puz_parse::{parse_file, write_file};
///
/// let puzzle = parse_file("puzzle.puz")?;
/// write_file(&puzzle, "copy.puz")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn write_file<P: AsRef<Path>>(puzzle: &Puzzle, path: P) -> Result<(), PuzError> {
    let file = std::fs::File::create(path.as_ref()).map_err(|e| PuzError::IoError {
        message: format!("Failed to create file: {e}"),
        kind: e.kind(),
        position: None,
    })?;
    write(puzzle, file)
}
