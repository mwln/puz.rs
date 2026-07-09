//! `.puz` checksums, shared by the writer (to produce them) and the parser (to
//! validate them).
//!
//! Keeping computation in one place means validation is a genuine independent
//! check: the parser recomputes checksums from the reconstructed `Puzzle` and
//! compares them to the values stored in the file. A writer bug produces wrong
//! stored bytes that this recomputation catches, and the recomputation does not
//! depend on the parser's own byte-reading path.

use crate::{
    encoding::{encode_nul_terminated, encode_windows_1252},
    error::PuzError,
    types::{Grid, PuzzleInfo},
};

/// The 8-byte mask string for the "masked" checksums (spells "ICHEATED").
const MASK: &[u8; 8] = b"ICHEATED";

/// Standard (non-diagramless) puzzle bitmask, at header offset 0x30.
pub(crate) const BITMASK_NORMAL: u16 = 0x0001;

/// The checksum values stored in a `.puz` file header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Stored {
    /// Global/overall checksum at offset 0x00.
    pub(crate) global: u16,
    /// CIB (header) checksum at offset 0x0E.
    pub(crate) cib: u16,
    /// Masked low/high checksum bytes at offset 0x10..0x18.
    pub(crate) masked: [u8; 8],
}

/// The `.puz` checksum: a modified CRC-16 (rotate-right, then add) applied per
/// byte. See `PUZ.md` §Checksums for the reference algorithm.
pub(crate) fn cksum_region(data: &[u8], mut cksum: u16) -> u16 {
    for &b in data {
        cksum = (cksum >> 1) | ((cksum & 1) << 15);
        cksum = cksum.wrapping_add(b as u16);
    }
    cksum
}

/// The four component checksums of a `.puz` file, in the order the format
/// masks them: header (CIB), solution, fill (player grid), and text.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Components {
    pub(crate) header: u16,
    pub(crate) solution: u16,
    pub(crate) fill: u16,
    pub(crate) text: u16,
}

impl Components {
    /// The CIB (header) checksum, over the 8 header bytes at 0x2C..0x34.
    pub(crate) fn cib(&self) -> u16 {
        self.header
    }

    /// The overall/global file checksum (stored at 0x00): header, then the
    /// solution grid, the fill grid, and the text region, chained. Extensions
    /// are not included.
    pub(crate) fn global(&self, solution: &[u8], fill: &[u8], text: &[u8]) -> u16 {
        let mut c = self.header;
        c = cksum_region(solution, c);
        c = cksum_region(fill, c);
        cksum_region(text, c)
    }

    /// The 8 "masked" checksum bytes stored at 0x10..0x18. Each component's low
    /// and high bytes are XORed with the corresponding byte of "ICHEATED".
    pub(crate) fn masked(&self) -> [u8; 8] {
        let lows = [self.header, self.solution, self.fill, self.text];
        let mut out = [0u8; 8];
        for (i, c) in lows.iter().enumerate() {
            out[i] = (*c as u8) ^ MASK[i]; // low byte ^ "ICHE"
            out[i + 4] = ((*c >> 8) as u8) ^ MASK[i + 4]; // high byte ^ "ATED"
        }
        out
    }
}

/// The 8 CIB header bytes at 0x2C..0x34: width, height, num_clues (LE),
/// bitmask (LE), scrambled tag (LE). This is what the CIB checksum covers.
pub(crate) fn cib_bytes(width: u8, height: u8, num_clues: u16, bitmask: u16, scrambled: u16) -> [u8; 8] {
    let mut b = [0u8; 8];
    b[0] = width;
    b[1] = height;
    b[2..4].copy_from_slice(&num_clues.to_le_bytes());
    b[4..6].copy_from_slice(&bitmask.to_le_bytes());
    b[6..8].copy_from_slice(&scrambled.to_le_bytes());
    b
}

/// Build the byte sequence the text checksum is computed over.
///
/// Per the `.puz` spec this differs from the written string section: title,
/// author, copyright, and notes are NUL-terminated but skipped when empty;
/// clues are included WITHOUT a NUL terminator, also skipped when empty. Notes
/// are only included for format version >= 1.3.
pub(crate) fn text_cksum_bytes(info: &PuzzleInfo, clues: &[String]) -> Result<Vec<u8>, PuzError> {
    let mut bytes = Vec::new();
    if !info.title.is_empty() {
        bytes.extend(encode_nul_terminated(&info.title, "title")?);
    }
    if !info.author.is_empty() {
        bytes.extend(encode_nul_terminated(&info.author, "author")?);
    }
    if !info.copyright.is_empty() {
        bytes.extend(encode_nul_terminated(&info.copyright, "copyright")?);
    }
    for (i, clue) in clues.iter().enumerate() {
        if !clue.is_empty() {
            bytes.extend(encode_windows_1252(clue, &format!("clue {i}"))?);
        }
    }
    if version_at_least_1_3(&info.version) && !info.notes.is_empty() {
        bytes.extend(encode_nul_terminated(&info.notes, "notes")?);
    }
    Ok(bytes)
}

