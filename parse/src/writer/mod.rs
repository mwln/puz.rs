use crate::{error::PuzError, types::Puzzle};

mod checksums;
mod strings;

/// Serialize a puzzle into an in-memory `.puz` byte buffer.
pub(crate) fn write_puzzle(_puzzle: &Puzzle) -> Result<Vec<u8>, PuzError> {
    todo!("implemented in later tasks")
}
