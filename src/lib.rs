//! A library for parsing .puz crossword puzzle files.
//!
//! This library provides functionality to parse the binary .puz file format
//! used by crossword puzzle applications in the early-mid 2000s, like AcrossLite.
//!
//! # Example
//!
//! ```rust,no_run
//! use std::fs::File;
//! use puz_rs::parse;
//!
//! let file = File::open("puzzle.puz")?;
//! let result = parse(file)?;
//! let puzzle = result.result;
//! println!("Title: {}", puzzle.info.title);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod error;
mod parser;
mod types;

pub use error::{PuzError, PuzWarning, ParseResult};
pub use types::*;

use std::io::Read;

/// Parse a .puz file from any source that implements `Read`.
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
/// use puz_rs::parse;
///
/// let file = File::open("puzzle.puz")?;
/// let result = parse(file)?;
/// let puzzle = result.result;
/// for warning in &result.warnings {
///     println!("Warning: {}", warning);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse<R: Read>(reader: R) -> Result<ParseResult<Puzzle>, PuzError> {
    parser::parse_puzzle(reader)
}
