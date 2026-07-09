use crate::error::PuzError;

/// Encode a string as Windows-1252 bytes.
///
/// This is the exact inverse of `parser::io::windows_1252_to_char`, so that
/// `decode(encode(s)) == s` for any encodable `s`. Returns
/// [`PuzError::EncodingError`] for characters outside the Windows-1252 repertoire.
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

/// Map a `char` back to its Windows-1252 byte, or `None` if unrepresentable.
///
/// Mirrors `parser::io::windows_1252_to_char` exactly, including the code points
/// the decoder produces for the "unused" bytes (0x81, 0x8D, 0x8F, 0x90, 0x9D),
/// so round-tripping is lossless.
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

    #[test]
    fn test_encode_ascii() {
        assert_eq!(encode_windows_1252("ABC", "title").unwrap(), b"ABC");
    }

    #[test]
    fn test_encode_high_1252_char() {
        // U+2019 RIGHT SINGLE QUOTATION MARK -> 0x92 in Windows-1252
        assert_eq!(encode_windows_1252("\u{2019}", "title").unwrap(), vec![0x92]);
    }

    #[test]
    fn test_encode_iso_8859_1_char() {
        // U+00E9 (é) -> 0xE9
        assert_eq!(encode_windows_1252("café", "author").unwrap(), vec![b'c', b'a', b'f', 0xE9]);
    }

    #[test]
    fn test_encode_unrepresentable_errors() {
        // U+2603 SNOWMAN is not in Windows-1252
        let err = encode_windows_1252("\u{2603}", "title").unwrap_err();
        assert!(matches!(err, PuzError::EncodingError { .. }));
    }

    #[test]
    fn test_encode_nul_terminated_appends_nul() {
        assert_eq!(encode_nul_terminated("AB", "title").unwrap(), vec![b'A', b'B', 0]);
    }
}
