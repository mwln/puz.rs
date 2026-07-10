use crate::error::PuzError;
use byteorder::{ByteOrder, LittleEndian};
use std::io::{BufReader, Read};

pub(crate) fn validate_file_magic<R: Read>(reader: &mut BufReader<R>) -> Result<u16, PuzError> {
    // .puz file format starts with:
    // See: https://github.com/mwln/puz.rs/blob/main/PUZ.md
    //
    // Offset | Size | Description
    // -------|------|-------------
    // 0x00   | 2    | Overall file checksum
    // 0x02   | 12   | Magic string "ACROSS&DOWN\0"

    // Capture the 2-byte overall (global) file checksum for later validation.
    let global_cksum = read_u16(reader)?;

    // Read and validate the 12-byte magic string
    let mut magic = [0u8; 12];
    reader.read_exact(&mut magic)?;

    let expected_magic = b"ACROSS&DOWN\0";
    if magic != *expected_magic {
        return Err(PuzError::InvalidMagic {
            found: magic.to_vec(),
        });
    }

    Ok(global_cksum)
}

pub(crate) fn skip_bytes<R: Read>(reader: &mut BufReader<R>, count: usize) -> Result<(), PuzError> {
    let mut buffer = vec![0u8; count];
    reader.read_exact(&mut buffer)?;
    Ok(())
}

pub(crate) fn read_u8<R: Read>(reader: &mut BufReader<R>) -> Result<u8, PuzError> {
    let mut buffer = [0u8; 1];
    reader.read_exact(&mut buffer)?;
    Ok(buffer[0])
}

pub(crate) fn read_u16<R: Read>(reader: &mut BufReader<R>) -> Result<u16, PuzError> {
    let mut buffer = [0u8; 2];
    reader.read_exact(&mut buffer)?;
    Ok(LittleEndian::read_u16(&buffer))
}

pub(crate) fn read_bytes<R: Read>(
    reader: &mut BufReader<R>,
    count: usize,
) -> Result<Vec<u8>, PuzError> {
    let mut buffer = vec![0u8; count];
    reader.read_exact(&mut buffer)?;
    Ok(buffer)
}

/// Read a NUL-terminated string, returning both the decoded `String` and the
/// raw bytes as they appeared in the file (excluding the terminator).
///
/// The raw bytes are needed for byte-faithful checksum validation: decoding and
/// re-encoding is not always a round-trip (e.g. a character stored as UTF-8 that
/// is also representable in Windows-1252), so checksums must be computed over
/// the original bytes.
pub(crate) fn read_string_until_nul_raw<R: Read>(
    reader: &mut BufReader<R>,
) -> Result<(String, Vec<u8>), PuzError> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        if byte[0] == 0 {
            break;
        }
        bytes.push(byte[0]);
    }
    let decoded = crate::encoding::decode_puz_string(&bytes)?;
    Ok((decoded, bytes))
}

pub(crate) fn read_remaining_data<R: Read>(reader: &mut BufReader<R>) -> Result<Vec<u8>, PuzError> {
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;
    Ok(data)
}

