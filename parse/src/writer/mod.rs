use crate::{
    checksums::{self, BITMASK_DIAGRAMLESS, BITMASK_NORMAL},
    encoding::encode_nul_terminated,
    error::PuzError,
    puzzle::Puzzle,
};

mod extensions;
mod grids;
mod header;

/// Serialize a puzzle into an in-memory `.puz` byte buffer.
pub(crate) fn write_puzzle(puzzle: &Puzzle) -> Result<Vec<u8>, PuzError> {
    validate(puzzle)?;

    let info = &puzzle.info;

    // Diagramless puzzles get the diagramless bitmask and emit black squares as
    // ':' instead of '.'. The checksums below are computed over these emitted
    // bytes, so the file stays internally consistent.
    let bitmask = if info.is_diagramless {
        BITMASK_DIAGRAMLESS
    } else {
        BITMASK_NORMAL
    };

    // --- Body sections ---
    let mut header = header::serialize_header(
        info.width,
        info.height,
        // num_clues is derived from the grid-ordered clue list below.
        0,
        &info.version,
        bitmask,
    );

    let grid_bytes = grids::serialize_grids(&puzzle.grid, info.is_diagramless);
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
        bitmask,
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

/// Validate a puzzle before serializing, returning a descriptive error rather
/// than producing a corrupt file.
///
/// Checks that the grids match the declared dimensions, that the clue counts
/// match what the grid implies, and rejects scrambled puzzles (writing the
/// scramble algorithm is deferred — see the design doc).
fn validate(puzzle: &Puzzle) -> Result<(), PuzError> {
    let info = &puzzle.info;

    // Scrambled writing is not supported; reject rather than emit a file that
    // claims (via a 0x0000 tag) to be unscrambled when it isn't.
    if info.is_scrambled {
        return Err(PuzError::UnsupportedFeature {
            feature: "scrambled puzzles".to_string(),
        });
    }

    let (w, h) = (info.width as usize, info.height as usize);

    // Both grids must have `height` rows, each `width` wide.
    for (name, rows) in [
        ("solution", &puzzle.grid.solution),
        ("blank", &puzzle.grid.blank),
    ] {
        if rows.len() != h {
            return Err(PuzError::InvalidGrid {
                reason: format!("{name} grid has {} rows, expected {h} (height)", rows.len()),
            });
        }
        if let Some(bad) = rows.iter().find(|r| r.chars().count() != w) {
            return Err(PuzError::InvalidGrid {
                reason: format!(
                    "{name} grid row width {} does not match declared width {w}",
                    bad.chars().count()
                ),
            });
        }
    }

    // The number of clues provided must match what the grid geometry implies.
    let (exp_across, exp_down) = crate::grid::count_clues(&puzzle.grid.blank);
    if puzzle.clues.across.len() != exp_across {
        return Err(PuzError::InvalidClues {
            reason: format!(
                "expected {exp_across} across clues, got {}",
                puzzle.clues.across.len()
            ),
        });
    }
    if puzzle.clues.down.len() != exp_down {
        return Err(PuzError::InvalidClues {
            reason: format!(
                "expected {exp_down} down clues, got {}",
                puzzle.clues.down.len()
            ),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{error::PuzError, parse_bytes, puzzle::Puzzle, to_bytes, types::*};
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
                is_diagramless: false,
            },
            grid: Grid {
                solution: vec!["AB".into(), "CD".into()],
                blank: vec!["--".into(), "--".into()],
            },
            clues: Clues::new(
                ClueSet::new([(1, "a1"), (3, "a3")]),
                ClueSet::new([(1, "d1"), (2, "d2")]),
            ),
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
    fn test_round_trip_diagramless() {
        // Build a diagramless puzzle, write it (emits ':' + 0x0401), and parse
        // it back. The parser detects ':' and normalizes it to '.', so the
        // reparsed puzzle equals the original.
        let p = Puzzle::new(["AB.", "CDE"])
            .unwrap()
            .title("Diagramless")
            .author("Tester")
            .diagramless(true);

        let bytes = to_bytes(&p).unwrap();

        // The file's stored checksums are over the emitted ':' bytes, and the
        // verifier must checksum the same bytes, so strict validation passes.
        crate::validate_bytes(&bytes).expect("diagramless file must pass strict validation");

        let reparsed = parse_bytes(&bytes).unwrap();
        assert!(reparsed.info.is_diagramless);
        assert_eq!(reparsed.grid.solution[0], "AB."); // ':' normalized back to '.'
        assert_eq!(reparsed, p);
    }

    #[test]
    fn test_round_trip_clues_set_via_api() {
        // Build a puzzle, set specific clue text through the Clues API, write it,
        // and confirm the clues survive a parse round-trip.
        let mut p = Puzzle::new(["AB", "CD"]).unwrap();
        p.clues.across.set(1, "First across");
        p.clues.across.set(3, "Third across");
        p.clues.down.set(1, "First down");
        p.clues.down.set(2, "Second down");

        let bytes = to_bytes(&p).unwrap();
        let reparsed = parse_bytes(&bytes).unwrap();

        assert_eq!(reparsed.clues.across.get(1), Some("First across"));
        assert_eq!(reparsed.clues.across.get(3), Some("Third across"));
        assert_eq!(reparsed.clues.down.get(1), Some("First down"));
        assert_eq!(reparsed.clues.down.get(2), Some("Second down"));
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
    fn test_round_trip_windows_1252_strings() {
        let mut p = sample_puzzle();
        // café (é = 0xE9) and a right single quote (U+2019 = 0x92): both
        // representable in Windows-1252 but not ASCII.
        p.info.author = "caf\u{e9}".into();
        p.info.title = "it\u{2019}s".into();
        let bytes = to_bytes(&p).unwrap();
        assert_eq!(parse_bytes(&bytes).unwrap(), p);
    }

    #[test]
    fn test_round_trip_empty_metadata() {
        let mut p = sample_puzzle();
        p.info.title = String::new();
        p.info.author = String::new();
        p.info.copyright = String::new();
        p.info.notes = String::new();
        let bytes = to_bytes(&p).unwrap();
        assert_eq!(parse_bytes(&bytes).unwrap(), p);
    }

    #[test]
    fn test_round_trip_larger_grid_with_blocks() {
        // 5x5 with a symmetric block pattern.
        let solution = vec![
            "ABCDE".to_string(),
            "F.GH.".to_string(),
            "IJKLM".to_string(),
            ".NO.P".to_string(),
            "QRSTU".to_string(),
        ];
        let blank: Vec<String> = solution
            .iter()
            .map(|r| {
                r.chars()
                    .map(|c| if c == '.' { '.' } else { '-' })
                    .collect()
            })
            .collect();

        let mut clues = Clues::default();
        let (na, nd) = crate::grid::count_clues(&blank);
        // Fill exactly the required number of clues, numbered by position.
        let ordered_numbers = numbered_cells(&blank);
        assign_clues(&ordered_numbers, &blank, &mut clues);
        assert_eq!(clues.across.len(), na);
        assert_eq!(clues.down.len(), nd);

        let p = Puzzle {
            info: PuzzleInfo {
                title: "Blocks".into(),
                author: "A".into(),
                copyright: "(c)".into(),
                notes: String::new(),
                width: 5,
                height: 5,
                version: "1.3".into(),
                is_scrambled: false,
                is_diagramless: false,
            },
            grid: Grid { solution, blank },
            clues,
            extensions: Extensions {
                rebus: None,
                circles: None,
                given: None,
            },
        };
        let bytes = to_bytes(&p).unwrap();
        assert_eq!(parse_bytes(&bytes).unwrap(), p);
    }

    // --- test helpers for building a fully-clued larger grid ---

    fn numbered_cells(blank: &[String]) -> Vec<(usize, usize, u16)> {
        let mut out = Vec::new();
        let h = blank.len();
        let w = if h > 0 { blank[0].len() } else { 0 };
        let mut n = 1u16;
        for row in 0..h {
            for col in 0..w {
                let a = crate::grid::cell_needs_across_clue(blank, row, col);
                let d = crate::grid::cell_needs_down_clue(blank, row, col);
                if a || d {
                    out.push((row, col, n));
                    n += 1;
                }
            }
        }
        out
    }

    fn assign_clues(cells: &[(usize, usize, u16)], blank: &[String], clues: &mut Clues) {
        for &(row, col, n) in cells {
            if crate::grid::cell_needs_across_clue(blank, row, col) {
                clues.across.set(n, format!("across {n}"));
            }
            if crate::grid::cell_needs_down_clue(blank, row, col) {
                clues.down.set(n, format!("down {n}"));
            }
        }
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
    fn test_reject_scrambled_puzzle() {
        let mut p = sample_puzzle();
        p.info.is_scrambled = true;
        assert!(matches!(
            to_bytes(&p).unwrap_err(),
            PuzError::UnsupportedFeature { .. }
        ));
    }

    #[test]
    fn test_reject_grid_row_width_mismatch() {
        let mut p = sample_puzzle();
        // declared width 2, but a row is width 3
        p.grid.solution = vec!["ABC".into(), "CD".into()];
        assert!(matches!(
            to_bytes(&p).unwrap_err(),
            PuzError::InvalidGrid { .. }
        ));
    }

    #[test]
    fn test_reject_grid_row_count_mismatch() {
        let mut p = sample_puzzle();
        // declared height 2, but only 1 blank row
        p.grid.blank = vec!["--".into()];
        assert!(matches!(
            to_bytes(&p).unwrap_err(),
            PuzError::InvalidGrid { .. }
        ));
    }

    #[test]
    fn test_reject_clue_count_mismatch() {
        let mut p = sample_puzzle();
        // remove a required across clue
        p.clues.across.remove(3);
        assert!(matches!(
            to_bytes(&p).unwrap_err(),
            PuzError::InvalidClues { .. }
        ));
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
            !result
                .warnings
                .iter()
                .any(|w| matches!(w, crate::PuzWarning::ChecksumMismatch { .. })),
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
