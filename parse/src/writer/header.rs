/// The `.puz` header is 52 bytes (0x00..0x34).
const HEADER_LEN: usize = 0x34;

/// The 12-byte magic string identifying a `.puz` file, at offset 0x02.
const MAGIC: &[u8; 12] = b"ACROSS&DOWN\0";

/// Serialize the fixed 52-byte `.puz` header.
///
/// Checksum slots (overall-file at 0x00, CIB at 0x0E, masked at 0x10..0x18) and
/// the scrambled-checksum slot are left zero; the overall/CIB/masked slots are
/// backfilled after the body is assembled (see `writer::mod`). The scrambled
/// tag (0x32) is always `0x0000` — the writer never scrambles (scrambled input
/// is rejected up front in validation).
///
/// Layout (see `parser::header` and `PUZ.md`):
/// ```text
/// 0x00  2   overall file checksum   (placeholder 0)
/// 0x02  12  magic "ACROSS&DOWN\0"
/// 0x0E  2   CIB checksum            (placeholder 0)
/// 0x10  8   masked low/high         (placeholder 0)
/// 0x18  4   version string (NUL-padded)
/// 0x1C  2   reserved
/// 0x1E  2   scrambled checksum      (0)
/// 0x20  12  reserved
/// 0x2C  1   width
/// 0x2D  1   height
/// 0x2E  2   number of clues (LE)
/// 0x30  2   bitmask (LE)
/// 0x32  2   scrambled tag (0)
/// ```
pub(crate) fn serialize_header(
    width: u8,
    height: u8,
    num_clues: u16,
    version: &str,
    bitmask: u16,
) -> Vec<u8> {
    let mut h = vec![0u8; HEADER_LEN];

    // 0x02: magic
    h[0x02..0x0E].copy_from_slice(MAGIC);

    // 0x18: version, 4 bytes, NUL-padded/truncated
    let vbytes = version.as_bytes();
    let vlen = vbytes.len().min(4);
    h[0x18..0x18 + vlen].copy_from_slice(&vbytes[..vlen]);

    // 0x2C: width, height
    h[0x2C] = width;
    h[0x2D] = height;

    // 0x2E: num clues (LE)
    h[0x2E..0x30].copy_from_slice(&num_clues.to_le_bytes());

    // 0x30: bitmask (LE)
    h[0x30..0x32].copy_from_slice(&bitmask.to_le_bytes());

    // 0x32: scrambled tag stays 0x0000.

    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_length_and_magic() {
        let h = serialize_header(15, 15, 76, "1.3", 0x0001);
        assert_eq!(h.len(), 0x34);
        assert_eq!(&h[0x02..0x0E], b"ACROSS&DOWN\0");
        assert_eq!(h[0x2C], 15); // width
        assert_eq!(h[0x2D], 15); // height
        assert_eq!(u16::from_le_bytes([h[0x2E], h[0x2F]]), 76); // num_clues
        assert_eq!(u16::from_le_bytes([h[0x30], h[0x31]]), 0x0001); // bitmask
    }

    #[test]
    fn test_header_scrambled_tag_always_zero() {
        // Writer never scrambles; tag at 0x32 is always 0x0000.
        let h = serialize_header(15, 15, 76, "1.3", 0x0001);
        assert_eq!(u16::from_le_bytes([h[0x32], h[0x33]]), 0x0000);
    }

    #[test]
    fn test_header_checksum_slots_are_placeholder_zero() {
        let h = serialize_header(15, 15, 76, "1.3", 0x0001);
        assert_eq!(&h[0x00..0x02], &[0, 0]); // overall file checksum
        assert_eq!(&h[0x0E..0x10], &[0, 0]); // CIB checksum
        assert_eq!(&h[0x10..0x18], &[0; 8]); // masked low/high
    }

    #[test]
    fn test_header_version_written_and_padded() {
        let h = serialize_header(3, 3, 4, "1.3", 0x0001);
        // "1.3" + NUL pad
        assert_eq!(&h[0x18..0x1C], b"1.3\0");
    }
}
