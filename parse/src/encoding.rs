//! Windows-1252 codec shared by the parser and writer.
//!
//! `.puz` string data is Windows-1252 encoded (modern files may be UTF-8). This
//! module keeps the decode and encode halves side by side so they stay exact
//! inverses of each other: `decode_puz_string(&encode_windows_1252(s)?) == s`
//! for any encodable `s`.

use crate::error::PuzError;

/// Decode `.puz` string bytes into a `String`.
///
/// Tries UTF-8 first (modern files), falling back to Windows-1252 for legacy
/// files.
pub(crate) fn decode_puz_string(bytes: &[u8]) -> Result<String, PuzError> {
    if let Ok(s) = std::str::from_utf8(bytes) {
        return Ok(s.to_string());
    }

    Ok(bytes.iter().map(|&b| windows_1252_to_char(b)).collect())
}

/// Encode a string as Windows-1252 bytes.
///
/// The exact inverse of [`windows_1252_to_char`]. Returns
/// [`PuzError::EncodingError`] for characters outside the Windows-1252
/// repertoire.
pub(crate) fn encode_windows_1252(s: &str, context: &str) -> Result<Vec<u8>, PuzError> {
    let mut out = Vec::with_capacity(s.len());
    for ch in s.chars() {
        let byte = char_to_windows_1252(ch).ok_or_else(|| PuzError::EncodingError {
            character: ch,
            context: context.to_string(),
        })?;
        out.push(byte);
    }
    Ok(out)
}

/// Encode a string as Windows-1252 and append a NUL terminator.
pub(crate) fn encode_nul_terminated(s: &str, context: &str) -> Result<Vec<u8>, PuzError> {
    let mut bytes = encode_windows_1252(s, context)?;
    bytes.push(0);
    Ok(bytes)
}

fn windows_1252_to_char(byte: u8) -> char {
    // Windows-1252 character mapping for bytes 128-159 that differ from ISO-8859-1.
    // Legacy .puz files often use Windows-1252 encoding for special characters.
    match byte {
        // Standard ASCII range (0-127) maps directly
        0..=127 => byte as char,
        // Windows-1252 specific mappings for 128-159 range
        128 => '€',        // Euro sign
        129 => '\u{0081}', // Unused
        130 => '‚',        // Single low-9 quotation mark
        131 => 'ƒ',        // Latin small letter f with hook
        132 => '„',        // Double low-9 quotation mark
        133 => '…',        // Horizontal ellipsis
        134 => '†',        // Dagger
        135 => '‡',        // Double dagger
        136 => 'ˆ',        // Modifier letter circumflex accent
        137 => '‰',        // Per mille sign
        138 => 'Š',        // Latin capital letter S with caron
        139 => '‹',        // Single left-pointing angle quotation mark
        140 => 'Œ',        // Latin capital ligature OE
        141 => '\u{008D}', // Unused
        142 => 'Ž',        // Latin capital letter Z with caron
        143 => '\u{008F}', // Unused
        144 => '\u{0090}', // Unused
        145 => '\u{2018}', // Left single quotation mark
        146 => '\u{2019}', // Right single quotation mark
        147 => '\u{201C}', // Left double quotation mark
        148 => '\u{201D}', // Right double quotation mark
        149 => '•',        // Bullet
        150 => '–',        // En dash
        151 => '—',        // Em dash
        152 => '˜',        // Small tilde
        153 => '™',        // Trade mark sign
        154 => 'š',        // Latin small letter s with caron
        155 => '›',        // Single right-pointing angle quotation mark
        156 => 'œ',        // Latin small ligature oe
        157 => '\u{009D}', // Unused
        158 => 'ž',        // Latin small letter z with caron
        159 => 'Ÿ',        // Latin capital letter Y with diaeresis
        // ISO-8859-1 range (160-255) is identical to Windows-1252
        160..=255 => byte as char,
    }
}

