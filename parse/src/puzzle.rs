use crate::grid::{cell_needs_across_clue, cell_needs_down_clue, FREE_SQUARE, TAKEN_SQUARE};
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
/// let puzzle = Puzzle::new()
///     .title("Example")
///     .author("Me")
///     .grid(["AB.", "CDE"])   // '.' is a black square
///     .diagramless(true);
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
    /// Start a new, empty puzzle.
    ///
    /// The puzzle has no grid and no clues yet; add them with [`Puzzle::grid`]
    /// and refine metadata with the chained setters ([`Puzzle::title`],
    /// [`Puzzle::diagramless`], and so on). The chain is infallible; the puzzle
    /// is validated when it is written (for example by
    /// [`to_bytes`](crate::to_bytes)).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use puz_parse::Puzzle;
    ///
    /// let puzzle = Puzzle::new()
    ///     .title("Example")
    ///     .author("Me")
    ///     .grid(["AB.", "CDE"]);
    ///
    /// assert_eq!(puzzle.info.width, 3);
    /// assert_eq!(puzzle.info.height, 2);
    /// ```
    #[must_use]
    pub fn new() -> Puzzle {
        Puzzle {
            info: PuzzleInfo {
                title: String::new(),
                author: String::new(),
                copyright: String::new(),
                notes: String::new(),
                width: 0,
                height: 0,
                version: "1.3".to_string(),
                is_scrambled: false,
                is_diagramless: false,
            },
            grid: Grid {
                blank: Vec::new(),
                solution: Vec::new(),
            },
            clues: Clues::default(),
            extensions: Extensions {
                rebus: None,
                circles: None,
                given: None,
            },
        }
    }

    /// Set the puzzle grid from solution rows.
    ///
    /// Each row is a string using `.` for black squares and letters/digits for
    /// filled cells. This derives the width and height, generates the blank grid
    /// (`-` for open cells, `.` for black), and generates placeholder clues for
    /// every slot. Replace those with [`Puzzle::clues`] if you have real clues.
    ///
    /// Malformed grids (for example, rows of differing widths) are not rejected
    /// here; they are caught when the puzzle is written.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use puz_parse::Puzzle;
    ///
    /// let puzzle = Puzzle::new().grid(["AB.", "CDE"]);
    /// assert_eq!(puzzle.grid.blank, vec!["--.".to_string(), "---".to_string()]);
    /// ```
    #[must_use]
    pub fn grid<I, S>(mut self, rows: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let solution: Vec<String> = rows.into_iter().map(|r| r.as_ref().to_string()).collect();

        let height = solution.len();
        let width = solution.first().map(|r| r.chars().count()).unwrap_or(0);

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

        self.clues = generate_placeholder_clues(&blank);
        self.info.width = width.min(u8::MAX as usize) as u8;
        self.info.height = height.min(u8::MAX as usize) as u8;
        self.grid = Grid { blank, solution };
        self
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

impl Default for Puzzle {
    fn default() -> Self {
        Puzzle::new()
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
    fn test_new_is_empty() {
        let puzzle = Puzzle::new();
        assert_eq!(puzzle.info.width, 0);
        assert_eq!(puzzle.info.height, 0);
        assert_eq!(puzzle.info.version, "1.3");
        assert!(puzzle.grid.solution.is_empty());
        assert!(puzzle.clues.across.is_empty());
    }

    #[test]
    fn test_grid_derives_dimensions() {
        let puzzle = Puzzle::new().grid(["AB.", "CDE"]);
        assert_eq!(puzzle.info.width, 3);
        assert_eq!(puzzle.info.height, 2);
    }

    #[test]
    fn test_grid_generates_blank_matching_black_squares() {
        let puzzle = Puzzle::new().grid(["AB.", "CDE"]);
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
    fn test_grid_generates_a_clue_for_every_slot() {
        // A 2x2 open grid has slots numbered 1..=3.
        let puzzle = Puzzle::new().grid(["AB", "CD"]);
        // Across: 1 and 3. Down: 1 and 2.
        assert!(puzzle.clues.across.contains(1));
        assert!(puzzle.clues.across.contains(3));
        assert!(puzzle.clues.down.contains(1));
        assert!(puzzle.clues.down.contains(2));
    }

    #[test]
    fn test_diagramless_toggle_sets_flag() {
        let puzzle = Puzzle::new().grid(["AB.", "CDE"]).diagramless(true);
        assert!(puzzle.info.is_diagramless);
    }

    #[test]
    fn test_setters_chain() {
        let puzzle = Puzzle::new()
            .title("T")
            .author("A")
            .copyright("C")
            .notes("N")
            .version("1.4")
            .grid(["AB", "CD"]);
        assert_eq!(puzzle.info.title, "T");
        assert_eq!(puzzle.info.author, "A");
        assert_eq!(puzzle.info.copyright, "C");
        assert_eq!(puzzle.info.notes, "N");
        assert_eq!(puzzle.info.version, "1.4");
        assert_eq!(puzzle.info.width, 2);
    }
}
