use crate::error::PuzError;
use crate::grid::{cell_needs_across_clue, cell_needs_down_clue};
use crate::types::Clues;
use std::collections::HashMap;

/// Flatten the `Clues` maps into the order the `.puz` format stores them.
///
/// This is the inverse of `parser::clues::process_clues`: walk the blank grid
/// row-major, and at each numbered cell emit its across clue (if it starts an
/// across word) then its down clue (if it starts a down word). The clue number
/// increments once per numbered cell, matching the parser's read order.
pub(crate) fn order_clues(blank_grid: &[String], clues: &Clues) -> Result<Vec<String>, PuzError> {
    let mut ordered = Vec::new();
    let height = blank_grid.len();
    let width = if height > 0 { blank_grid[0].len() } else { 0 };
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

fn clue_at(map: &HashMap<u16, String>, n: u16, dir: &str) -> Result<String, PuzError> {
    map.get(&n).cloned().ok_or_else(|| PuzError::InvalidClues {
        reason: format!("missing {dir} clue for number {n}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_simple_2x2_open_grid() {
        // 2x2 all-open grid: cell (0,0) starts across #1 and down #1;
        // (0,1) starts down #2; (1,0) starts across #3.
        let blank = vec!["--".to_string(), "--".to_string()];
        let mut across = HashMap::new();
        across.insert(1, "a1".to_string());
        across.insert(3, "a3".to_string());
        let mut down = HashMap::new();
        down.insert(1, "d1".to_string());
        down.insert(2, "d2".to_string());
        let clues = Clues { across, down };

        let ordered = order_clues(&blank, &clues).unwrap();
        assert_eq!(ordered, vec!["a1", "d1", "d2", "a3"]);
    }

    #[test]
    fn test_order_3x3_with_center_block() {
        // 3x3 with a single center block. Numbering (matches
        // parser::process_clues; verified empirically):
        //   - - -    (0,0) #1: starts across (right playable, at left edge) and
        //   - . -          down (below playable, at top edge) -> emit a then d
        //   - - -    (0,2) #2: down only (below playable, top edge); no across
        //                      because col+1 is off-grid
        //            (2,0) #3: across only (right playable, left edge); no down
        //                      because row+1 is off-grid
        // The center block prevents (1,x)/(x,1) from starting words, and bottom
        // row / right column can't start down/across respectively.
        let blank = vec![
            "---".to_string(),
            "-.-".to_string(),
            "---".to_string(),
        ];
        let mut across = HashMap::new();
        across.insert(1, "1a".to_string());
        across.insert(3, "3a".to_string());
        let mut down = HashMap::new();
        down.insert(1, "1d".to_string());
        down.insert(2, "2d".to_string());
        let clues = Clues { across, down };

        let ordered = order_clues(&blank, &clues).unwrap();
        assert_eq!(ordered, vec!["1a", "1d", "2d", "3a"]);
    }

    #[test]
    fn test_order_emits_across_before_down_at_same_number() {
        // A cell that starts BOTH must emit across first, then down.
        let blank = vec!["--".to_string(), "--".to_string()];
        let mut across = HashMap::new();
        across.insert(1, "ACROSS".to_string());
        across.insert(3, "x".to_string());
        let mut down = HashMap::new();
        down.insert(1, "DOWN".to_string());
        down.insert(2, "y".to_string());
        let clues = Clues { across, down };

        let ordered = order_clues(&blank, &clues).unwrap();
        // At number 1 the across clue precedes the down clue.
        assert_eq!(ordered[0], "ACROSS");
        assert_eq!(ordered[1], "DOWN");
    }

    #[test]
    fn test_order_missing_clue_errors() {
        let blank = vec!["--".to_string(), "--".to_string()];
        // Provide across #1 but omit down #1 -> error.
        let mut across = HashMap::new();
        across.insert(1, "a1".to_string());
        let clues = Clues {
            across,
            down: HashMap::new(),
        };

        let err = order_clues(&blank, &clues).unwrap_err();
        assert!(matches!(err, PuzError::InvalidClues { .. }));
    }

    #[test]
    fn test_order_ignores_extra_unreferenced_clues() {
        // order_clues pulls by number, so extra map entries that no cell
        // references are simply not emitted (they don't error here; the
        // count mismatch is caught by validation in Task 9).
        let blank = vec!["--".to_string(), "--".to_string()];
        let mut across = HashMap::new();
        across.insert(1, "a1".to_string());
        across.insert(3, "a3".to_string());
        across.insert(99, "orphan".to_string()); // no cell #99
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
