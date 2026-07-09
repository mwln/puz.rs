use crate::{
    checksums::{self, BITMASK_NORMAL},
    encoding::encode_nul_terminated,
    error::PuzError,
    types::Puzzle,
};

mod extensions;
mod grids;
mod header;

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

    let ordered_clues = crate::grid::order_clues(&puzzle.grid.blank, &puzzle.clues)?;

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

    // --- Checksums (shared with parser validation) ---
    // Scrambled tag is always 0x0000 (writer never scrambles; validation in
    // Task 9 rejects scrambled input).
    let components = checksums::compute(
        info,
        BITMASK_NORMAL,
        0x0000,
        &solution_bytes,
        &fill_bytes,
        &ordered_clues,
    )?;

    let text_region = checksums::text_cksum_bytes(info, &ordered_clues)?;
    let global = components.global(&solution_bytes, &fill_bytes, &text_region);

    header[0x00..0x02].copy_from_slice(&global.to_le_bytes());
    header[0x0E..0x10].copy_from_slice(&components.cib().to_le_bytes());
    header[0x10..0x18].copy_from_slice(&components.masked());

    // Extension sections (GRBS/RTBL/GEXT) follow the strings. They carry their
    // own per-section checksums and are NOT part of the header checksums.
    let extension_bytes =
        extensions::serialize_extensions(&puzzle.extensions, info.width, info.height)?;

    // --- Assemble ---
    let mut out = Vec::with_capacity(
        header.len() + grid_bytes.len() + string_bytes.len() + extension_bytes.len(),
    );
    out.extend_from_slice(&header);
    out.extend_from_slice(&grid_bytes);
    out.extend_from_slice(&string_bytes);
    out.extend_from_slice(&extension_bytes);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use crate::{error::PuzError, parse_bytes, to_bytes, types::*};
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
    fn test_round_trip_with_rebus() {
        let mut p = sample_puzzle();
        let mut table = HashMap::new();
        table.insert(1u8, "HEART".to_string());
        p.extensions.rebus = Some(Rebus {
            // rebus key 1 at cell (0,0); solution letter there is 'A'
            grid: vec![vec![1, 0], vec![0, 0]],
            table,
        });
        let bytes = to_bytes(&p).unwrap();
        assert_eq!(parse_bytes(&bytes).unwrap(), p);
    }

    #[test]
    fn test_round_trip_with_circles() {
        let mut p = sample_puzzle();
        p.extensions.circles = Some(vec![vec![true, false], vec![false, true]]);
        let bytes = to_bytes(&p).unwrap();
        assert_eq!(parse_bytes(&bytes).unwrap(), p);
    }

    #[test]
    fn test_round_trip_with_given() {
        let mut p = sample_puzzle();
        p.extensions.given = Some(vec![vec![false, true], vec![true, false]]);
        let bytes = to_bytes(&p).unwrap();
        assert_eq!(parse_bytes(&bytes).unwrap(), p);
    }

    #[test]
    fn test_round_trip_with_circles_and_given() {
        let mut p = sample_puzzle();
        p.extensions.circles = Some(vec![vec![true, false], vec![false, false]]);
        p.extensions.given = Some(vec![vec![false, false], vec![false, true]]);
        let bytes = to_bytes(&p).unwrap();
        assert_eq!(parse_bytes(&bytes).unwrap(), p);
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

    #[test]
    fn test_written_file_passes_strict_validation() {
        // The writer's checksums must satisfy the parser's independent
        // recomputation. This is the non-circular integrity check: the parser
        // recomputes from puzzle data, not from how the writer built the bytes.
        let bytes = to_bytes(&sample_puzzle()).unwrap();
        crate::validate_bytes(&bytes).expect("written file should validate");
        // lenient parse should emit no checksum-mismatch warning
        let result = crate::parse(&bytes[..]).unwrap();
        assert!(
            !result.warnings.iter().any(|w| matches!(
                w,
                crate::PuzWarning::ChecksumMismatch { .. }
            )),
            "unexpected checksum warning on valid file"
        );
    }

    #[test]
    fn test_corrupted_global_checksum_is_detected() {
        // Flip the stored global checksum; strict validation must reject it and
        // lenient parse must warn. Proves validation isn't a no-op.
        let mut bytes = to_bytes(&sample_puzzle()).unwrap();
        bytes[0x00] ^= 0xFF;

        let err = crate::validate_bytes(&bytes).unwrap_err();
        assert!(matches!(err, PuzError::InvalidChecksum { .. }));

        let result = crate::parse(&bytes[..]).unwrap();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| matches!(w, crate::PuzWarning::ChecksumMismatch { .. })),
            "expected a checksum-mismatch warning on corrupted file"
        );
    }

    #[test]
    fn test_corrupted_cib_checksum_is_detected() {
        let mut bytes = to_bytes(&sample_puzzle()).unwrap();
        bytes[0x0E] ^= 0xFF;
        assert!(matches!(
            crate::validate_bytes(&bytes).unwrap_err(),
            PuzError::InvalidChecksum { .. }
        ));
    }

    #[test]
    fn test_corrupted_masked_checksum_is_detected() {
        let mut bytes = to_bytes(&sample_puzzle()).unwrap();
        bytes[0x10] ^= 0xFF;
        assert!(matches!(
            crate::validate_bytes(&bytes).unwrap_err(),
            PuzError::InvalidChecksum { .. }
        ));
    }
}
