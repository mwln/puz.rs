use std::io;

use thiserror::Error;

/// Warnings that can occur during parsing but don't prevent puzzle creation.
///
/// These indicate non-critical issues that were encountered during parsing
/// but were handled gracefully. The parsing can continue and produce a valid
/// puzzle, but some information might be missing or using fallback values.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum PuzWarning {
    /// An optional extension section was skipped due to parsing issues.
    #[error("Skipped extension section '{section}': {reason}")]
    SkippedExtension { section: String, reason: String },

    /// Character encoding issues were encountered but handled.
    #[error("Encoding issue in {context}: {}", recovery_note(*recovered))]
    EncodingIssue { context: String, recovered: bool },

    /// Invalid data was found but default values were used.
    #[error("Data recovery for '{field}': {issue}")]
    DataRecovery { field: String, issue: String },

    /// Puzzle is scrambled and may not display correctly.
    #[error(
        "Puzzle is scrambled (version {version}). Solution may not be readable without descrambling."
    )]
    ScrambledPuzzle { version: String },

    /// A stored checksum did not match the recomputed value (non-fatal in
    /// lenient parsing; many real-world files have incorrect checksums).
    #[error(
        "Checksum mismatch in {context}: expected 0x{expected:04X}, found 0x{found:04X}. The file may be corrupted."
    )]
    ChecksumMismatch {
        context: String,
        expected: u16,
        found: u16,
    },

    /// A solution cell holds a non-standard character (not a letter, digit, or
    /// black square) with no rebus entry at that position. It could be a rebus
    /// glyph the file failed to describe, or it could be corruption.
    #[error(
        "Solution cell ({row}, {col}) has non-standard character '{character}' (U+{:04X}) with no rebus entry at that position.",
        *character as u32
    )]
    UnbackedGridChar {
        character: char,
        row: usize,
        col: usize,
    },

    /// The file stored more clue strings than the grid has word slots. The
    /// extra strings are preserved in [`Clues::raw`](crate::Clues::raw); the
    /// numbered across/down maps use the first `slots` clues in reading order.
    /// Some puzzles include extra authored clues (for example a meta-puzzle
    /// revealer) that do not correspond to a grid slot.
    #[error(
        "File provided {provided} clue strings but the grid has {slots} word slots; \
         the {} extra clue(s) are preserved in Clues::raw.",
        provided - slots
    )]
    ExtraClues {
        /// Number of grid word slots (mapped clues).
        slots: usize,
        /// Number of clue strings the file provided.
        provided: usize,
    },
}

fn recovery_note(recovered: bool) -> &'static str {
    if recovered {
        "recovered using fallback"
    } else {
        "could not recover"
    }
}

/// Result type for parsing that includes warnings.
///
/// Returned by the verbose parse entry points (for example
/// [`PuzzleReader::from_file_verbose`](crate::PuzzleReader::from_file_verbose)):
/// it holds the parsed puzzle alongside any non-fatal [`PuzWarning`]s.
#[derive(Debug)]
pub struct ParseResult<T> {
    /// The successfully parsed puzzle.
    pub result: T,
    /// Any warnings that occurred during parsing.
    pub warnings: Vec<PuzWarning>,
}

impl<T> ParseResult<T> {
    pub(crate) fn with_warnings(result: T, warnings: Vec<PuzWarning>) -> Self {
        Self { result, warnings }
    }
}

/// Errors that can occur when parsing or writing a .puz file.
///
/// These represent critical issues that prevent successful processing. Unlike
/// [`PuzWarning`], they cause the operation to fail. `PuzError` implements
/// [`std::error::Error`], so the underlying I/O error is reachable via
/// [`Error::source`](std::error::Error::source).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PuzError {
    /// The file magic header is invalid.
    #[error(
        "Invalid .puz file magic header. Expected 'ACROSS&DOWN\\0', found: {found:?}. This file may be corrupted or not a .puz file."
    )]
    InvalidMagic { found: Vec<u8> },

    /// Checksum validation failed.
    #[error(
        "Checksum validation failed in {context}: expected 0x{expected:04X}, found 0x{found:04X}. The file may be corrupted."
    )]
    InvalidChecksum {
        expected: u16,
        found: u16,
        context: String,
    },

    /// Puzzle dimensions are invalid.
    #[error("Invalid puzzle dimensions: {width}x{height}. Dimensions must be between 1 and 255.")]
    InvalidDimensions { width: u8, height: u8 },

    /// Clue count doesn't match the expected value.
    #[error(
        "Clue count mismatch: expected {expected} clues, found {found}. The file may be corrupted."
    )]
    InvalidClueCount { expected: u16, found: usize },

    /// Extension section size mismatch.
    #[error(
        "Extension section '{section}' size mismatch: expected {expected} bytes, found {found}. The section may be corrupted."
    )]
    SectionSizeMismatch {
        section: String,
        expected: usize,
        found: usize,
    },

    /// An I/O error occurred while reading or writing the file.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// The file contains invalid UTF-8 data.
    #[error("Invalid UTF-8 data: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),

    /// The file version is not supported.
    #[error("Unsupported .puz file version: '{version}'. Only standard versions are supported.")]
    UnsupportedVersion { version: String },

    /// Grid validation failed.
    #[error("Invalid puzzle grid: {reason}")]
    InvalidGrid { reason: String },

    /// Clue processing failed.
    #[error("Invalid clues: {reason}")]
    InvalidClues { reason: String },

    /// A string contains a character that cannot be encoded in Windows-1252.
    #[error(
        "Cannot encode character {character:?} in {context}: not representable in Windows-1252."
    )]
    EncodingError { character: char, context: String },

    /// A requested puzzle feature is not supported by the writer.
    #[error("Writing is not supported for: {feature}.")]
    UnsupportedFeature { feature: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    #[test]
    fn test_io_error_from_and_source_chain() {
        let io = io::Error::new(io::ErrorKind::NotFound, "boom");
        let err: PuzError = io.into();
        assert!(matches!(err, PuzError::Io(_)));
        // The originating io::Error is reachable via the error source chain.
        let source = err.source().expect("Io variant should expose its source");
        assert_eq!(source.to_string(), "boom");
    }

    #[test]
    fn test_non_io_error_has_no_source() {
        let err = PuzError::InvalidGrid {
            reason: "bad".to_string(),
        };
        assert!(err.source().is_none());
    }

    #[test]
    fn test_display_messages() {
        let err = PuzError::InvalidDimensions {
            width: 0,
            height: 5,
        };
        assert!(err.to_string().contains("0x5"));

        let warn = PuzWarning::ExtraClues {
            slots: 76,
            provided: 78,
        };
        assert!(warn.to_string().contains("2 extra"));
    }
}