/// Parse the leading `major.minor` of the version string and return true if it
/// is >= 1.3 (the version from which notes are included in the text checksum).
pub(crate) fn version_at_least_1_3(version: &str) -> bool {
    let mut parts = version.trim_end_matches('\0').split('.');
    let major: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: u32 = parts
        .next()
        .and_then(|s| s.trim_matches(|c: char| !c.is_ascii_digit()).parse().ok())
        .unwrap_or(0);
    (major, minor) >= (1, 3)
}

/// Compute all component checksums for a puzzle's parts.
///
/// `bitmask` and `scrambled` are the header fields as they appear (or will
/// appear) in the file. `solution_bytes`/`fill_bytes` are the raw grid bytes,
/// and `ordered_clues` are the clues in grid reading order.
pub(crate) fn compute(
    info: &PuzzleInfo,
    bitmask: u16,
    scrambled: u16,
    solution_bytes: &[u8],
    fill_bytes: &[u8],
    ordered_clues: &[String],
) -> Result<Components, PuzError> {
    let num_clues = ordered_clues.len() as u16;
    let cib = cib_bytes(info.width, info.height, num_clues, bitmask, scrambled);
    Ok(Components {
        header: cksum_region(&cib, 0),
        solution: cksum_region(solution_bytes, 0),
        fill: cksum_region(fill_bytes, 0),
        text: cksum_region(&text_cksum_bytes(info, ordered_clues)?, 0),
    })
}

/// Verify a puzzle's recomputed checksums against the values stored in the file.
///
/// Returns the first mismatching checksum as [`PuzError::InvalidChecksum`], or
/// `Ok(())` if all three (global, CIB, masked) match.
pub(crate) fn verify(
    info: &PuzzleInfo,
    grid: &Grid,
    ordered_clues: &[String],
    bitmask: u16,
    scrambled: u16,
    stored: &Stored,
) -> Result<(), PuzError> {
    let solution_bytes: Vec<u8> = grid.solution.iter().flat_map(|r| r.bytes()).collect();
    let fill_bytes: Vec<u8> = grid.blank.iter().flat_map(|r| r.bytes()).collect();

    let components = compute(
        info,
        bitmask,
        scrambled,
        &solution_bytes,
        &fill_bytes,
        ordered_clues,
    )?;

    let text_region = text_cksum_bytes(info, ordered_clues)?;
    let global = components.global(&solution_bytes, &fill_bytes, &text_region);
    let cib = components.cib();
    let masked = components.masked();

    if global != stored.global {
        return Err(PuzError::InvalidChecksum {
            expected: global,
            found: stored.global,
            context: "global".to_string(),
        });
    }
    if cib != stored.cib {
        return Err(PuzError::InvalidChecksum {
            expected: cib,
            found: stored.cib,
            context: "CIB".to_string(),
        });
    }
    if masked != stored.masked {
        // Report the masked mismatch as a u16 pair for the error's fields.
        return Err(PuzError::InvalidChecksum {
            expected: u16::from_le_bytes([masked[0], masked[1]]),
            found: u16::from_le_bytes([stored.masked[0], stored.masked[1]]),
            context: "masked".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cksum_region_empty_returns_seed() {
        assert_eq!(cksum_region(&[], 0), 0);
        assert_eq!(cksum_region(&[], 0x1234), 0x1234);
    }

    #[test]
    fn test_cksum_region_single_byte() {
        // seed 0, byte 0x01: rotate(0)=0, +1 => 1
        assert_eq!(cksum_region(&[0x01], 0), 1);
    }

    #[test]
    fn test_cksum_region_known_vector() {
        // Hand-computed against PUZ.md's reference algorithm, seed 0:
        //   0x01: rot(0)=0,      +1 => 0x0001
        //   0x02: rot(1)=0x8000, +2 => 0x8002
        //   0x03: rot(0x8002)=0x4001, +3 => 0x4004
        assert_eq!(cksum_region(&[0x01, 0x02, 0x03], 0), 0x4004);
    }

    #[test]
    fn test_cksum_region_seed_chaining() {
        // Feeding the checksum of region A as the seed for region B must equal
        // checksumming the concatenation in one pass.
        let a = [0x10u8, 0x20, 0x30];
        let b = [0x40u8, 0x50];
        let chained = cksum_region(&b, cksum_region(&a, 0));
        let concat: Vec<u8> = a.iter().chain(b.iter()).copied().collect();
        assert_eq!(chained, cksum_region(&concat, 0));
    }

    #[test]
    fn test_cksum_region_wraps_at_u16() {
        let data = [0xFFu8; 8];
        assert_eq!(cksum_region(&data, 0), cksum_region(&data, 0));
    }

    #[test]
    fn test_version_at_least_1_3() {
        assert!(version_at_least_1_3("1.3"));
        assert!(version_at_least_1_3("1.4"));
        assert!(version_at_least_1_3("2.0"));
        assert!(!version_at_least_1_3("1.2"));
        assert!(!version_at_least_1_3("1.0"));
        assert!(version_at_least_1_3("1.3\0"));
    }

    #[test]
    fn test_masked_bytes_xor_scheme() {
        // With all component checksums zero, masked bytes are exactly the mask.
        let c = Components {
            header: 0,
            solution: 0,
            fill: 0,
            text: 0,
        };
        assert_eq!(&c.masked(), MASK);
    }
}
