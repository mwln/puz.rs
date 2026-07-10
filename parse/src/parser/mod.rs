use crate::{
    error::{ParseResult, PuzError, PuzWarning},
    types::*,
};
use std::io::{BufReader, Read};

mod clues;
mod extensions;
mod grids;
mod header;
mod io;
mod strings;
mod validation;

use clues::process_clues;
use extensions::parse_extensions_with_recovery;
use grids::parse_grids;
use header::parse_header;
use io::{read_remaining_data, validate_file_magic};
use strings::parse_strings;
pub(crate) use strings::RawStrings;
use validation::validate_puzzle;

pub(crate) fn parse_puzzle<R: Read>(reader: R) -> Result<ParseResult<Puzzle>, PuzError> {
    parse_puzzle_inner(reader, false)
}

/// Parse and require all stored checksums to match; a mismatch is a hard error.
pub(crate) fn parse_puzzle_strict<R: Read>(reader: R) -> Result<ParseResult<Puzzle>, PuzError> {
    parse_puzzle_inner(reader, true)
}

fn parse_puzzle_inner<R: Read>(reader: R, strict: bool) -> Result<ParseResult<Puzzle>, PuzError> {
    let mut buf_reader = BufReader::new(reader);
    let mut warnings = Vec::new();

    let global_cksum = validate_file_magic(&mut buf_reader)?;
    let header = parse_header(&mut buf_reader)?;

    if header.is_scrambled {
        warnings.push(PuzWarning::ScrambledPuzzle {
            version: header.version.clone(),
        });
    }

    // Capture stored checksums before consuming the header fields we need.
    let stored = crate::checksums::Stored {
        global: global_cksum,
        cib: header.cib_cksum,
        masked: header.masked_cksum,
    };
    let bitmask = header.bitmask;
    let scrambled_tag = header.scrambled_tag;

    let grids = parse_grids(&mut buf_reader, header.width, header.height)?;

    let strings = parse_strings(&mut buf_reader, header.num_clues)?;

    let extra_data = read_remaining_data(&mut buf_reader)?;
    let (extensions, ext_warnings) =
        parse_extensions_with_recovery(&extra_data, header.width, header.height)?;
    warnings.extend(ext_warnings);

    let clues = process_clues(&grids.blank, &strings.clues)?;

    let raw_strings = strings.raw;
    let puzzle = Puzzle {
        info: PuzzleInfo {
            title: strings.title,
            author: strings.author,
            copyright: strings.copyright,
            notes: strings.notes,
            width: header.width,
            height: header.height,
            version: header.version,
            is_scrambled: header.is_scrambled,
        },
        grid: grids,
        clues,
        extensions,
    };

    validate_puzzle(&puzzle)?;

    // Warn about non-standard solution characters that no rebus entry explains.
    warnings.extend(check_unbacked_grid_chars(&puzzle));

    // Checksum validation: reconstruct the clue order and recompute checksums
    // independently of the writer, then compare with the stored values. When
    // available, the raw string bytes make the text checksum byte-faithful.
    let ordered_clues = crate::grid::order_clues(&puzzle.grid.blank, &puzzle.clues)?;
    match crate::checksums::verify(
        &puzzle.info,
        &puzzle.grid,
        &ordered_clues,
        &raw_strings,
        bitmask,
        scrambled_tag,
        &stored,
    ) {
        Ok(()) => {}
        Err(e) => {
            if strict {
                return Err(e);
            }
            if let PuzError::InvalidChecksum {
                expected,
                found,
                context,
            } = &e
            {
                warnings.push(PuzWarning::ChecksumMismatch {
                    context: context.clone(),
                    expected: *expected,
                    found: *found,
                });
            }
        }
    }

    Ok(ParseResult::with_warnings(puzzle, warnings))
}

/// Warn about solution cells that hold a non-standard character with no rebus
/// entry backing them.
///
/// A `.puz` file places no constraint on cell bytes, and rebus puzzles put
/// arbitrary glyphs (`#`, `*`, high bytes, ...) in solution cells. Those are
/// legitimate when the GRBS grid marks the cell as a rebus. A non-standard char
/// with no such backing is unusual and may indicate corruption, so we surface a
/// warning without rejecting the file.
fn check_unbacked_grid_chars(puzzle: &Puzzle) -> Vec<PuzWarning> {
    let mut warnings = Vec::new();
    let rebus_grid = puzzle.extensions.rebus.as_ref().map(|r| &r.grid);

    for (row, line) in puzzle.grid.solution.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if crate::grid::is_standard_cell_char(ch) {
                continue;
            }
            let backed = rebus_grid
                .and_then(|g| g.get(row))
                .and_then(|r| r.get(col))
                .is_some_and(|&key| key != 0);
            if !backed {
                warnings.push(PuzWarning::UnbackedGridChar {
                    character: ch,
                    row,
                    col,
                });
            }
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn puzzle_with_solution(solution: Vec<String>, rebus: Option<Rebus>) -> Puzzle {
        let width = solution[0].chars().count() as u8;
        let height = solution.len() as u8;
        let blank = solution
            .iter()
            .map(|row| {
                row.chars()
                    .map(|c| if c == '.' { '.' } else { '-' })
                    .collect()
            })
            .collect();
        Puzzle {
            info: PuzzleInfo {
                title: String::new(),
                author: String::new(),
                copyright: String::new(),
                notes: String::new(),
                width,
                height,
                version: "1.3".to_string(),
                is_scrambled: false,
            },
            grid: Grid { blank, solution },
            clues: Clues {
                across: HashMap::new(),
                down: HashMap::new(),
            },
            extensions: Extensions {
                rebus,
                circles: None,
                given: None,
            },
        }
    }

    #[test]
    fn test_plain_grid_produces_no_warning() {
        let p = puzzle_with_solution(vec!["AB".into(), "CD".into()], None);
        assert!(check_unbacked_grid_chars(&p).is_empty());
    }

    #[test]
    fn test_unbacked_marker_char_warns() {
        // '#' at (0,0) with no rebus data.
        let p = puzzle_with_solution(vec!["#B".into(), "CD".into()], None);
        let warnings = check_unbacked_grid_chars(&p);
        assert_eq!(warnings.len(), 1);
        assert!(matches!(
            warnings[0],
            PuzWarning::UnbackedGridChar {
                character: '#',
                row: 0,
                col: 0
            }
        ));
    }

    #[test]
    fn test_marker_char_backed_by_rebus_is_silent() {
        // '#' at (0,0), and the rebus grid marks that cell.
        let mut table = HashMap::new();
        table.insert(1u8, "HASH".to_string());
        let rebus = Rebus {
            grid: vec![vec![1, 0], vec![0, 0]],
            table,
        };
        let p = puzzle_with_solution(vec!["#B".into(), "CD".into()], Some(rebus));
        assert!(check_unbacked_grid_chars(&p).is_empty());
    }

    #[test]
    fn test_high_byte_char_backed_by_rebus_is_silent() {
        // 'Â' (0xC2) at (0,0) backed by a rebus entry.
        let mut table = HashMap::new();
        table.insert(1u8, "CENT".to_string());
        let rebus = Rebus {
            grid: vec![vec![1, 0], vec![0, 0]],
            table,
        };
        let p = puzzle_with_solution(vec!["\u{00C2}B".into(), "CD".into()], Some(rebus));
        assert!(check_unbacked_grid_chars(&p).is_empty());
    }
}
