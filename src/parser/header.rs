use super::io::{decode_puz_string, read_bytes, read_u16, read_u8, skip_bytes};
use crate::error::PuzError;
use std::io::{BufReader, Read};

#[derive(Debug)]
pub(crate) struct Header {
    pub width: u8,
    pub height: u8,
    pub num_clues: u16,
    pub version: String,
    #[allow(dead_code)]
    pub bitmask: u16,
    pub is_scrambled: bool,
}

pub(crate) fn parse_header<R: Read>(reader: &mut BufReader<R>) -> Result<Header, PuzError> {
    // .puz file header format (after 12-byte magic string):
    // See: https://github.com/mwln/puz.rs/blob/main/PUZ.md
    //
    // Offset | Size | Description
    // -------|------|-------------
    // 0x0E   | 2    | CIB Checksum (skip)
    // 0x10   | 8    | Masked low/high checksums (skip)
    // 0x18   | 4    | Version string (e.g. "1.3\0")
    // 0x1C   | 2    | Reserved (skip)
    // 0x1E   | 2    | Scrambled checksum (skip)
    // 0x20   | 12   | Reserved (skip)
    // 0x2C   | 1    | Width
    // 0x2D   | 1    | Height
    // 0x2E   | 2    | Number of clues
    // 0x30   | 2    | Puzzle type bitmask
    // 0x32   | 2    | Scrambled tag

    // Skip CIB checksum (2) + masked checksums (8) = 10 bytes
    skip_bytes(reader, 10)?;

    // Read version string (4 bytes)
    let version_bytes = read_bytes(reader, 4)?;
    let version = decode_puz_string(&version_bytes)?;

    // Skip reserved (2) + scrambled checksum (2) + reserved (12) = 16 bytes
    skip_bytes(reader, 16)?;

    let width = read_u8(reader)?;
    let height = read_u8(reader)?;
    let num_clues = read_u16(reader)?;
    let bitmask = read_u16(reader)?;
    let scrambled_tag = read_u16(reader)?;

    // Validate dimensions (must be non-zero per .puz format)
    if width == 0 || height == 0 {
        return Err(PuzError::InvalidDimensions { width, height });
    }

    // Scrambled tag: 0x0000 = normal, non-zero = scrambled puzzle
    let is_scrambled = scrambled_tag != 0;

    Ok(Header {
        width,
        height,
        num_clues,
        version: version.trim_end_matches('\0').to_string(),
        bitmask,
        is_scrambled,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Create a valid header data structure for testing
    /// Layout: 10 bytes (skipped) + 4 bytes version + 16 bytes (skipped) + header fields
    fn create_header_data(
        width: u8,
        height: u8,
        num_clues: u16,
        version: &[u8; 4],
        bitmask: u16,
        scrambled_tag: u16,
    ) -> Vec<u8> {
        let mut data = Vec::new();

        // Skip bytes (10 total: CIB checksum + masked checksums)
        data.extend_from_slice(&[0; 10]);

        // Version string (4 bytes)
        data.extend_from_slice(version);

        // Skip bytes (16 total: reserved + scrambled checksum + reserved)
        data.extend_from_slice(&[0; 16]);

        // Header fields
        data.push(width);
        data.push(height);
        data.extend_from_slice(&num_clues.to_le_bytes());
        data.extend_from_slice(&bitmask.to_le_bytes());
        data.extend_from_slice(&scrambled_tag.to_le_bytes());

        data
    }

    /// Test parsing a valid header with standard dimensions
    /// This covers the most common case of parsing puzzle headers
    #[test]
    fn test_parse_header_valid() {
        let data = create_header_data(
            15,       // width
            15,       // height
            76,       // num_clues
            b"1.3\0", // version with null terminator
            0x0000,   // bitmask
            0x0000,   // not scrambled
        );

        let mut reader = BufReader::new(Cursor::new(data));
        let header = parse_header(&mut reader).unwrap();

        assert_eq!(header.width, 15);
        assert_eq!(header.height, 15);
        assert_eq!(header.num_clues, 76);
        assert_eq!(header.version, "1.3");
        assert_eq!(header.bitmask, 0x0000);
        assert!(!header.is_scrambled);
    }

    /// Test parsing header with scrambled puzzle detection
    /// Scrambled puzzles require special handling and should generate warnings
    #[test]
    fn test_parse_header_scrambled() {
        let data = create_header_data(
            21,      // width
            21,      // height
            140,     // num_clues
            b"1.2c", // version (no null terminator)
            0x0004,  // bitmask
            0x0004,  // scrambled tag (non-zero indicates scrambling)
        );

        let mut reader = BufReader::new(Cursor::new(data));
        let header = parse_header(&mut reader).unwrap();

        assert_eq!(header.width, 21);
        assert_eq!(header.height, 21);
        assert_eq!(header.num_clues, 140);
        assert_eq!(header.version, "1.2c");
        assert_eq!(header.bitmask, 0x0004);
        assert!(header.is_scrambled);
    }

    /// Test parsing header with various version string formats
    /// Version strings can have different formats and null termination
    #[test]
    fn test_parse_header_version_formats() {
        // Test with null-terminated version
        let data1 = create_header_data(15, 15, 76, b"1.4\0", 0x0000, 0x0000);
        let mut reader1 = BufReader::new(Cursor::new(data1));
        let header1 = parse_header(&mut reader1).unwrap();
        assert_eq!(header1.version, "1.4");

        // Test with non-null-terminated version
        let data2 = create_header_data(15, 15, 76, b"2.0a", 0x0000, 0x0000);
        let mut reader2 = BufReader::new(Cursor::new(data2));
        let header2 = parse_header(&mut reader2).unwrap();
        assert_eq!(header2.version, "2.0a");

        // Test with partial null termination
        let data3 = create_header_data(15, 15, 76, b"1\0\0\0", 0x0000, 0x0000);
        let mut reader3 = BufReader::new(Cursor::new(data3));
        let header3 = parse_header(&mut reader3).unwrap();
        assert_eq!(header3.version, "1");
    }

    /// Test header parsing with invalid dimensions
    /// Zero dimensions should be rejected as they indicate file corruption
    #[test]
    fn test_parse_header_invalid_dimensions_zero_width() {
        let data = create_header_data(0, 15, 76, b"1.3\0", 0x0000, 0x0000);
        let mut reader = BufReader::new(Cursor::new(data));
        let result = parse_header(&mut reader);

        assert!(result.is_err());
        if let Err(PuzError::InvalidDimensions { width, height }) = result {
            assert_eq!(width, 0);
            assert_eq!(height, 15);
        } else {
            panic!("Expected InvalidDimensions error");
        }
    }

    /// Test header parsing with zero height
    /// Zero height should also be rejected
    #[test]
    fn test_parse_header_invalid_dimensions_zero_height() {
        let data = create_header_data(15, 0, 76, b"1.3\0", 0x0000, 0x0000);
        let mut reader = BufReader::new(Cursor::new(data));
        let result = parse_header(&mut reader);

        assert!(result.is_err());
        if let Err(PuzError::InvalidDimensions { width, height }) = result {
            assert_eq!(width, 15);
            assert_eq!(height, 0);
        } else {
            panic!("Expected InvalidDimensions error");
        }
    }

    /// Test header parsing with both dimensions zero
    /// Should reject both zero dimensions
    #[test]
    fn test_parse_header_invalid_dimensions_both_zero() {
        let data = create_header_data(0, 0, 0, b"1.3\0", 0x0000, 0x0000);
        let mut reader = BufReader::new(Cursor::new(data));
        let result = parse_header(&mut reader);

        assert!(result.is_err());
        if let Err(PuzError::InvalidDimensions { width, height }) = result {
            assert_eq!(width, 0);
            assert_eq!(height, 0);
        } else {
            panic!("Expected InvalidDimensions error");
        }
    }

    /// Test parsing header with extreme but valid dimensions
    /// Large puzzles should be accepted
    #[test]
    fn test_parse_header_large_dimensions() {
        let data = create_header_data(255, 255, 30000, b"1.3\0", 0x0000, 0x0000);
        let mut reader = BufReader::new(Cursor::new(data));
        let header = parse_header(&mut reader).unwrap();

        assert_eq!(header.width, 255);
        assert_eq!(header.height, 255);
        assert_eq!(header.num_clues, 30000);
    }

    /// Test parsing header with minimal valid dimensions
    /// 1x1 puzzles should be valid (though unusual)
    #[test]
    fn test_parse_header_minimal_dimensions() {
        let data = create_header_data(1, 1, 2, b"1.3\0", 0x0000, 0x0000);
        let mut reader = BufReader::new(Cursor::new(data));
        let header = parse_header(&mut reader).unwrap();

        assert_eq!(header.width, 1);
        assert_eq!(header.height, 1);
        assert_eq!(header.num_clues, 2);
    }

    /// Test header parsing with different scrambling patterns
    /// Various non-zero scramble tags should all be detected as scrambled
    #[test]
    fn test_parse_header_scrambling_detection() {
        let scramble_values = [0x0001, 0x0004, 0x0008, 0xFFFF];

        for &scramble_tag in &scramble_values {
            let data = create_header_data(15, 15, 76, b"1.3\0", 0x0000, scramble_tag);
            let mut reader = BufReader::new(Cursor::new(data));
            let header = parse_header(&mut reader).unwrap();

            assert!(
                header.is_scrambled,
                "Failed to detect scrambling for tag 0x{:04X}",
                scramble_tag
            );
        }

        // Test that zero scramble tag is not detected as scrambled
        let data = create_header_data(15, 15, 76, b"1.3\0", 0x0000, 0x0000);
        let mut reader = BufReader::new(Cursor::new(data));
        let header = parse_header(&mut reader).unwrap();
        assert!(!header.is_scrambled);
    }

    /// Test header parsing with various bitmask values
    /// Bitmask field should be preserved even if not currently used
    #[test]
    fn test_parse_header_bitmask_values() {
        let bitmask_values = [0x0000, 0x0001, 0x0080, 0x8000, 0xFFFF];

        for &bitmask in &bitmask_values {
            let data = create_header_data(15, 15, 76, b"1.3\0", bitmask, 0x0000);
            let mut reader = BufReader::new(Cursor::new(data));
            let header = parse_header(&mut reader).unwrap();

            assert_eq!(
                header.bitmask, bitmask,
                "Bitmask not preserved: expected 0x{:04X}, got 0x{:04X}",
                bitmask, header.bitmask
            );
        }
    }

    /// Test header parsing with truncated data
    /// Should handle incomplete headers gracefully
    #[test]
    fn test_parse_header_truncated_data() {
        // Create partial header data (missing some fields)
        let mut data = Vec::new();
        data.extend_from_slice(&[0; 10]); // Skip bytes
        data.extend_from_slice(b"1.3\0"); // Version
        data.extend_from_slice(&[0; 16]); // Skip bytes
        data.push(15); // width
        data.push(15); // height
                       // Missing num_clues, bitmask, and scrambled_tag

        let mut reader = BufReader::new(Cursor::new(data));
        let result = parse_header(&mut reader);

        assert!(result.is_err());
        matches!(result.unwrap_err(), PuzError::IoError { .. });
    }

    /// Test header parsing with Windows-1252 encoded version strings
    /// Version strings might contain special characters
    #[test]
    fn test_parse_header_version_encoding() {
        // Create version with Windows-1252 characters (em dash)
        let version_bytes = [b'v', 0x97, b'1', 0x00]; // "v—1" with null terminator
        let data = create_header_data(15, 15, 76, &version_bytes, 0x0000, 0x0000);

        let mut reader = BufReader::new(Cursor::new(data));
        let header = parse_header(&mut reader).unwrap();

        // Should contain em dash character
        assert!(header.version.contains('—'));
        assert!(header.version.starts_with('v'));
        assert!(header.version.ends_with('1'));
    }
}
