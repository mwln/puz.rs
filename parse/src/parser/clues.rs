use crate::{
    error::{PuzError, PuzWarning},
    grid::{cell_needs_across_clue, cell_needs_down_clue},
    types::Clues,
};

/// Map grid word slots to clue strings in reading order.
///
/// The across/down maps hold the numbered view. The full clue list is preserved
/// verbatim in [`Clues::raw`] so no data is lost.
///
/// - Fewer clue strings than grid slots is a hard error: the grid cannot be
///   clued and there is no meaningful mapping.
/// - More clue strings than grid slots is tolerated: the extra strings stay in
///   `raw` and a [`PuzWarning::ExtraClues`] is returned. Some puzzles author
///   extra clues (for example a meta-puzzle revealer) with no grid slot.
pub(crate) fn process_clues(
    blank_grid: &[String],
    clue_strings: &[String],
) -> Result<(Clues, Option<PuzWarning>), PuzError> {
    let mut clues = Clues::default();
    let mut clue_index = 0;
    let mut clue_number = 1u16;

    let height = blank_grid.len();
    let width = if height > 0 { blank_grid[0].len() } else { 0 };

    for row in 0..height {
        for col in 0..width {
            let needs_across = cell_needs_across_clue(blank_grid, row, col);
            let needs_down = cell_needs_down_clue(blank_grid, row, col);

            if needs_across || needs_down {
                if needs_across {
                    if clue_index < clue_strings.len() {
                        clues
                            .across
                            .set(clue_number, clue_strings[clue_index].clone());
                        clue_index += 1;
                    } else {
                        return Err(PuzError::InvalidClues {
                            reason: format!(
                                "Not enough clues provided: need across clue for position {clue_number}"
                            ),
                        });
                    }
                }

                if needs_down {
                    if clue_index < clue_strings.len() {
                        clues
                            .down
                            .set(clue_number, clue_strings[clue_index].clone());
                        clue_index += 1;
                    } else {
                        return Err(PuzError::InvalidClues {
                            reason: format!(
                                "Not enough clues provided: need down clue for position {clue_number}"
                            ),
                        });
                    }
                }

                clue_number += 1;
            }
        }
    }

    // Preserve the complete clue list from the file, in order, with no loss.
    clues.raw = clue_strings.to_vec();

    // Extra clue strings beyond the grid's slots are tolerated: the file stored
    // more clues than the grid can number (e.g. a meta revealer). Keep them in
    // `raw` and report a recoverable warning.
    let warning = if clue_index < clue_strings.len() {
        Some(PuzWarning::ExtraClues {
            slots: clue_index,
            provided: clue_strings.len(),
        })
    } else {
        None
    };

    Ok((clues, warning))
}

#[cfg(test)]
mod tests {
    use super::*;

    // A 2x2 open grid: slots are 1-Across, 3-Across, 1-Down, 2-Down (4 total),
    // emitted in reading order as [1A, 1D, 2D, 3A].
    fn open_2x2() -> Vec<String> {
        vec!["--".to_string(), "--".to_string()]
    }

    #[test]
    fn test_exact_clue_count_maps_and_no_warning() {
        let clues = ["1A", "1D", "2D", "3A"].map(String::from);
        let (result, warning) = process_clues(&open_2x2(), &clues).unwrap();
        assert!(warning.is_none());
        assert_eq!(result.across.get(1), Some("1A"));
        assert_eq!(result.across.get(3), Some("3A"));
        assert_eq!(result.down.get(1), Some("1D"));
        assert_eq!(result.down.get(2), Some("2D"));
        assert_eq!(result.raw, clues.to_vec());
    }

    #[test]
    fn test_extra_clues_warn_and_preserve_raw() {
        // 5 clue strings for a 4-slot grid: the extra is tolerated.
        let clues = ["1A", "1D", "2D", "3A", "EXTRA"].map(String::from);
        let (result, warning) = process_clues(&open_2x2(), &clues).unwrap();

        assert_eq!(
            warning,
            Some(PuzWarning::ExtraClues {
                slots: 4,
                provided: 5,
            })
        );
        // The 4 slots still map correctly...
        assert_eq!(result.across.get(1), Some("1A"));
        assert_eq!(result.down.get(2), Some("2D"));
        // ...and the full list (including the extra) is preserved with no loss.
        assert_eq!(result.raw, clues.to_vec());
        assert_eq!(result.raw.last().map(String::as_str), Some("EXTRA"));
    }

    #[test]
    fn test_too_few_clues_is_an_error() {
        // Only 3 clue strings for a 4-slot grid: cannot number the grid.
        let clues = ["1A", "1D", "2D"].map(String::from);
        let err = process_clues(&open_2x2(), &clues).unwrap_err();
        assert!(matches!(err, PuzError::InvalidClues { .. }));
    }
}
