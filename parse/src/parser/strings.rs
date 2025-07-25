use super::io::read_string_until_nul;
use crate::error::PuzError;
use std::io::{BufReader, Read};

#[derive(Debug)]
pub(crate) struct StringData {
    pub title: String,
    pub author: String,
    pub copyright: String,
    pub notes: String,
    pub clues: Vec<String>,
}

pub(crate) fn parse_strings<R: Read>(
    reader: &mut BufReader<R>,
    num_clues: u16,
) -> Result<StringData, PuzError> {
    // String data format (after grid data):
    // See: https://github.com/mwln/puz.rs/blob/main/PUZ.md
    //
    // All strings are null-terminated and stored consecutively:
    // 1. Title (null-terminated)
    // 2. Author (null-terminated)
    // 3. Copyright (null-terminated)
    // 4. Clues (num_clues null-terminated strings, in reading order)
    // 5. Notes (null-terminated)

    let title = read_string_until_nul(reader)?;
    let author = read_string_until_nul(reader)?;
    let copyright = read_string_until_nul(reader)?;

    // Read clues in grid reading order (across clues first, then down clues)
    let mut clues = Vec::with_capacity(num_clues as usize);
    for i in 0..num_clues {
        match read_string_until_nul(reader) {
            Ok(clue) => clues.push(clue),
            Err(_e) => {
                return Err(PuzError::InvalidClueCount {
                    expected: num_clues,
                    found: i as usize,
                });
            }
        }
    }

    let notes = read_string_until_nul(reader)?;

    // Verify we got the expected number of clues
    if clues.len() != num_clues as usize {
        return Err(PuzError::InvalidClueCount {
            expected: num_clues,
            found: clues.len(),
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
