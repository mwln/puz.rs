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
//! # Features
//!
//! - **Complete .puz parsing**: Supports all standard puzzle features
//! - **Error recovery**: Continues parsing with warnings for non-critical issues  
//! - **Memory efficient**: Zero-copy parsing where possible
//! - **Extensible**: Handles rebus squares, circles, and other puzzle extensions
//! - **JSON support**: Optional serde support via the `json` feature
//!
//! # Optional Features
//!
//! - `json`: Enables JSON serialization support via serde

mod error;
mod parser;
mod types;

pub use error::{ParseResult, PuzError, PuzWarning};
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
        message: format!("Failed to open file: {}", e),
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
