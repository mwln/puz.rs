use super::io::read_string_until_nul_raw;
use crate::error::PuzError;
use std::io::{BufReader, Read};

#[derive(Debug)]
pub(crate) struct StringData {
    pub(crate) title: String,
    pub(crate) author: String,
    pub(crate) copyright: String,
    pub(crate) notes: String,
    pub(crate) clues: Vec<String>,
    /// Raw pre-decode bytes of each string field, for byte-faithful checksum
    /// validation. Retaining these is effectively free — they are the buffers
    /// the reader already allocates — so we always keep them.
    pub(crate) raw: RawStrings,
}

/// Raw bytes of each string field exactly as stored in the file, used for
/// byte-faithful checksum validation.
#[derive(Debug)]
pub(crate) struct RawStrings {
    pub(crate) title: Vec<u8>,
    pub(crate) author: Vec<u8>,
    pub(crate) copyright: Vec<u8>,
    pub(crate) notes: Vec<u8>,
    pub(crate) clues: Vec<Vec<u8>>,
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

    let (title, title_raw) = read_string_until_nul_raw(reader)?;
    let (author, author_raw) = read_string_until_nul_raw(reader)?;
    let (copyright, copyright_raw) = read_string_until_nul_raw(reader)?;

    let mut clues = Vec::with_capacity(num_clues as usize);
    let mut clues_raw = Vec::with_capacity(num_clues as usize);
    for i in 0..num_clues {
        match read_string_until_nul_raw(reader) {
            Ok((clue, raw)) => {
                clues.push(clue);
                clues_raw.push(raw);
            }
            Err(_e) => {
                return Err(PuzError::InvalidClueCount {
                    expected: num_clues,
                    found: i as usize,
                });
            }
        }
    }

    let (notes, notes_raw) = read_string_until_nul_raw(reader)?;

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
        raw: RawStrings {
            title: title_raw,
            author: author_raw,
            copyright: copyright_raw,
            notes: notes_raw,
            clues: clues_raw,
        },
    })
}
