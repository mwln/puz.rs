use super::io::read_bytes;
use crate::{
    error::PuzError,
    types::{Grid, FREE_SQUARE, TAKEN_SQUARE},
};
use std::io::{BufReader, Read};

/// Parse the solution and blank grids
pub(crate) fn parse_grids<R: Read>(
    reader: &mut BufReader<R>,
    width: u8,
    height: u8,
) -> Result<Grid, PuzError> {
    let board_size = (width as usize) * (height as usize);

    // Read solution grid - convert bytes to chars, handling non-UTF8
    let solution_bytes = read_bytes(reader, board_size)?;
    let solution_chars: String = solution_bytes.iter().map(|&b| b as char).collect();

    // Read blank grid - convert bytes to chars, handling non-UTF8
    let blank_bytes = read_bytes(reader, board_size)?;
    let blank_chars: String = blank_bytes.iter().map(|&b| b as char).collect();

    // Convert to row-based format
    let solution = string_to_grid(&solution_chars, width as usize);
    let blank = string_to_grid(&blank_chars, width as usize);

    // Validate grid consistency
    validate_grid_consistency(&solution, &blank, width, height)?;

    Ok(Grid { blank, solution })
}

/// Convert a flat string to a grid of rows
fn string_to_grid(s: &str, width: usize) -> Vec<String> {
    s.chars()
        .collect::<Vec<char>>()
        .chunks(width)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}

/// Validate that the grids are consistent
fn validate_grid_consistency(
    solution: &[String],
    blank: &[String],
    width: u8,
    height: u8,
) -> Result<(), PuzError> {
    // Check dimensions match
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
        if sol_row.len() != width as usize || blank_row.len() != width as usize {
            return Err(PuzError::InvalidGrid {
                reason: format!(
                    "Grid width mismatch at row {}: expected {}, got solution: {}, blank: {}",
                    i,
                    width,
                    sol_row.len(),
                    blank_row.len()
                ),
            });
        }

        // Validate that blocked squares match
        for (j, (sol_char, blank_char)) in sol_row.chars().zip(blank_row.chars()).enumerate() {
            if (sol_char == TAKEN_SQUARE) != (blank_char == TAKEN_SQUARE) {
                return Err(PuzError::InvalidGrid {
                    reason: format!(
                        "Grid consistency error at ({}, {}): blocked squares don't match",
                        i, j
                    ),
                });
            }
        }
    }

    Ok(())
}

/// Check if a cell needs an across clue
pub(crate) fn cell_needs_across_clue(grid: &[String], row: usize, col: usize) -> bool {
    if let Some(row_str) = grid.get(row) {
        if let Some(this_char) = row_str.chars().nth(col) {
            if this_char == FREE_SQUARE {
                // Check if next cell is also free
                if let Some(next_char) = row_str.chars().nth(col + 1) {
                    if next_char == FREE_SQUARE {
                        // This starts an across word if it's at the left edge
                        // or the previous cell is blocked
                        return col == 0 || row_str.chars().nth(col - 1) == Some(TAKEN_SQUARE);
                    }
                }
            }
        }
    }
    false
}

