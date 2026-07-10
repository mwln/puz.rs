use super::io::read_bytes;
use crate::{error::PuzError, grid::TAKEN_SQUARE, types::Grid};
use std::io::{BufReader, Read};

pub(crate) fn parse_grids<R: Read>(
    reader: &mut BufReader<R>,
    width: u8,
    height: u8,
) -> Result<(Grid, bool), PuzError> {
    // Grid data format (after header and before strings):
    // See: https://github.com/mwln/puz.rs/blob/main/PUZ.md
    //
    // The grids are stored as two consecutive byte arrays:
    // 1. Solution grid: width * height bytes (actual puzzle answers)
    // 2. Blank grid: width * height bytes (puzzle state for solver)
    //
    // Each byte represents one cell:
    // - '.' (0x2E) = black/blocked square
    // - '-' (0x2D) = empty square (in blank grid)
    // - A-Z, 0-9 = letter/number content
    //
    // Diagramless puzzles store black squares as ':' (0x3A) instead of '.'.
    // These are detected here and normalized to '.' so downstream code, which
    // keys on '.', is unchanged.

    let board_size = (width as usize) * (height as usize);

    // Read solution grid (width * height bytes)
    let solution_bytes = read_bytes(reader, board_size)?;

    // Read blank grid (width * height bytes)
    let blank_bytes = read_bytes(reader, board_size)?;

    // Diagramless puzzles store black squares as ':' (0x3A) instead of '.'.
    // Detect by content: ':' appears only in diagramless grids (verified across
    // the corpus), and it is the authoritative signal (the header bitmask is
    // sometimes wrong). Normalize ':' to '.' so downstream code, which keys on
    // '.', is unchanged.
    let is_diagramless = solution_bytes.contains(&b':') || blank_bytes.contains(&b':');

    let normalize = |b: u8| -> char {
        if b == b':' {
            '.'
        } else {
            b as char
        }
    };
    let solution_chars: String = solution_bytes.iter().map(|&b| normalize(b)).collect();
    let blank_chars: String = blank_bytes.iter().map(|&b| normalize(b)).collect();

    // Convert flat strings to row-based grids
    let solution = string_to_grid(&solution_chars, width as usize);
    let blank = string_to_grid(&blank_chars, width as usize);

    // Ensure blocked squares match between grids
    validate_grid_consistency(&solution, &blank, width, height)?;

    Ok((Grid { blank, solution }, is_diagramless))
}

fn string_to_grid(s: &str, width: usize) -> Vec<String> {
    s.chars()
        .collect::<Vec<char>>()
        .chunks(width)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}

