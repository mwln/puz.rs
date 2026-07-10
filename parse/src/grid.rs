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

/// The character at column `col` of a grid row.
///
/// Uses O(1) byte indexing for the common all-ASCII case (`.puz` grids contain
/// only `.`, `-`, and alphanumerics) and falls back to a `chars()` scan for the
/// rare non-ASCII row, so it is behavior-identical to `row.chars().nth(col)`
/// while avoiding that method's O(col) walk on every lookup.
fn cell_char(row: &str, col: usize) -> Option<char> {
    if row.is_ascii() {
        row.as_bytes().get(col).map(|&b| b as char)
    } else {
        row.chars().nth(col)
    }
}

/// Returns `true` when the cell at `(row, col)` starts an across word: it is
/// playable, the cell to its right is playable, and it is either at the left
/// edge or preceded by a blocked square.
pub(crate) fn cell_needs_across_clue(grid: &[String], row: usize, col: usize) -> bool {
    if let Some(row_str) = grid.get(row) {
        if is_playable_square(cell_char(row_str, col))
            && is_playable_square(cell_char(row_str, col + 1))
        {
            return col == 0 || cell_char(row_str, col - 1) == Some(TAKEN_SQUARE);
        }
    }
    false
}

/// Returns `true` when the cell at `(row, col)` starts a down word: it is
/// playable, the cell below it is playable, and it is either at the top edge or
/// preceded above by a blocked square.
pub(crate) fn cell_needs_down_clue(grid: &[String], row: usize, col: usize) -> bool {
    if let Some(row_str) = grid.get(row) {
        if is_playable_square(cell_char(row_str, col))
            && is_playable_square(grid.get(row + 1).and_then(|r| cell_char(r, col)))
        {
            return row == 0
                || grid.get(row - 1).and_then(|r| cell_char(r, col)) == Some(TAKEN_SQUARE);
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
    let width = if height > 0 {
        grid[0].chars().count()
    } else {
        0
    };

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

/// Flatten the `Clues` maps into the canonical `.puz` reading order.
///
/// Walk the blank grid row-major; at each numbered cell emit its across clue
/// (if it starts an across word) then its down clue (if it starts a down word),
/// incrementing the number once per numbered cell. This is the single source of
/// truth for clue order — used by the writer to serialize clues and by parser
/// validation to reconstruct the text-checksum region.
pub(crate) fn order_clues(
    blank_grid: &[String],
    clues: &crate::types::Clues,
) -> Result<Vec<String>, crate::error::PuzError> {
    let mut ordered = Vec::new();
    let height = blank_grid.len();
    let width = if height > 0 {
        blank_grid[0].chars().count()
    } else {
        0
    };
    let mut number = 1u16;

    for row in 0..height {
        for col in 0..width {
            let across = cell_needs_across_clue(blank_grid, row, col);
            let down = cell_needs_down_clue(blank_grid, row, col);
            if across || down {
                if across {
                    ordered.push(clue_at(&clues.across, number, "across")?);
                }
                if down {
                    ordered.push(clue_at(&clues.down, number, "down")?);
                }
                number += 1;
            }
        }
    }
    Ok(ordered)
}

fn clue_at(
    map: &std::collections::HashMap<u16, String>,
    n: u16,
    dir: &str,
) -> Result<String, crate::error::PuzError> {
    map.get(&n)
        .cloned()
        .ok_or_else(|| crate::error::PuzError::InvalidClues {
            reason: format!("missing {dir} clue for number {n}"),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::PuzError;
    use crate::types::Clues;
    use std::collections::HashMap;

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
        let grid = vec!["---".to_string(), "...".to_string(), "--.".to_string()];

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
        let grid = vec!["-.-".to_string(), "-.-".to_string(), "...".to_string()];

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
        let grid = vec!["---".to_string(), "-.-".to_string(), "---".to_string()];

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
        let grid = vec!["---".to_string(), "-.-".to_string(), "---".to_string()];
        // Across starts: (0,0), (2,0). Down starts: (0,0), (0,2).
        assert_eq!(count_clues(&grid), (2, 2));
    }

    #[test]
    fn test_count_clues_complex() {
        let grid = vec!["--.".to_string(), "...".to_string(), ".--".to_string()];
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
        for c in [
            'A', 'Z', 'a', 'z', '0', '9', ' ', '-', '\'', '&', '.', '!', '?',
        ] {
            assert!(is_valid_puzzle_char(c), "expected {c:?} to be valid");
        }
        for c in ['\0', '\n', '\t', '@', '#', '$', '%', '^', '*', '(', ')'] {
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
            vec![
                "-----".to_string(),
                "-.-.-".to_string(),
                "-----".to_string(),
            ],
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

    // --- order_clues ---

    #[test]
    fn test_order_simple_2x2_open_grid() {
        // 2x2 all-open: (0,0) starts across #1 and down #1; (0,1) down #2;
        // (1,0) across #3.
        let blank = vec!["--".to_string(), "--".to_string()];
        let mut across = HashMap::new();
        across.insert(1, "a1".to_string());
        across.insert(3, "a3".to_string());
        let mut down = HashMap::new();
        down.insert(1, "d1".to_string());
        down.insert(2, "d2".to_string());
        let clues = Clues { across, down };
        assert_eq!(
            order_clues(&blank, &clues).unwrap(),
            vec!["a1", "d1", "d2", "a3"]
        );
    }

    #[test]
    fn test_order_3x3_with_center_block() {
        // Verified empirically against parser numbering.
        let blank = vec!["---".to_string(), "-.-".to_string(), "---".to_string()];
        let mut across = HashMap::new();
        across.insert(1, "1a".to_string());
        across.insert(3, "3a".to_string());
        let mut down = HashMap::new();
        down.insert(1, "1d".to_string());
        down.insert(2, "2d".to_string());
        let clues = Clues { across, down };
        assert_eq!(
            order_clues(&blank, &clues).unwrap(),
            vec!["1a", "1d", "2d", "3a"]
        );
    }

    #[test]
    fn test_order_emits_across_before_down_at_same_number() {
        let blank = vec!["--".to_string(), "--".to_string()];
        let mut across = HashMap::new();
        across.insert(1, "ACROSS".to_string());
        across.insert(3, "x".to_string());
        let mut down = HashMap::new();
        down.insert(1, "DOWN".to_string());
        down.insert(2, "y".to_string());
        let clues = Clues { across, down };
        let ordered = order_clues(&blank, &clues).unwrap();
        assert_eq!(ordered[0], "ACROSS");
        assert_eq!(ordered[1], "DOWN");
    }

    #[test]
    fn test_order_missing_clue_errors() {
        let blank = vec!["--".to_string(), "--".to_string()];
        let mut across = HashMap::new();
        across.insert(1, "a1".to_string());
        let clues = Clues {
            across,
            down: HashMap::new(),
        };
        assert!(matches!(
            order_clues(&blank, &clues).unwrap_err(),
            PuzError::InvalidClues { .. }
        ));
    }

    #[test]
    fn test_order_ignores_extra_unreferenced_clues() {
        let blank = vec!["--".to_string(), "--".to_string()];
        let mut across = HashMap::new();
        across.insert(1, "a1".to_string());
        across.insert(3, "a3".to_string());
        across.insert(99, "orphan".to_string());
        let mut down = HashMap::new();
        down.insert(1, "d1".to_string());
        down.insert(2, "d2".to_string());
        let clues = Clues { across, down };
        let ordered = order_clues(&blank, &clues).unwrap();
        assert_eq!(ordered, vec!["a1", "d1", "d2", "a3"]);
        assert!(!ordered.contains(&"orphan".to_string()));
    }

    #[test]
    fn test_order_empty_grid_yields_no_clues() {
        let blank: Vec<String> = vec![];
        let clues = Clues {
            across: HashMap::new(),
            down: HashMap::new(),
        };
        assert!(order_clues(&blank, &clues).unwrap().is_empty());
    }
}
