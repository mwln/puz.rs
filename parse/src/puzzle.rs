use crate::error::PuzError;
use crate::grid::{cell_needs_across_clue, cell_needs_down_clue, FREE_SQUARE, TAKEN_SQUARE};
use crate::parser::validate_puzzle;
use crate::types::{Clues, Extensions, Grid, PuzzleInfo};

/// A complete crossword puzzle.
///
/// This is the main data structure returned by the parsing functions and the
/// value you build with [`Puzzle::new`]. It contains everything needed to
/// display and interact with a crossword puzzle.
///
/// # Parsing
///
/// ```rust,no_run
/// use puz_parse::parse_file;
///
/// let puzzle = parse_file("puzzle.puz")?;
/// println!("Title: {}", puzzle.info.title);
/// println!("Grid size: {}x{}", puzzle.info.width, puzzle.info.height);
/// println!("Number of across clues: {}", puzzle.clues.across.len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Building
///
/// ```rust
/// use puz_parse::Puzzle;
///
/// let puzzle = Puzzle::new(["AB.", "CDE"])?   // '.' is a black square
///     .title("Example")
///     .author("Me")
///     .diagramless(true);
/// # Ok::<(), puz_parse::PuzError>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct Puzzle {
    /// Basic puzzle metadata (title, author, dimensions, etc.)
    pub info: PuzzleInfo,
    /// The puzzle grid (solution and blank grids)
    pub grid: Grid,
    /// Clues organized by direction and number
    pub clues: Clues,
    /// Optional puzzle extensions (rebus, circles, etc.)
    pub extensions: Extensions,
}

impl Puzzle {
    /// Build a valid puzzle from solution rows.
    ///
    /// Each row is a string using `.` for black squares and letters/digits for
    /// filled cells. The blank grid (`-` for open cells, `.` for black) and
    /// placeholder clues are generated automatically, so the result is a
    /// self-consistent puzzle. Refine it with the chained setters
    /// ([`Puzzle::title`], [`Puzzle::diagramless`], and so on).
    ///
    /// Returns an error if the rows do not form a valid grid (for example,
    /// rows of differing widths).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use puz_parse::Puzzle;
    ///
    /// let puzzle = Puzzle::new(["AB.", "CDE"])?;
    /// assert_eq!(puzzle.info.width, 3);
    /// assert_eq!(puzzle.info.height, 2);
    /// # Ok::<(), puz_parse::PuzError>(())
    /// ```
    pub fn new<I, S>(rows: I) -> Result<Puzzle, PuzError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let solution: Vec<String> = rows.into_iter().map(|r| r.as_ref().to_string()).collect();

        let height = solution.len();
        let width = solution.first().map(|r| r.chars().count()).unwrap_or(0);
        if height > 255 || width > 255 {
            return Err(PuzError::InvalidGrid {
                reason: format!("grid {width}x{height} exceeds the 255x255 maximum"),
            });
        }
        for (i, row) in solution.iter().enumerate() {
            let cells = row.chars().count();
            if cells != width {
                return Err(PuzError::InvalidGrid {
                    reason: format!(
                        "row {i} has width {cells}, expected {width} (rows must be equal width)"
                    ),
                });
            }
        }

        // Blank grid mirrors the solution: black squares stay '.', everything
        // else becomes an open cell '-'.
        let blank: Vec<String> = solution
            .iter()
            .map(|row| {
                row.chars()
                    .map(|c| {
                        if c == TAKEN_SQUARE {
                            TAKEN_SQUARE
                        } else {
                            FREE_SQUARE
                        }
                    })
                    .collect()
            })
            .collect();

        let clues = generate_placeholder_clues(&blank);

        let puzzle = Puzzle {
            info: PuzzleInfo {
                title: String::new(),
                author: String::new(),
                copyright: String::new(),
                notes: String::new(),
                width: width as u8,
                height: height as u8,
                version: "1.3".to_string(),
                is_scrambled: false,
                is_diagramless: false,
            },
            grid: Grid { blank, solution },
            clues,
            extensions: Extensions {
                rebus: None,
                circles: None,
                given: None,
            },
        };

