use std::{error::Error as StdError, fmt, io};

/// Warnings that can occur during parsing but don't prevent puzzle creation
#[derive(Debug, Clone, PartialEq)]
pub enum PuzWarning {
    /// An optional extension section was skipped due to parsing issues
    SkippedExtension { section: String, reason: String },
    /// Character encoding issues were encountered but handled
    EncodingIssue { context: String, recovered: bool },
    /// Invalid data was found but default values were used
    DataRecovery { field: String, issue: String },
    /// Puzzle is scrambled and may not display correctly
    ScrambledPuzzle { version: String },
}

/// Result type for parsing that includes warnings
#[derive(Debug)]
pub struct ParseResult<T> {
    pub result: T,
    pub warnings: Vec<PuzWarning>,
}

impl<T> ParseResult<T> {
    pub fn new(result: T) -> Self {
        Self {
            result,
            warnings: Vec::new(),
        }
    }

    pub fn with_warnings(result: T, warnings: Vec<PuzWarning>) -> Self {
        Self { result, warnings }
    }

    pub fn add_warning(&mut self, warning: PuzWarning) {
        self.warnings.push(warning);
    }
}

/// Errors that can occur when parsing a .puz file.
#[derive(Debug, Clone, PartialEq)]
pub enum PuzError {
    /// The file magic header is invalid
    InvalidMagic { found: Vec<u8> },

    /// Checksum validation failed
    InvalidChecksum {
        expected: u16,
        found: u16,
        context: String,
    },

    /// Puzzle dimensions are invalid
    InvalidDimensions { width: u8, height: u8 },

    /// Clue count doesn't match expected value
    InvalidClueCount { expected: u16, found: usize },

    /// Extension section size mismatch
    SectionSizeMismatch {
        section: String,
        expected: usize,
        found: usize,
    },

    /// Parse error with position context
    ParseError {
        message: String,
        position: Option<u64>,
        context: String,
    },

    /// An I/O error occurred while reading the file
    IoError {
        message: String,
        kind: io::ErrorKind,
        position: Option<u64>,
    },

    /// The file contains invalid UTF-8 data
    InvalidUtf8 {
        message: String,
        position: Option<u64>,
    },

    /// Required data is missing from the file
    MissingData {
        field: String,
        position: Option<u64>,
    },

    /// The file version is not supported
    UnsupportedVersion { version: String },

    /// Grid validation failed
    InvalidGrid { reason: String },

    /// Clue processing failed
    InvalidClues { reason: String },
}

impl fmt::Display for PuzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PuzError::InvalidMagic { found } => {
                write!(f, "Invalid .puz file magic header. Expected 'ACROSS&DOWN\\0', found: {found:?}. This file may be corrupted or not a .puz file.")
            }
            PuzError::InvalidChecksum {
                expected,
                found,
                context,
            } => {
                write!(f, "Checksum validation failed in {context}: expected 0x{expected:04X}, found 0x{found:04X}. The file may be corrupted.")
            }
            PuzError::InvalidDimensions { width, height } => {
                write!(
                    f,
                    "Invalid puzzle dimensions: {width}x{height}. Dimensions must be between 1 and 255."
                )
            }
            PuzError::InvalidClueCount { expected, found } => {
                write!(
                    f,
                    "Clue count mismatch: expected {expected} clues, found {found}. The file may be corrupted."
                )
            }
            PuzError::SectionSizeMismatch {
                section,
                expected,
                found,
            } => {
                write!(f, "Extension section '{section}' size mismatch: expected {expected} bytes, found {found}. The section may be corrupted.")
            }
            PuzError::ParseError {
                message,
                position,
                context,
            } => match position {
                Some(pos) => write!(f, "Parse error at position {pos}: {message} ({context})"),
                None => write!(f, "Parse error: {message} ({context})"),
            },
            PuzError::IoError {
                message,
                kind,
                position,
            } => match position {
                Some(pos) => write!(f, "I/O error at position {pos}: {message} ({kind:?})"),
                None => write!(f, "I/O error: {message} ({kind:?})"),
            },
            PuzError::InvalidUtf8 { message, position } => match position {
                Some(pos) => write!(f, "Invalid UTF-8 data at position {pos}: {message}"),
                None => write!(f, "Invalid UTF-8 data: {message}"),
            },
            PuzError::MissingData { field, position } => match position {
                Some(pos) => write!(f, "Missing required data '{field}' at position {pos}"),
                None => write!(f, "Missing required data: {field}"),
            },
            PuzError::UnsupportedVersion { version } => {
                write!(
                    f,
                    "Unsupported .puz file version: '{version}'. Only standard versions are supported."
                )
            }
            PuzError::InvalidGrid { reason } => {
                write!(f, "Invalid puzzle grid: {reason}")
            }
            PuzError::InvalidClues { reason } => {
                write!(f, "Invalid clues: {reason}")
            }
        }
    }
}

impl StdError for PuzError {}

impl From<io::Error> for PuzError {
    fn from(error: io::Error) -> Self {
        PuzError::IoError {
            message: format!("I/O operation failed: {error}"),
            kind: error.kind(),
            position: None,
        }
    }
}

impl From<std::str::Utf8Error> for PuzError {
    fn from(error: std::str::Utf8Error) -> Self {
        PuzError::InvalidUtf8 {
            message: format!("UTF-8 decoding failed: {error}"),
            position: None,
        }
    }
}

impl PuzError {
    /// Add position context to an existing error
    pub fn with_position(mut self, position: u64) -> Self {
        match &mut self {
            PuzError::IoError { position: pos, .. } => *pos = Some(position),
            PuzError::InvalidUtf8 { position: pos, .. } => *pos = Some(position),
            PuzError::MissingData { position: pos, .. } => *pos = Some(position),
            PuzError::ParseError { position: pos, .. } => *pos = Some(position),
            _ => {} // Other error types don't have position fields
        }
        self
    }

    /// Add context to an existing error
    pub fn with_context(self, context: &str) -> Self {
        match self {
            PuzError::IoError {
                message,
                kind,
                position,
            } => PuzError::IoError {
                message: format!("{context}: {message}"),
                kind,
                position,
            },
            PuzError::InvalidUtf8 { message, position } => PuzError::InvalidUtf8 {
                message: format!("{context}: {message}"),
                position,
            },
            PuzError::ParseError {
                message,
                position,
                context: existing_context,
            } => PuzError::ParseError {
                message,
                position,
                context: format!("{context}: {existing_context}"),
            },
            other => other, // For other types, return as-is or convert to ParseError
        }
    }
}

impl fmt::Display for PuzWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PuzWarning::SkippedExtension { section, reason } => {
                write!(f, "Skipped extension section '{section}': {reason}")
            }
            PuzWarning::EncodingIssue { context, recovered } => {
                write!(
                    f,
                    "Encoding issue in {}: {}",
                    context,
                    if *recovered {
                        "recovered using fallback"
                    } else {
                        "could not recover"
                    }
                )
            }
            PuzWarning::DataRecovery { field, issue } => {
                write!(f, "Data recovery for '{field}': {issue}")
            }
            PuzWarning::ScrambledPuzzle { version } => {
                write!(f, "Puzzle is scrambled (version {version}). Solution may not be readable without descrambling.")
            }
        }
    }
}