fn validate_grid_consistency(
    solution: &[String],
    blank: &[String],
    width: u8,
    height: u8,
) -> Result<(), PuzError> {
    if solution.len() != height as usize || blank.len() != height as usize {
        return Err(PuzError::InvalidGrid {
            reason: format!(
                "Grid height mismatch: expected {}, got solution: {}, blank: {}",
                height,
                solution.len(),
                blank.len()
            ),
        });
    }

    for (i, (sol_row, blank_row)) in solution.iter().zip(blank.iter()).enumerate() {
        // Each grid cell is one character (a byte may decode to a multi-byte
        // char, e.g. a high byte used as a rebus marker), so compare cell
        // counts via `chars().count()`, not byte length.
        let sol_cells = sol_row.chars().count();
        let blank_cells = blank_row.chars().count();
        if sol_cells != width as usize || blank_cells != width as usize {
            return Err(PuzError::InvalidGrid {
                reason: format!(
                    "Grid width mismatch at row {i}: expected {width}, got solution: {sol_cells}, blank: {blank_cells}"
                ),
            });
        }

        for (j, (sol_char, blank_char)) in sol_row.chars().zip(blank_row.chars()).enumerate() {
            if (sol_char == TAKEN_SQUARE) != (blank_char == TAKEN_SQUARE) {
                return Err(PuzError::InvalidGrid {
                    reason: format!(
                        "Grid consistency error at ({i}, {j}): blocked squares don't match"
                    ),
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Test parsing valid grids with standard layout
    /// This covers the most common case of rectangular crossword grids
    #[test]
    fn test_parse_grids_valid() {
        let width = 3u8;
        let height = 3u8;

        // Create solution grid: simple 3x3 with some black squares (9 bytes)
        let solution_data = b"ABC.DEFGH";
        // Create blank grid: same pattern but with dashes for empty squares (9 bytes, blacks must match)
        let blank_data = b"---.-----";

        let mut data = Vec::new();
        data.extend_from_slice(solution_data);
        data.extend_from_slice(blank_data);

        let mut reader = BufReader::new(Cursor::new(data));
        let (grid, _) = parse_grids(&mut reader, width, height).unwrap();

        assert_eq!(grid.solution.len(), 3);
        assert_eq!(grid.blank.len(), 3);
        assert_eq!(grid.solution[0], "ABC");
        assert_eq!(grid.solution[1], ".DE");
        assert_eq!(grid.solution[2], "FGH");
        assert_eq!(grid.blank[0], "---");
        assert_eq!(grid.blank[1], ".--");
        assert_eq!(grid.blank[2], "---");
    }

    /// Test parsing a grid whose solution contains a non-ASCII cell byte.
    ///
    /// Some puzzles use high bytes (e.g. 0xC2) as rebus/special-cell markers.
    /// Each cell is one byte -> one char, so a row must validate by cell count,
    /// not UTF-8 byte length. Byte 0xC2 decodes to 'Â' (2 UTF-8 bytes), so a
    /// byte-length check would see width 3 for a 2-cell row and wrongly reject.
    #[test]
    fn test_parse_grids_non_ascii_cell() {
        let width = 2u8;
        let height = 2u8;

        // Row 0 solution: [0xC2, 'B'] = one high-byte cell + one letter.
        let solution_data = [0xC2, b'B', b'C', b'D'];
        let blank_data = *b"----";

        let mut data = Vec::new();
        data.extend_from_slice(&solution_data);
        data.extend_from_slice(&blank_data);

        let mut reader = BufReader::new(Cursor::new(data));
        let (grid, _) = parse_grids(&mut reader, width, height).unwrap();

        assert_eq!(grid.solution[0].chars().count(), 2);
        assert_eq!(grid.solution[0], "\u{00C2}B");
        assert_eq!(grid.blank[0], "--");
    }

    /// A grid containing ':' is diagramless; every ':' becomes '.'.
    #[test]
    fn test_parse_grids_diagramless_normalizes_colon() {
        let width = 2u8;
        let height = 2u8;
        let solution_data = b":BC:"; // colons at (0,0) and (1,1)
        let blank_data = b":--:";

        let mut data = Vec::new();
        data.extend_from_slice(solution_data);
        data.extend_from_slice(blank_data);

        let mut reader = BufReader::new(Cursor::new(data));
        let (grid, is_diagramless) = parse_grids(&mut reader, width, height).unwrap();

        assert!(is_diagramless);
        assert_eq!(grid.solution, vec![".B".to_string(), "C.".to_string()]);
        assert_eq!(grid.blank, vec![".-".to_string(), "-.".to_string()]);
    }

    /// No ':' anywhere -> normal puzzle, unchanged.
    #[test]
    fn test_parse_grids_no_colon_is_not_diagramless() {
        let mut data = Vec::new();
        data.extend_from_slice(b"AB.D"); // solution with a normal '.' black square
        data.extend_from_slice(b"--.-"); // blank
        let mut reader = BufReader::new(Cursor::new(data));
        let (grid, is_diagramless) = parse_grids(&mut reader, 2, 2).unwrap();

        assert!(!is_diagramless);
        assert_eq!(grid.solution, vec!["AB".to_string(), ".D".to_string()]);
    }

    /// Test parsing grids with all black squares
    /// Edge case where entire puzzle is blocked
    #[test]
    fn test_parse_grids_all_black() {
        let width = 2u8;
        let height = 2u8;

        let solution_data = b"....";
        let blank_data = b"....";

        let mut data = Vec::new();
        data.extend_from_slice(solution_data);
        data.extend_from_slice(blank_data);

        let mut reader = BufReader::new(Cursor::new(data));
        let (grid, _) = parse_grids(&mut reader, width, height).unwrap();

        assert_eq!(grid.solution, vec!["..".to_string(), "..".to_string()]);
        assert_eq!(grid.blank, vec!["..".to_string(), "..".to_string()]);
    }

    /// Test parsing grids with all free squares
    /// Edge case where no squares are blocked
    #[test]
    fn test_parse_grids_all_free() {
        let width = 2u8;
        let height = 2u8;

        let solution_data = b"ABCD";
        let blank_data = b"----";

        let mut data = Vec::new();
        data.extend_from_slice(solution_data);
        data.extend_from_slice(blank_data);

        let mut reader = BufReader::new(Cursor::new(data));
        let (grid, _) = parse_grids(&mut reader, width, height).unwrap();

        assert_eq!(grid.solution, vec!["AB".to_string(), "CD".to_string()]);
        assert_eq!(grid.blank, vec!["--".to_string(), "--".to_string()]);
    }

    /// Test parsing 1x1 grid (minimal valid case)
    /// Ensures single-cell puzzles work
    #[test]
    fn test_parse_grids_single_cell() {
        let width = 1u8;
        let height = 1u8;

        let solution_data = b"A";
        let blank_data = b"-";

        let mut data = Vec::new();
        data.extend_from_slice(solution_data);
        data.extend_from_slice(blank_data);

        let mut reader = BufReader::new(Cursor::new(data));
        let (grid, _) = parse_grids(&mut reader, width, height).unwrap();

        assert_eq!(grid.solution, vec!["A".to_string()]);
        assert_eq!(grid.blank, vec!["-".to_string()]);
    }

    /// Test parsing large grid
    /// Ensures performance with typical newspaper puzzle sizes
    #[test]
    fn test_parse_grids_large() {
        let width = 15u8;
        let height = 15u8;
        let board_size = (width as usize) * (height as usize);

        // Create alternating pattern
        let mut solution_data = Vec::new();
        let mut blank_data = Vec::new();

        for i in 0..board_size {
            if i % 2 == 0 {
                solution_data.push(b'A');
                blank_data.push(b'-');
            } else {
                solution_data.push(b'.');
                blank_data.push(b'.');
            }
        }

        let mut data = Vec::new();
        data.extend(solution_data);
        data.extend(blank_data);

        let mut reader = BufReader::new(Cursor::new(data));
        let (grid, _) = parse_grids(&mut reader, width, height).unwrap();

        assert_eq!(grid.solution.len(), 15);
        assert_eq!(grid.blank.len(), 15);
        assert_eq!(grid.solution[0].len(), 15);
        assert_eq!(grid.blank[0].len(), 15);
    }

    /// Test grid parsing with insufficient data
    /// Should handle truncated grid data gracefully
    #[test]
    fn test_parse_grids_insufficient_data() {
        let width = 3u8;
        let height = 3u8;

        // Only provide solution data, missing blank data
        let solution_data = b"ABC.DEFGH";

        let mut reader = BufReader::new(Cursor::new(solution_data));
        let result = parse_grids(&mut reader, width, height);

        assert!(result.is_err());
        matches!(result.unwrap_err(), PuzError::IoError { .. });
    }

    /// Test grid parsing with consistency validation failure
    /// Blank and solution grids must have matching blocked squares
    #[test]
    fn test_parse_grids_consistency_failure() {
        let width = 2u8;
        let height = 2u8;

        // Solution has black square at (0,1), blank doesn't
        let solution_data = b"A.BC";
        let blank_data = b"--B-"; // Inconsistent: should be "-.B-"

        let mut data = Vec::new();
        data.extend_from_slice(solution_data);
        data.extend_from_slice(blank_data);

        let mut reader = BufReader::new(Cursor::new(data));
        let result = parse_grids(&mut reader, width, height);

        assert!(result.is_err());
        if let Err(PuzError::InvalidGrid { reason }) = result {
            assert!(reason.contains("consistency error"));
        } else {
            panic!("Expected InvalidGrid error with consistency message");
        }
    }

    /// Test string_to_grid function with various inputs
    /// This is the core grid transformation logic
    #[test]
    fn test_string_to_grid() {
        // Test normal case
        let result = string_to_grid("ABCDEF", 3);
        assert_eq!(result, vec!["ABC".to_string(), "DEF".to_string()]);

        // Test single row
        let result = string_to_grid("ABC", 3);
        assert_eq!(result, vec!["ABC".to_string()]);

        // Test single column
        let result = string_to_grid("ABC", 1);
        assert_eq!(
            result,
            vec!["A".to_string(), "B".to_string(), "C".to_string()]
        );

        // Test empty string
        let result = string_to_grid("", 1);
        assert_eq!(result, Vec::<String>::new());
    }

    /// Test grid validation with mismatched dimensions
    /// Ensures proper error handling for corrupted data
    #[test]
    fn test_validate_grid_consistency_dimension_mismatch() {
        let solution = vec!["ABC".to_string(), "DEF".to_string()]; // 2 rows
        let blank = vec!["---".to_string()]; // 1 row

        let result = validate_grid_consistency(&solution, &blank, 3, 2);
        assert!(result.is_err());
        if let Err(PuzError::InvalidGrid { reason }) = result {
            assert!(reason.contains("height mismatch"));
        } else {
            panic!("Expected InvalidGrid error with height mismatch");
        }
    }

    /// Test grid validation with mismatched row widths
    /// Ensures proper error handling for malformed rows
    #[test]
    fn test_validate_grid_consistency_width_mismatch() {
        let solution = vec!["ABC".to_string(), "DE".to_string()]; // Second row too short
        let blank = vec!["---".to_string(), "--".to_string()];

        let result = validate_grid_consistency(&solution, &blank, 3, 2);
        assert!(result.is_err());
        if let Err(PuzError::InvalidGrid { reason }) = result {
            assert!(reason.contains("width mismatch"));
        } else {
            panic!("Expected InvalidGrid error with width mismatch");
        }
    }
}