        validate_puzzle(&puzzle)?;
        Ok(puzzle)
    }

    /// Set the puzzle title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.info.title = title.into();
        self
    }

    /// Set the puzzle author.
    #[must_use]
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.info.author = author.into();
        self
    }

    /// Set the copyright text.
    #[must_use]
    pub fn copyright(mut self, copyright: impl Into<String>) -> Self {
        self.info.copyright = copyright.into();
        self
    }

    /// Set the notes/instructions text.
    #[must_use]
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.info.notes = notes.into();
        self
    }

    /// Set the file format version (defaults to `"1.3"`).
    #[must_use]
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.info.version = version.into();
        self
    }

    /// Mark the puzzle as diagramless (or not).
    ///
    /// A diagramless puzzle hides the black squares from the solver. When
    /// written to `.puz`, black squares are emitted as `:` and the diagramless
    /// bitmask is set.
    #[must_use]
    pub fn diagramless(mut self, is_diagramless: bool) -> Self {
        self.info.is_diagramless = is_diagramless;
        self
    }

    /// Replace the generated placeholder clues with explicit ones.
    ///
    /// The caller is responsible for providing an entry for every numbered slot;
    /// this is a convenience and is not validated against the grid.
    #[must_use]
    pub fn clues(mut self, clues: Clues) -> Self {
        self.clues = clues;
        self
    }
}

/// Generate one placeholder clue per across/down slot, numbered in reading order.
///
/// Mirrors the numbering in [`crate::grid::order_clues`] so generated clues line
/// up with the writer's slot ordering.
fn generate_placeholder_clues(blank: &[String]) -> Clues {
    let mut clues = Clues::default();
    let height = blank.len();
    let width = blank.first().map(|r| r.chars().count()).unwrap_or(0);
    let mut number = 1u16;

    for row in 0..height {
        for col in 0..width {
            let needs_across = cell_needs_across_clue(blank, row, col);
            let needs_down = cell_needs_down_clue(blank, row, col);
            if needs_across || needs_down {
                if needs_across {
                    clues.across.set(number, format!("Across {number}"));
                }
                if needs_down {
                    clues.down.set(number, format!("Down {number}"));
                }
                number += 1;
            }
        }
    }

    clues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_derives_dimensions() {
        let puzzle = Puzzle::new(["AB.", "CDE"]).unwrap();
        assert_eq!(puzzle.info.width, 3);
        assert_eq!(puzzle.info.height, 2);
    }

    #[test]
    fn test_new_generates_blank_grid_matching_black_squares() {
        let puzzle = Puzzle::new(["AB.", "CDE"]).unwrap();
        assert_eq!(
            puzzle.grid.blank,
            vec!["--.".to_string(), "---".to_string()]
        );
        assert_eq!(
            puzzle.grid.solution,
            vec!["AB.".to_string(), "CDE".to_string()]
        );
    }

    #[test]
    fn test_new_generates_a_clue_for_every_slot() {
        // A 2x2 open grid has slots numbered 1..=3.
        let puzzle = Puzzle::new(["AB", "CD"]).unwrap();
        // Across: 1 and 3. Down: 1 and 2.
        assert!(puzzle.clues.across.contains(1));
        assert!(puzzle.clues.across.contains(3));
        assert!(puzzle.clues.down.contains(1));
        assert!(puzzle.clues.down.contains(2));
    }

    #[test]
    fn test_ragged_rows_error() {
        let err = Puzzle::new(["ABC", "DE"]).unwrap_err();
        assert!(matches!(err, PuzError::InvalidGrid { .. }));
    }

    #[test]
    fn test_diagramless_toggle_sets_flag() {
        let puzzle = Puzzle::new(["AB.", "CDE"]).unwrap().diagramless(true);
        assert!(puzzle.info.is_diagramless);
    }

    #[test]
    fn test_setters_chain() {
        let puzzle = Puzzle::new(["AB", "CD"])
            .unwrap()
            .title("T")
            .author("A")
            .copyright("C")
            .notes("N")
            .version("1.4");
        assert_eq!(puzzle.info.title, "T");
        assert_eq!(puzzle.info.author, "A");
        assert_eq!(puzzle.info.copyright, "C");
        assert_eq!(puzzle.info.notes, "N");
        assert_eq!(puzzle.info.version, "1.4");
    }
}
