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
    fn test_order_empty_grid_yields_no_clues() {
        let blank: Vec<String> = vec![];
        let clues = Clues {
            across: HashMap::new(),
            down: HashMap::new(),
        };
        assert!(order_clues(&blank, &clues).unwrap().is_empty());
    }
}