/// Map a `char` back to its Windows-1252 byte, or `None` if unrepresentable.
///
/// The exact inverse of [`windows_1252_to_char`], including the code points the
/// decoder produces for the "unused" bytes (0x81, 0x8D, 0x8F, 0x90, 0x9D), so
/// round-tripping is lossless.
fn char_to_windows_1252(ch: char) -> Option<u8> {
    match ch {
        // Standard ASCII (0x00..=0x7F) maps directly.
        '\u{0000}'..='\u{007F}' => Some(ch as u8),
        // Windows-1252 specific mappings for the 0x80..=0x9F range.
        '\u{20AC}' => Some(0x80), // Euro sign
        '\u{0081}' => Some(0x81), // Unused (decoder passthrough)
        '\u{201A}' => Some(0x82), // Single low-9 quotation mark
        '\u{0192}' => Some(0x83), // Latin small letter f with hook
        '\u{201E}' => Some(0x84), // Double low-9 quotation mark
        '\u{2026}' => Some(0x85), // Horizontal ellipsis
        '\u{2020}' => Some(0x86), // Dagger
        '\u{2021}' => Some(0x87), // Double dagger
        '\u{02C6}' => Some(0x88), // Modifier letter circumflex accent
        '\u{2030}' => Some(0x89), // Per mille sign
        '\u{0160}' => Some(0x8A), // Latin capital letter S with caron
        '\u{2039}' => Some(0x8B), // Single left-pointing angle quotation mark
        '\u{0152}' => Some(0x8C), // Latin capital ligature OE
        '\u{008D}' => Some(0x8D), // Unused (decoder passthrough)
        '\u{017D}' => Some(0x8E), // Latin capital letter Z with caron
        '\u{008F}' => Some(0x8F), // Unused (decoder passthrough)
        '\u{0090}' => Some(0x90), // Unused (decoder passthrough)
        '\u{2018}' => Some(0x91), // Left single quotation mark
        '\u{2019}' => Some(0x92), // Right single quotation mark
        '\u{201C}' => Some(0x93), // Left double quotation mark
        '\u{201D}' => Some(0x94), // Right double quotation mark
        '\u{2022}' => Some(0x95), // Bullet
        '\u{2013}' => Some(0x96), // En dash
        '\u{2014}' => Some(0x97), // Em dash
        '\u{02DC}' => Some(0x98), // Small tilde
        '\u{2122}' => Some(0x99), // Trade mark sign
        '\u{0161}' => Some(0x9A), // Latin small letter s with caron
        '\u{203A}' => Some(0x9B), // Single right-pointing angle quotation mark
        '\u{0153}' => Some(0x9C), // Latin small ligature oe
        '\u{009D}' => Some(0x9D), // Unused (decoder passthrough)
        '\u{017E}' => Some(0x9E), // Latin small letter z with caron
        '\u{0178}' => Some(0x9F), // Latin capital letter Y with diaeresis
        // ISO-8859-1 range (0xA0..=0xFF) is identical to Windows-1252.
        '\u{00A0}'..='\u{00FF}' => Some(ch as u8),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- decode ---

    #[test]
    fn test_decode_puz_string_utf8() {
        let utf8_bytes = "Hello, 世界!".as_bytes();
        let result = decode_puz_string(utf8_bytes).unwrap();
        assert_eq!(result, "Hello, 世界!");
    }

    #[test]
    fn test_decode_puz_string_windows_1252() {
        // Bytes that are invalid UTF-8 but valid Windows-1252
        let win1252_bytes = vec![
            0x93, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x94, // "Hello" with smart quotes
            0x97, // em dash
            0x85, // ellipsis
        ];

        let result = decode_puz_string(&win1252_bytes).unwrap();
        assert!(result.contains('\u{201C}')); // left double quote
        assert!(result.contains('\u{201D}')); // right double quote
        assert!(result.contains('—')); // em dash
        assert!(result.contains('…')); // ellipsis
    }

    #[test]
    fn test_windows_1252_special_chars() {
        let test_cases = vec![
            (128, '€'),
            (130, '‚'),
            (133, '…'),
            (145, '\u{2018}'),
            (146, '\u{2019}'),
            (147, '\u{201C}'),
            (148, '\u{201D}'),
            (150, '–'),
            (151, '—'),
            (153, '™'),
        ];

        for (byte_val, expected_char) in test_cases {
            let result = windows_1252_to_char(byte_val);
            assert_eq!(result, expected_char, "Failed for byte {byte_val}");
        }
    }

    #[test]
    fn test_windows_1252_ascii_passthrough() {
        for byte_val in 0..=127 {
            let result = windows_1252_to_char(byte_val);
            assert_eq!(result, byte_val as char);
        }
    }

    #[test]
    fn test_windows_1252_iso_8859_1_passthrough() {
        for byte_val in 160..=255 {
            let result = windows_1252_to_char(byte_val);
            assert_eq!(result, byte_val as char);
        }
    }

    // --- encode ---

    #[test]
    fn test_encode_ascii() {
        assert_eq!(encode_windows_1252("ABC", "title").unwrap(), b"ABC");
    }

    #[test]
    fn test_encode_high_1252_char() {
        // U+2019 RIGHT SINGLE QUOTATION MARK -> 0x92 in Windows-1252
        assert_eq!(
            encode_windows_1252("\u{2019}", "title").unwrap(),
            vec![0x92]
        );
    }

    #[test]
    fn test_encode_iso_8859_1_char() {
        // U+00E9 (é) -> 0xE9
        assert_eq!(
            encode_windows_1252("café", "author").unwrap(),
            vec![b'c', b'a', b'f', 0xE9]
        );
    }

    #[test]
    fn test_encode_unrepresentable_errors() {
        // U+2603 SNOWMAN is not in Windows-1252
        let err = encode_windows_1252("\u{2603}", "title").unwrap_err();
        assert!(matches!(err, PuzError::EncodingError { .. }));
    }

    #[test]
    fn test_encode_nul_terminated_appends_nul() {
        assert_eq!(
            encode_nul_terminated("AB", "title").unwrap(),
            vec![b'A', b'B', 0]
        );
    }

    // --- round-trip: the reason these live together ---

    #[test]
    fn test_roundtrip_all_encodable_bytes() {
        // Every byte decodes to some char; that char must encode back to the
        // same byte. This is the invariant that keeps the two tables in sync.
        for byte in 0u8..=255 {
            let ch = windows_1252_to_char(byte);
            assert_eq!(
                char_to_windows_1252(ch),
                Some(byte),
                "byte 0x{byte:02X} -> {ch:?} did not round-trip"
            );
        }
    }
}
