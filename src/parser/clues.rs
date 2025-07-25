use super::grids::{cell_needs_across_clue, cell_needs_down_clue};
use crate::{error::PuzError, types::Clues};
use std::collections::HashMap;

/// Process clues to map them to grid positions
pub(crate) fn process_clues(
    blank_grid: &[String],
    clue_strings: &[String],
) -> Result<Clues, PuzError> {
    let mut across = HashMap::new();
    let mut down = HashMap::new();
    let mut clue_index = 0;
    let mut clue_number = 1u16;

    let height = blank_grid.len();
    let width = if height > 0 { blank_grid[0].len() } else { 0 };

    for row in 0..height {
        for col in 0..width {
            let mut needs_across = false;
            let mut needs_down = false;

            if cell_needs_across_clue(blank_grid, row, col) {
                needs_across = true;
            }

            if cell_needs_down_clue(blank_grid, row, col) {
                needs_down = true;
            }

            if needs_across || needs_down {
                if needs_across {
                    if clue_index < clue_strings.len() {
                        across.insert(clue_number, clue_strings[clue_index].clone());
                        clue_index += 1;
                    } else {
                        return Err(PuzError::InvalidClues {
                            reason: format!(
                                "Not enough clues provided: need across clue for position {}",
                                clue_number
                            ),
                        });
                    }
                }

                if needs_down {
                    if clue_index < clue_strings.len() {
                        down.insert(clue_number, clue_strings[clue_index].clone());
                        clue_index += 1;
                    } else {
                        return Err(PuzError::InvalidClues {
                            reason: format!(
                                "Not enough clues provided: need down clue for position {}",
                                clue_number
                            ),
                        });
                    }
                }

                clue_number += 1;
            }
        }
    }

    // Check if we have unused clues
    if clue_index < clue_strings.len() {
        return Err(PuzError::InvalidClues {
            reason: format!(
                "Too many clues provided: expected {}, got {}",
                clue_index,
                clue_strings.len()
            ),
        });
    }

    Ok(Clues { across, down })
}
