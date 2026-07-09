//! Grid geometry shared by the parser and writer.
//!
//! The `.puz` format doesn't store square numbers or word boundaries — they're
//! derived from the grid layout. This module is the single source of truth for
//! that derivation (which cells start across/down words, the block/empty square
//! sentinels, and how many clues a grid implies), so the read and write paths
//! can never disagree about numbering.

/// Sentinel for an empty (unfilled) square in the blank grid.
pub(crate) const FREE_SQUARE: char = '-';

/// Sentinel for a blocked/black square.
pub(crate) const TAKEN_SQUARE: char = '.';

/// Returns `true` when a cell is playable, i.e. it holds either an empty
/// square (`FREE_SQUARE`) or letter/number content, as opposed to a blocked
/// square (`TAKEN_SQUARE`).
///
/// `None` (a cell outside the grid) is treated as not playable.
pub(crate) fn is_playable_square(cell: Option<char>) -> bool {
    matches!(cell, Some(c) if c == FREE_SQUARE || c.is_ascii_alphanumeric())
}

/// Returns `true` when the cell at `(row, col)` starts an across word: it is
/// playable, the cell to its right is playable, and it is either at the left
/// edge or preceded by a blocked square.
pub(crate) fn cell_needs_across_clue(grid: &[String], row: usize, col: usize) -> bool {
    if let Some(row_str) = grid.get(row) {
        if is_playable_square(row_str.chars().nth(col))
            && is_playable_square(row_str.chars().nth(col + 1))
        {
            return col == 0 || row_str.chars().nth(col - 1) == Some(TAKEN_SQUARE);
        }
    }
    false
}

/// Returns `true` when the cell at `(row, col)` starts a down word: it is
/// playable, the cell below it is playable, and it is either at the top edge or
/// preceded above by a blocked square.
pub(crate) fn cell_needs_down_clue(grid: &[String], row: usize, col: usize) -> bool {
    if let Some(row_str) = grid.get(row) {
        if is_playable_square(row_str.chars().nth(col))
            && is_playable_square(grid.get(row + 1).and_then(|r| r.chars().nth(col)))
        {
            return row == 0
                || grid.get(row - 1).and_then(|r| r.chars().nth(col)) == Some(TAKEN_SQUARE);
        }
    }
    false
}

/// Count the number of across and down clues a blank grid implies, by walking
/// cells in reading order.
///
/// Returns `(across_count, down_count)`. Shared by parser validation and the
/// writer's clue ordering so both agree on how many clues a grid requires.
pub(crate) fn count_clues(grid: &[String]) -> (usize, usize) {
    let mut across_count = 0;
    let mut down_count = 0;

    let height = grid.len();
    let width = if height > 0 { grid[0].len() } else { 0 };

    for row in 0..height {
        for col in 0..width {
            if cell_needs_across_clue(grid, row, col) {
                across_count += 1;
            }
            if cell_needs_down_clue(grid, row, col) {
                down_count += 1;
            }
        }
    }

    (across_count, down_count)
}

