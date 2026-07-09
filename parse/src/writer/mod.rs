use crate::{
    encoding::{encode_nul_terminated, encode_windows_1252},
    error::PuzError,
    types::Puzzle,
};

mod checksums;
mod clues;
mod grids;
mod header;

use checksums::{cksum_region, Components};

/// Standard (non-diagramless) puzzle bitmask.
const BITMASK_NORMAL: u16 = 0x0001;

/// Serialize a puzzle into an in-memory `.puz` byte buffer.
pub(crate) fn write_puzzle(puzzle: &Puzzle) -> Result<Vec<u8>, PuzError> {
    let info = &puzzle.info;

    // --- Body sections ---
    let mut header = header::serialize_header(
        info.width,
        info.height,
        // num_clues is derived from the grid-ordered clue list below.
        0,
        &info.version,
        BITMASK_NORMAL,
    );

    let grid_bytes = grids::serialize_grids(&puzzle.grid);
    let solution_bytes = grid_bytes[..grid_bytes.len() / 2].to_vec();
    let fill_bytes = grid_bytes[grid_bytes.len() / 2..].to_vec();

    let ordered_clues = clues::order_clues(&puzzle.grid.blank, &puzzle.clues)?;

    // The string section written to the file: title, author, copyright, each
    // clue, notes — all NUL-terminated.
    let mut string_bytes = Vec::new();
    string_bytes.extend(encode_nul_terminated(&info.title, "title")?);
    string_bytes.extend(encode_nul_terminated(&info.author, "author")?);
    string_bytes.extend(encode_nul_terminated(&info.copyright, "copyright")?);
    for (i, clue) in ordered_clues.iter().enumerate() {
        string_bytes.extend(encode_nul_terminated(clue, &format!("clue {i}"))?);
    }
    string_bytes.extend(encode_nul_terminated(&info.notes, "notes")?);

    // Patch num_clues now that we know the count.
    let num_clues = ordered_clues.len() as u16;
    header[0x2E..0x30].copy_from_slice(&num_clues.to_le_bytes());

    // --- Checksums ---
    // The header (CIB) checksum covers the 8 bytes at 0x2C..0x34.
    let components = Components {
        header: cksum_region(&header[0x2C..0x34], 0),
        solution: cksum_region(&solution_bytes, 0),
        fill: cksum_region(&fill_bytes, 0),
        text: text_cksum(info, &ordered_clues)?,
    };

    let text_region = text_cksum_bytes(info, &ordered_clues)?;
    let global = components.global(&solution_bytes, &fill_bytes, &text_region);

    header[0x00..0x02].copy_from_slice(&global.to_le_bytes());
    header[0x0E..0x10].copy_from_slice(&components.cib().to_le_bytes());
    header[0x10..0x18].copy_from_slice(&components.masked());

    // --- Assemble ---
    let mut out = Vec::with_capacity(header.len() + grid_bytes.len() + string_bytes.len());
    out.extend_from_slice(&header);
    out.extend_from_slice(&grid_bytes);
    out.extend_from_slice(&string_bytes);
    Ok(out)
}

/// The byte sequence the text checksum is computed over.
///
/// Per the `.puz` spec this differs from the written string section: title,
/// author, copyright, and notes are NUL-terminated but skipped when empty;
/// clues are included WITHOUT a NUL terminator, also skipped when empty. Notes
/// are only included for format version >= 1.3.
fn text_cksum_bytes(
    info: &crate::types::PuzzleInfo,
    clues: &[String],
) -> Result<Vec<u8>, PuzError> {
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

fn text_cksum(info: &crate::types::PuzzleInfo, clues: &[String]) -> Result<u16, PuzError> {
    Ok(cksum_region(&text_cksum_bytes(info, clues)?, 0))
}

/// Parse the leading `major.minor` of the version string and return true if it
/// is >= 1.3 (the version from which notes are included in the text checksum).
fn version_at_least_1_3(version: &str) -> bool {
    let mut parts = version.trim_end_matches('\0').split('.');
    let major: u32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: u32 = parts
        .next()
        .and_then(|s| s.trim_matches(|c: char| !c.is_ascii_digit()).parse().ok())
        .unwrap_or(0);
    (major, minor) >= (1, 3)
}

#[cfg(test)]
mod tests {
    use crate::{parse_bytes, to_bytes, types::*};
    use std::collections::HashMap;

    fn sample_puzzle() -> Puzzle {
        Puzzle {
            info: PuzzleInfo {
                title: "Test".into(),
                author: "Me".into(),
                copyright: "(c) 2026".into(),
                notes: String::new(),
                width: 2,
                height: 2,
                version: "1.3".into(),
                is_scrambled: false,
            },
            grid: Grid {
                solution: vec!["AB".into(), "CD".into()],
                blank: vec!["--".into(), "--".into()],
            },
            clues: {
                let mut across = HashMap::new();
                across.insert(1, "a1".into());
                across.insert(3, "a3".into());
                let mut down = HashMap::new();
                down.insert(1, "d1".into());
                down.insert(2, "d2".into());
                Clues { across, down }
            },
            extensions: Extensions {
                rebus: None,
                circles: None,
                given: None,
            },
        }
    }

    #[test]
    fn test_round_trip_basic() {
        let p = sample_puzzle();
        let bytes = to_bytes(&p).unwrap();
        let reparsed = parse_bytes(&bytes).unwrap();
        assert_eq!(reparsed, p);
    }

    #[test]
    fn test_round_trip_with_notes() {
        let mut p = sample_puzzle();
        p.info.notes = "a note".into();
        let bytes = to_bytes(&p).unwrap();
        assert_eq!(parse_bytes(&bytes).unwrap(), p);
    }

    #[test]
    fn test_written_header_checksums_are_nonzero() {
        // The whole point of the writer: real checksums, not the zeroed
        // placeholders. Confirm the global/CIB slots are populated.
        let bytes = to_bytes(&sample_puzzle()).unwrap();
        assert_ne!(&bytes[0x00..0x02], &[0, 0], "global checksum not written");
        assert_ne!(&bytes[0x0E..0x10], &[0, 0], "CIB checksum not written");
    }
}