/// Check if a cell needs a down clue
pub(crate) fn cell_needs_down_clue(grid: &[String], row: usize, col: usize) -> bool {
    if let Some(row_str) = grid.get(row) {
        if let Some(this_char) = row_str.chars().nth(col) {
            if this_char == FREE_SQUARE {
                // Check if cell below is also free
                if let Some(next_row) = grid.get(row + 1) {
                    if let Some(next_char) = next_row.chars().nth(col) {
                        if next_char == FREE_SQUARE {
                            // This starts a down word if it's at the top edge
                            // or the cell above is blocked
                            return row == 0
                                || grid.get(row - 1).and_then(|r| r.chars().nth(col))
                                    == Some(TAKEN_SQUARE);
                        }
                    }
                }
            }
        }
    }
    false
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
        let grid = parse_grids(&mut reader, width, height).unwrap();

        assert_eq!(grid.solution.len(), 3);
        assert_eq!(grid.blank.len(), 3);
        assert_eq!(grid.solution[0], "ABC");
        assert_eq!(grid.solution[1], ".DE");
        assert_eq!(grid.solution[2], "FGH");
        assert_eq!(grid.blank[0], "---");
        assert_eq!(grid.blank[1], ".--");
        assert_eq!(grid.blank[2], "---");
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
        let grid = parse_grids(&mut reader, width, height).unwrap();

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
        let grid = parse_grids(&mut reader, width, height).unwrap();

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
        let grid = parse_grids(&mut reader, width, height).unwrap();

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
        let grid = parse_grids(&mut reader, width, height).unwrap();

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

    /// Test cell_needs_across_clue function
    /// This determines which cells start across words
    #[test]
    fn test_cell_needs_across_clue() {
        let grid = vec![
            "---".to_string(), // Row 0: across clue at (0,0)
            "...".to_string(), // Row 1: all blocked, no across clues
            "--.".to_string(), // Row 2: across clue at (2,0), not at (2,2)
        ];

        // Test start of across word - needs two consecutive free squares
        assert!(cell_needs_across_clue(&grid, 0, 0)); // --, starts word
        assert!(!cell_needs_across_clue(&grid, 0, 1)); // continues word
        assert!(!cell_needs_across_clue(&grid, 0, 2)); // continues word

        // Test blocked squares
        assert!(!cell_needs_across_clue(&grid, 1, 0)); // blocked square
        assert!(!cell_needs_across_clue(&grid, 1, 1)); // blocked square
        assert!(!cell_needs_across_clue(&grid, 1, 2)); // blocked square

        // Test second row
        assert!(cell_needs_across_clue(&grid, 2, 0)); // --, starts word
        assert!(!cell_needs_across_clue(&grid, 2, 1)); // continues word
        assert!(!cell_needs_across_clue(&grid, 2, 2)); // blocked (isolated)
    }

    /// Test cell_needs_down_clue function
    /// This determines which cells start down words
    #[test]
    fn test_cell_needs_down_clue() {
        let grid = vec![
            "-.-".to_string(), // Row 0
            "-.-".to_string(), // Row 1
            "...".to_string(), // Row 2: all blocked
        ];

        // Test start of down word - needs two consecutive free squares vertically
        assert!(cell_needs_down_clue(&grid, 0, 0)); // -/-, starts down word
        assert!(!cell_needs_down_clue(&grid, 1, 0)); // continues down word
        assert!(!cell_needs_down_clue(&grid, 2, 0)); // blocked square

        // Test blocked column
        assert!(!cell_needs_down_clue(&grid, 0, 1)); // blocked square
        assert!(!cell_needs_down_clue(&grid, 1, 1)); // blocked square
        assert!(!cell_needs_down_clue(&grid, 2, 1)); // blocked square

        // Test column with down word
        assert!(cell_needs_down_clue(&grid, 0, 2)); // -/-, starts down word
        assert!(!cell_needs_down_clue(&grid, 1, 2)); // continues down word
        assert!(!cell_needs_down_clue(&grid, 2, 2)); // blocked square
    }

    /// Test across clue detection with edge cases
    /// Boundary conditions and single-letter words
    #[test]
    fn test_across_clue_edge_cases() {
        // Single column grid
        let grid = vec!["-".to_string(), "-".to_string(), ".".to_string()];
        assert!(!cell_needs_across_clue(&grid, 0, 0)); // Can't have across word with width 1
        assert!(!cell_needs_across_clue(&grid, 1, 0)); // Can't have across word with width 1
        assert!(!cell_needs_across_clue(&grid, 2, 0)); // Blocked square

        // Grid with gaps
        let grid = vec!["-.--.".to_string()];
        assert!(!cell_needs_across_clue(&grid, 0, 0)); // - (isolated, no next free square)
        assert!(!cell_needs_across_clue(&grid, 0, 1)); // blocked
        assert!(cell_needs_across_clue(&grid, 0, 2)); // -- (two free squares, starts word)
        assert!(!cell_needs_across_clue(&grid, 0, 3)); // continues word
        assert!(!cell_needs_across_clue(&grid, 0, 4)); // blocked
    }

    /// Test down clue detection with edge cases
    /// Boundary conditions and single-letter words
    #[test]
    fn test_down_clue_edge_cases() {
        // Single row grid
        let grid = vec!["---".to_string()];
        assert!(!cell_needs_down_clue(&grid, 0, 0)); // Can't have down word with height 1
        assert!(!cell_needs_down_clue(&grid, 0, 1)); // Can't have down word with height 1
        assert!(!cell_needs_down_clue(&grid, 0, 2)); // Can't have down word with height 1

        // Grid with gaps
        let grid = vec![
            "-".to_string(),
            ".".to_string(),
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
        ];
        assert!(!cell_needs_down_clue(&grid, 0, 0)); // - (isolated, no next free square)
        assert!(!cell_needs_down_clue(&grid, 1, 0)); // blocked
        assert!(cell_needs_down_clue(&grid, 2, 0)); // -- (two free squares, starts down word)
        assert!(!cell_needs_down_clue(&grid, 3, 0)); // continues down word
        assert!(!cell_needs_down_clue(&grid, 4, 0)); // continues down word
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

    /// Test clue detection with real crossword patterns
    /// Simulates actual crossword grid layouts
    #[test]
    fn test_clue_detection_realistic_grid() {
        let grid = vec![
            "---".to_string(), // Row 0: all free squares
            "-.-".to_string(), // Row 1: free, blocked, free
            "---".to_string(), // Row 2: all free squares
        ];

        // Across clues: should be at start of each word
        assert!(cell_needs_across_clue(&grid, 0, 0)); // 3-letter word at row 0
        assert!(!cell_needs_across_clue(&grid, 0, 1)); // continues word
        assert!(!cell_needs_across_clue(&grid, 0, 2)); // continues word

        assert!(!cell_needs_across_clue(&grid, 1, 0)); // only 1 free square before block
        assert!(!cell_needs_across_clue(&grid, 1, 1)); // blocked square
        assert!(!cell_needs_across_clue(&grid, 1, 2)); // only 1 free square (isolated)

        assert!(cell_needs_across_clue(&grid, 2, 0)); // 3-letter word at row 2
        assert!(!cell_needs_across_clue(&grid, 2, 1)); // continues word
        assert!(!cell_needs_across_clue(&grid, 2, 2)); // continues word

        // Down clues: should be at start of each word
        assert!(cell_needs_down_clue(&grid, 0, 0)); // 3-letter word at col 0
        assert!(!cell_needs_down_clue(&grid, 1, 0)); // continues word
        assert!(!cell_needs_down_clue(&grid, 2, 0)); // continues word

        assert!(!cell_needs_down_clue(&grid, 0, 1)); // only free at top, blocked below
        assert!(!cell_needs_down_clue(&grid, 1, 1)); // blocked square
        assert!(!cell_needs_down_clue(&grid, 2, 1)); // isolated - no cell below

        assert!(cell_needs_down_clue(&grid, 0, 2)); // 3-letter word at col 2
        assert!(!cell_needs_down_clue(&grid, 1, 2)); // continues word
        assert!(!cell_needs_down_clue(&grid, 2, 2)); // continues word

        // Middle of row 1 is blocked
        assert!(!cell_needs_across_clue(&grid, 1, 1)); // blocked
        assert!(!cell_needs_down_clue(&grid, 1, 1)); // blocked
    }
}