/// Returns `true` when `c` is a character allowed in puzzle string content.
pub(crate) fn is_valid_puzzle_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '\'' | '&' | '.' | '!' | '?')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_playable_square() {
        assert!(is_playable_square(Some(FREE_SQUARE)));
        assert!(is_playable_square(Some('A')));
        assert!(is_playable_square(Some('z')));
        assert!(is_playable_square(Some('0')));
        assert!(is_playable_square(Some('9')));
        assert!(!is_playable_square(Some(TAKEN_SQUARE)));
        assert!(!is_playable_square(Some(' ')));
        assert!(!is_playable_square(Some('#')));
        assert!(!is_playable_square(None));
    }

    #[test]
    fn test_cell_needs_across_clue() {
        let grid = vec![
            "---".to_string(),
            "...".to_string(),
            "--.".to_string(),
        ];

        assert!(cell_needs_across_clue(&grid, 0, 0));
        assert!(!cell_needs_across_clue(&grid, 0, 1));
        assert!(!cell_needs_across_clue(&grid, 0, 2));

        assert!(!cell_needs_across_clue(&grid, 1, 0));
        assert!(!cell_needs_across_clue(&grid, 1, 1));
        assert!(!cell_needs_across_clue(&grid, 1, 2));

        assert!(cell_needs_across_clue(&grid, 2, 0));
        assert!(!cell_needs_across_clue(&grid, 2, 1));
        assert!(!cell_needs_across_clue(&grid, 2, 2));
    }

    #[test]
    fn test_cell_needs_down_clue() {
        let grid = vec![
            "-.-".to_string(),
            "-.-".to_string(),
            "...".to_string(),
        ];

        assert!(cell_needs_down_clue(&grid, 0, 0));
        assert!(!cell_needs_down_clue(&grid, 1, 0));
        assert!(!cell_needs_down_clue(&grid, 2, 0));

        assert!(!cell_needs_down_clue(&grid, 0, 1));
        assert!(!cell_needs_down_clue(&grid, 1, 1));
        assert!(!cell_needs_down_clue(&grid, 2, 1));

        assert!(cell_needs_down_clue(&grid, 0, 2));
        assert!(!cell_needs_down_clue(&grid, 1, 2));
        assert!(!cell_needs_down_clue(&grid, 2, 2));
    }

    #[test]
    fn test_across_clue_edge_cases() {
        let grid = vec!["-".to_string(), "-".to_string(), ".".to_string()];
        assert!(!cell_needs_across_clue(&grid, 0, 0));
        assert!(!cell_needs_across_clue(&grid, 1, 0));
        assert!(!cell_needs_across_clue(&grid, 2, 0));

        let grid = vec!["-.--.".to_string()];
        assert!(!cell_needs_across_clue(&grid, 0, 0));
        assert!(!cell_needs_across_clue(&grid, 0, 1));
        assert!(cell_needs_across_clue(&grid, 0, 2));
        assert!(!cell_needs_across_clue(&grid, 0, 3));
        assert!(!cell_needs_across_clue(&grid, 0, 4));

        let grid = vec!["-.-A.".to_string()];
        assert!(!cell_needs_across_clue(&grid, 0, 0));
        assert!(!cell_needs_across_clue(&grid, 0, 1));
        assert!(cell_needs_across_clue(&grid, 0, 2));
        assert!(!cell_needs_across_clue(&grid, 0, 3));
        assert!(!cell_needs_across_clue(&grid, 0, 4));
    }

    #[test]
    fn test_down_clue_edge_cases() {
        let grid = vec!["---".to_string()];
        assert!(!cell_needs_down_clue(&grid, 0, 0));
        assert!(!cell_needs_down_clue(&grid, 0, 1));
        assert!(!cell_needs_down_clue(&grid, 0, 2));

        let grid = vec![
            "-".to_string(),
            ".".to_string(),
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
        ];
        assert!(!cell_needs_down_clue(&grid, 0, 0));
        assert!(!cell_needs_down_clue(&grid, 1, 0));
        assert!(cell_needs_down_clue(&grid, 2, 0));
        assert!(!cell_needs_down_clue(&grid, 3, 0));
        assert!(!cell_needs_down_clue(&grid, 4, 0));

        let grid = vec![
            "-".to_string(),
            ".".to_string(),
            "A".to_string(),
            "B".to_string(),
            "-".to_string(),
        ];
        assert!(!cell_needs_down_clue(&grid, 0, 0));
        assert!(!cell_needs_down_clue(&grid, 1, 0));
        assert!(cell_needs_down_clue(&grid, 2, 0));
        assert!(!cell_needs_down_clue(&grid, 3, 0));
        assert!(!cell_needs_down_clue(&grid, 4, 0));
    }

    #[test]
    fn test_clue_detection_realistic_grid() {
        let grid = vec![
            "---".to_string(),
            "-.-".to_string(),
            "---".to_string(),
        ];

        assert!(cell_needs_across_clue(&grid, 0, 0));
        assert!(!cell_needs_across_clue(&grid, 0, 1));
        assert!(!cell_needs_across_clue(&grid, 0, 2));
        assert!(!cell_needs_across_clue(&grid, 1, 0));
        assert!(!cell_needs_across_clue(&grid, 1, 1));
        assert!(!cell_needs_across_clue(&grid, 1, 2));
        assert!(cell_needs_across_clue(&grid, 2, 0));
        assert!(!cell_needs_across_clue(&grid, 2, 1));
        assert!(!cell_needs_across_clue(&grid, 2, 2));

        assert!(cell_needs_down_clue(&grid, 0, 0));
        assert!(!cell_needs_down_clue(&grid, 1, 0));
        assert!(!cell_needs_down_clue(&grid, 2, 0));
        assert!(!cell_needs_down_clue(&grid, 0, 1));
        assert!(!cell_needs_down_clue(&grid, 1, 1));
        assert!(!cell_needs_down_clue(&grid, 2, 1));
        assert!(cell_needs_down_clue(&grid, 0, 2));
        assert!(!cell_needs_down_clue(&grid, 1, 2));
        assert!(!cell_needs_down_clue(&grid, 2, 2));
    }

    #[test]
    fn test_count_clues() {
        // 3x3 open grid with a single center block.
        let grid = vec![
            "---".to_string(),
            "-.-".to_string(),
            "---".to_string(),
        ];
        // Across starts: (0,0), (2,0). Down starts: (0,0), (0,2).
        assert_eq!(count_clues(&grid), (2, 2));
    }

    #[test]
    fn test_count_clues_complex() {
        let grid = vec![
            "--.".to_string(),
            "...".to_string(),
            ".--".to_string(),
        ];
        let (across_count, down_count) = count_clues(&grid);
        assert_eq!(across_count, 2);
        assert!(down_count <= 3);
    }

    #[test]
    fn test_count_clues_empty() {
        let grid: Vec<String> = vec![];
        assert_eq!(count_clues(&grid), (0, 0));
    }

    #[test]
    fn test_count_clues_single_cell() {
        // A single cell can't form words, so no clues expected.
        assert_eq!(count_clues(&["-".to_string()]), (0, 0));
    }

    #[test]
    fn test_is_valid_puzzle_char() {
        for c in ['A', 'Z', 'a', 'z', '0', '9', ' ', '-', '\'', '&', '.', '!', '?'] {
            assert!(is_valid_puzzle_char(c), "expected {c:?} to be valid");
        }
        for c in [
            '\0', '\n', '\t', '@', '#', '$', '%', '^', '*', '(', ')',
        ] {
            assert!(!is_valid_puzzle_char(c), "expected {c:?} to be invalid");
        }
    }

    // --- cross-function invariants, only expressible now that the geometry
    // helpers and count_clues live in one module ---

    #[test]
    fn test_count_clues_agrees_with_cell_walk() {
        // count_clues must equal a manual walk using the same predicates, on a
        // variety of grids. This guards against the two ever drifting apart.
        let grids = [
            vec!["---".to_string(), "-.-".to_string(), "---".to_string()],
            vec!["--.".to_string(), "...".to_string(), ".--".to_string()],
            vec!["-----".to_string(), "-.-.-".to_string(), "-----".to_string()],
            vec!["-".to_string()],
            vec![],
        ];

        for grid in grids {
            let height = grid.len();
            let width = if height > 0 { grid[0].len() } else { 0 };
            let mut across = 0;
            let mut down = 0;
            for row in 0..height {
                for col in 0..width {
                    if cell_needs_across_clue(&grid, row, col) {
                        across += 1;
                    }
                    if cell_needs_down_clue(&grid, row, col) {
                        down += 1;
                    }
                }
            }
            assert_eq!(count_clues(&grid), (across, down), "mismatch on {grid:?}");
        }
    }

    #[test]
    fn test_square_sentinels_and_playability_are_consistent() {
        // The two sentinels are distinct, and playability treats them as the
        // spec requires: FREE is playable, TAKEN is not.
        assert_ne!(FREE_SQUARE, TAKEN_SQUARE);
        assert!(is_playable_square(Some(FREE_SQUARE)));
        assert!(!is_playable_square(Some(TAKEN_SQUARE)));
    }
}
