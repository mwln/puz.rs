use crate::error::PuzError;
use super::io::read_string_until_nul;
use std::io::{BufReader, Read};

/// Parsed string data from the file
#[derive(Debug)]
pub(crate) struct StringData {
    pub title: String,
    pub author: String,
    pub copyright: String,
    pub notes: String,
    pub clues: Vec<String>,
}

/// Parse all string data from the .puz file
pub(crate) fn parse_strings<R: Read>(
    reader: &mut BufReader<R>,
    num_clues: u16,
) -> Result<StringData, PuzError> {
    let title = read_string_until_nul(reader)?;
    let author = read_string_until_nul(reader)?;
    let copyright = read_string_until_nul(reader)?;

    let mut clues = Vec::with_capacity(num_clues as usize);
    for i in 0..num_clues {
        match read_string_until_nul(reader) {
            Ok(clue) => clues.push(clue),
            Err(_e) => {
                return Err(PuzError::InvalidClueCount { 
                    expected: num_clues, 
                    found: i as usize 
                });
            }
        }
    }

    let notes = read_string_until_nul(reader)?;

    // Validate clue count matches header
    if clues.len() != num_clues as usize {
        return Err(PuzError::InvalidClueCount { 
            expected: num_clues, 
            found: clues.len() 
        });
    }

    Ok(StringData {
        title,
        author,
        copyright,
        notes,
        clues,
    })
}