pub(crate) fn find_section(data: &[u8], section_name: &str) -> Result<Option<Vec<u8>>, PuzError> {
    // Extension sections format (after main puzzle data):
    // See: https://github.com/mwln/puz.rs/blob/main/PUZ.md
    //
    // Each section has the structure:
    // - Section name (4 bytes, e.g. "GRBS", "RTBL", "GEXT")
    // - Data length (2 bytes, little-endian)
    // - Checksum (2 bytes)
    // - Section data (variable length)

    if let Some(index) = data
        .windows(section_name.len())
        .position(|window| window == section_name.as_bytes())
    {
        let length_start = index + section_name.len();
        if length_start + 2 <= data.len() {
            let data_length =
                LittleEndian::read_u16(&data[length_start..length_start + 2]) as usize;
            let data_start = length_start + 4; // skip length (2) + checksum (2)
            let data_end = data_start + data_length;
            if data_end <= data.len() {
                return Ok(Some(data[data_start..data_end].to_vec()));
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Test that validate_file_magic correctly validates the ACROSS&DOWN magic string
    /// This test ensures the parser can identify valid .puz files and reject invalid ones
    #[test]
    fn test_validate_file_magic_valid() {
        // Create a valid .puz file header with checksum (2 bytes) + magic (12 bytes)
        let mut data = vec![0xAB, 0xCD]; // Dummy checksum
        data.extend_from_slice(b"ACROSS&DOWN\0");

        let mut reader = BufReader::new(Cursor::new(data));
        assert!(validate_file_magic(&mut reader).is_ok());
    }

    /// Test that validate_file_magic rejects files with invalid magic strings
    /// This prevents parsing non-.puz files that might cause undefined behavior
    #[test]
    fn test_validate_file_magic_invalid() {
        // Create invalid magic string (exactly 12 bytes)
        let mut data = vec![0xAB, 0xCD]; // Dummy checksum
        data.extend_from_slice(b"INVALID_MGIC"); // 12 bytes exactly

        let mut reader = BufReader::new(Cursor::new(data));
        let result = validate_file_magic(&mut reader);

        assert!(result.is_err());
        if let Err(PuzError::InvalidMagic { found }) = result {
            assert_eq!(found, b"INVALID_MGIC".to_vec());
        } else {
            panic!("Expected InvalidMagic error");
        }
    }

    /// Test that validate_file_magic handles incomplete data gracefully
    /// This ensures we don't panic on truncated files
    #[test]
    fn test_validate_file_magic_truncated() {
        // Too short - only 5 bytes instead of required 14
        let data = vec![0xAB, 0xCD, 0x41, 0x43, 0x52];

        let mut reader = BufReader::new(Cursor::new(data));
        let result = validate_file_magic(&mut reader);

        assert!(result.is_err());
        // Should get an IO error due to incomplete read
        matches!(result.unwrap_err(), PuzError::IoError { .. });
    }

    /// Test skip_bytes function with various byte counts
    /// This utility is used throughout the parser to skip over reserved/unused fields
    #[test]
    fn test_skip_bytes() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut reader = BufReader::new(Cursor::new(data));

        // Skip first 3 bytes
        assert!(skip_bytes(&mut reader, 3).is_ok());

        // Next byte should be 4
        assert_eq!(read_u8(&mut reader).unwrap(), 4);

        // Skip 2 more bytes
        assert!(skip_bytes(&mut reader, 2).is_ok());

        // Next byte should be 7
        assert_eq!(read_u8(&mut reader).unwrap(), 7);
    }

    /// Test skip_bytes with insufficient data
    /// Ensures we handle truncated files gracefully
    #[test]
    fn test_skip_bytes_insufficient_data() {
        let data = vec![1, 2, 3];
        let mut reader = BufReader::new(Cursor::new(data));

        // Try to skip more bytes than available
        let result = skip_bytes(&mut reader, 5);
        assert!(result.is_err());
        matches!(result.unwrap_err(), PuzError::IoError { .. });
    }

    /// Test reading single bytes
    /// Basic building block for binary parsing
    #[test]
    fn test_read_u8() {
        let data = vec![42, 255, 0, 128];
        let mut reader = BufReader::new(Cursor::new(data));

        assert_eq!(read_u8(&mut reader).unwrap(), 42);
        assert_eq!(read_u8(&mut reader).unwrap(), 255);
        assert_eq!(read_u8(&mut reader).unwrap(), 0);
        assert_eq!(read_u8(&mut reader).unwrap(), 128);
    }

    /// Test reading 16-bit little-endian values
    /// Critical for parsing .puz file dimensions, clue counts, etc.
    #[test]
    fn test_read_u16_little_endian() {
        // Little-endian: 0x1234 is stored as 0x34, 0x12
        let data = vec![0x34, 0x12, 0xFF, 0x00, 0x00, 0x80];
        let mut reader = BufReader::new(Cursor::new(data));

        assert_eq!(read_u16(&mut reader).unwrap(), 0x1234);
        assert_eq!(read_u16(&mut reader).unwrap(), 0x00FF);
        assert_eq!(read_u16(&mut reader).unwrap(), 0x8000);
    }

    /// Test reading byte arrays of specified lengths
    /// Used for reading grid data, version strings, etc.
    #[test]
    fn test_read_bytes() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let mut reader = BufReader::new(Cursor::new(data));

        let result = read_bytes(&mut reader, 3).unwrap();
        assert_eq!(result, vec![1, 2, 3]);

        let result = read_bytes(&mut reader, 2).unwrap();
        assert_eq!(result, vec![4, 5]);

        let result = read_bytes(&mut reader, 3).unwrap();
        assert_eq!(result, vec![6, 7, 8]);
    }

    /// Test reading null-terminated strings
    /// Standard format for .puz file strings (title, author, clues, etc.)
    #[test]
    fn test_read_string_until_nul() {
        // "Hello" followed by null terminator, then more data
        let data = vec![72, 101, 108, 108, 111, 0, 87, 111, 114, 108, 100, 0];
        let mut reader = BufReader::new(Cursor::new(data));

        let (decoded, raw) = read_string_until_nul_raw(&mut reader).unwrap();
        assert_eq!(decoded, "Hello");
        assert_eq!(raw, b"Hello");

        let (decoded, _) = read_string_until_nul_raw(&mut reader).unwrap();
        assert_eq!(decoded, "World");
    }

    /// Test reading null-terminated string with no terminator
    /// Should handle malformed files gracefully
    #[test]
    fn test_read_string_until_nul_no_terminator() {
        let data = vec![72, 101, 108, 108, 111]; // "Hello" with no null terminator
        let mut reader = BufReader::new(Cursor::new(data));

        let result = read_string_until_nul_raw(&mut reader);
        assert!(result.is_err());
        matches!(result.unwrap_err(), PuzError::IoError { .. });
    }

    /// Test finding sections in extension data
    /// .puz files use named sections for rebus, circles, etc.
    #[test]
    fn test_find_section_exists() {
        // Create mock extension data with GRBS section
        let mut data = Vec::new();
        data.extend_from_slice(b"GRBS"); // Section name
        data.extend_from_slice(&[0x04, 0x00]); // Length: 4 bytes (little-endian)
        data.extend_from_slice(&[0xAB, 0xCD]); // Checksum (dummy)
        data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]); // Section data

        let result = find_section(&data, "GRBS").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec![0x01, 0x02, 0x03, 0x04]);
    }

    /// Test finding non-existent sections
    /// Should return None without error
    #[test]
    fn test_find_section_not_found() {
        let data = b"SOME_OTHER_DATA".to_vec();
        let result = find_section(&data, "GRBS").unwrap();
        assert!(result.is_none());
    }

    /// Test finding section with insufficient data
    /// Should handle malformed extension sections gracefully
    #[test]
    fn test_find_section_insufficient_data() {
        // Section name exists but not enough data for length field
        let data = b"GRBS\x04".to_vec(); // Missing length byte and all data
        let result = find_section(&data, "GRBS").unwrap();
        assert!(result.is_none());
    }

    /// Test finding section with truncated data
    /// Length field indicates more data than available
    #[test]
    fn test_find_section_truncated_data() {
        let mut data = Vec::new();
        data.extend_from_slice(b"GRBS"); // Section name
        data.extend_from_slice(&[0x10, 0x00]); // Length: 16 bytes
        data.extend_from_slice(&[0xAB, 0xCD]); // Checksum
        data.extend_from_slice(&[0x01, 0x02]); // Only 2 bytes instead of 16

        let result = find_section(&data, "GRBS").unwrap();
        assert!(result.is_none());
    }

    /// Test reading all remaining data from reader
    /// Used for reading extension sections at end of file
    #[test]
    fn test_read_remaining_data() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let mut reader = BufReader::new(Cursor::new(data.clone()));

        // Read some data first
        let _ = read_u8(&mut reader).unwrap();
        let _ = read_u16(&mut reader).unwrap();

        // Read remaining
        let remaining = read_remaining_data(&mut reader).unwrap();
        assert_eq!(remaining, vec![4, 5, 6, 7, 8]);
    }

    /// Test reading remaining data from empty reader
    /// Should return empty vector, not error
    #[test]
    fn test_read_remaining_data_empty() {
        let data = Vec::new();
        let mut reader = BufReader::new(Cursor::new(data));

        let remaining = read_remaining_data(&mut reader).unwrap();
        assert!(remaining.is_empty());
    }
}
