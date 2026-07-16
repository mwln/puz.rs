use std::io::Read;
use std::path::Path;

use crate::error::{ParseResult, PuzError};
use crate::grid::{FREE_SQUARE, TAKEN_SQUARE, cell_needs_across_clue, cell_needs_down_clue};
use crate::types::{ClueAnswer, Clues, Direction, Extensions, Grid, PuzzleInfo};

/// A complete crossword puzzle.
///
/// This is the main data structure returned by the parsing functions and the
/// value you build with [`Puzzle::new`]. It contains everything needed to
/// display and interact with a crossword puzzle.
///
/// # Parsing
///
/// ```rust,no_run
/// use puz_parse::Puzzle;
///
/// let puzzle = Puzzle::from_file("puzzle.puz")?;
/// println!("Title: {}", puzzle.info.title);
/// println!("Grid size: {}x{}", puzzle.info.width, puzzle.info.height);
/// println!("Number of across clues: {}", puzzle.clues.across.len());
/// # Ok::<(), puz_parse::PuzError>(())
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

    /// Parse a puzzle from a `.puz` file path.
    ///
    /// Checksum mismatches and other recoverable issues are ignored; use
    /// [`Puzzle::reader`] to configure strict parsing or to collect warnings.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use puz_parse::Puzzle;
    ///
    /// let puzzle = Puzzle::from_file("puzzle.puz")?;
    /// println!("{} by {}", puzzle.info.title, puzzle.info.author);
    /// # Ok::<(), puz_parse::PuzError>(())
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Puzzle, PuzError> {
        PuzzleReader::new().from_file(path)
    }

    /// Parse a puzzle from `.puz` bytes already in memory.
    ///
    /// Checksum mismatches and other recoverable issues are ignored; use
    /// [`Puzzle::reader`] to configure strict parsing or to collect warnings.
    pub fn from_bytes(data: &[u8]) -> Result<Puzzle, PuzError> {
        PuzzleReader::new().from_bytes(data)
    }

    /// Parse a puzzle from any [`Read`] source.
    ///
    /// Checksum mismatches and other recoverable issues are ignored; use
    /// [`Puzzle::reader`] to configure strict parsing or to collect warnings.
    pub fn from_reader<R: Read>(reader: R) -> Result<Puzzle, PuzError> {
        PuzzleReader::new().from_reader(reader)
    }

    /// Begin a configurable parse.
    ///
    /// Set options such as [`PuzzleReader::strict`], then call a terminal
    /// method ([`PuzzleReader::from_file`], [`PuzzleReader::from_bytes`],
    /// [`PuzzleReader::from_reader`], or their `*_verbose` variants that also
    /// return parse warnings).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use puz_parse::Puzzle;
    ///
    /// // Reject any checksum mismatch.
    /// let puzzle = Puzzle::reader().strict(true).from_file("puzzle.puz")?;
    ///
    /// // Keep the parse warnings.
    /// let parsed = Puzzle::reader().from_file_verbose("puzzle.puz")?;
    /// for warning in &parsed.warnings {
    ///     eprintln!("{warning}");
    /// }
    /// # Ok::<(), puz_parse::PuzError>(())
    /// ```
    #[must_use]
    pub fn reader() -> PuzzleReader {
        PuzzleReader::new()
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

    /// Pair every clue with the answer read from the solution grid.
    ///
    /// Walks the grid in reading order. For each numbered cell, an across entry
    /// reads solution cells rightward until a black square or the grid edge; a
    /// down entry reads downward the same way. Each entry is matched with its
    /// clue text from [`Clues::across`] / [`Clues::down`]. Entries are returned
    /// in reading order (across before down at the same number).
    ///
    /// The answer characters are taken from the solution grid as-is, so a rebus
    /// or theme cell contributes whatever character the grid stores there.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use puz_parse::{Direction, Puzzle};
    ///
    /// let puzzle = Puzzle::new().grid(["AB", "CD"]);
    /// let entries = puzzle.clue_answers();
    /// let a1 = entries.iter().find(|e| e.direction == Direction::Across && e.number == 1).unwrap();
    /// assert_eq!(a1.answer, "AB");
    /// ```
    pub fn clue_answers(&self) -> Vec<ClueAnswer> {
        let blank = &self.grid.blank;
        let solution = &self.grid.solution;
        let width = blank.first().map(|r| r.chars().count()).unwrap_or(0);
        let height = blank.len();

        // Row-major char grid of the solution for O(1)-ish cell access.
        let sol: Vec<Vec<char>> = solution.iter().map(|r| r.chars().collect()).collect();
        let cell = |row: usize, col: usize| -> Option<char> {
            sol.get(row).and_then(|r| r.get(col)).copied()
        };

        let mut out = Vec::new();
        let mut number = 1u16;
        for row in 0..height {
            for col in 0..width {
                let starts_across = cell_needs_across_clue(blank, row, col);
                let starts_down = cell_needs_down_clue(blank, row, col);
                if !(starts_across || starts_down) {
                    continue;
                }

                if starts_across {
                    let mut answer = String::new();
                    let mut c = col;
                    while let Some(ch) = cell(row, c) {
                        if ch == TAKEN_SQUARE {
                            break;
                        }
                        answer.push(ch);
                        c += 1;
                    }
                    out.push(ClueAnswer {
                        direction: Direction::Across,
                        number,
                        clue: self.clues.across.get(number).unwrap_or("").to_string(),
                        answer,
                    });
                }

                if starts_down {
                    let mut answer = String::new();
                    let mut r = row;
                    while let Some(ch) = cell(r, col) {
                        if ch == TAKEN_SQUARE {
                            break;
                        }
                        answer.push(ch);
                        r += 1;
                    }
                    out.push(ClueAnswer {
                        direction: Direction::Down,
                        number,
                        clue: self.clues.down.get(number).unwrap_or("").to_string(),
                        answer,
                    });
                }

                number += 1;
            }
        }
        out
    }
}

impl Default for Puzzle {
    fn default() -> Self {
        Puzzle::new()
    }
}

/// A configurable `.puz` parser.
///
/// Created with [`Puzzle::reader`]. Set options with the chained setters, then
/// call a terminal method to parse from a file, bytes, or a reader. The
/// `from_*` terminals return just the [`Puzzle`] (discarding warnings); the
/// `from_*_verbose` terminals return a [`ParseResult`] that also carries the
/// parse warnings.
#[derive(Debug, Clone, Default)]
pub struct PuzzleReader {
    strict: bool,
}

impl PuzzleReader {
    fn new() -> Self {
        Self { strict: false }
    }

    /// Require all stored checksums to match.
    ///
    /// When `true`, a checksum mismatch is returned as
    /// [`PuzError::InvalidChecksum`] instead of being recorded as a warning.
    #[must_use]
    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    fn parse<R: Read>(&self, reader: R) -> Result<ParseResult<Puzzle>, PuzError> {
        if self.strict {
            crate::parser::parse_puzzle_strict(reader)
        } else {
            crate::parser::parse_puzzle(reader)
        }
    }

    fn open(path: &Path) -> Result<std::fs::File, PuzError> {
        // `io::Error` converts into `PuzError::Io` via `#[from]`, preserving the
        // original error as the source.
        Ok(std::fs::File::open(path)?)
    }

    /// Parse from a file path, returning the puzzle and its warnings.
    pub fn from_file_verbose<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<ParseResult<Puzzle>, PuzError> {
        self.parse(Self::open(path.as_ref())?)
    }

    /// Parse from bytes, returning the puzzle and its warnings.
    pub fn from_bytes_verbose(&self, data: &[u8]) -> Result<ParseResult<Puzzle>, PuzError> {
        self.parse(data)
    }

    /// Parse from any [`Read`] source, returning the puzzle and its warnings.
    pub fn from_reader_verbose<R: Read>(&self, reader: R) -> Result<ParseResult<Puzzle>, PuzError> {
        self.parse(reader)
    }

    /// Parse from a file path, discarding warnings.
    pub fn from_file<P: AsRef<Path>>(&self, path: P) -> Result<Puzzle, PuzError> {
        self.from_file_verbose(path).map(|r| r.result)
    }

    /// Parse from bytes, discarding warnings.
    pub fn from_bytes(&self, data: &[u8]) -> Result<Puzzle, PuzError> {
        self.from_bytes_verbose(data).map(|r| r.result)
    }

    /// Parse from any [`Read`] source, discarding warnings.
    pub fn from_reader<R: Read>(&self, reader: R) -> Result<Puzzle, PuzError> {
        self.from_reader_verbose(reader).map(|r| r.result)
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

    // Rebuild through `Clues::new` so `raw` is populated in reading order,
    // matching how a parsed puzzle's clues look.
    Clues::new(clues.across, clues.down)
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

    #[test]
    fn test_from_bytes_round_trip() {
        let p = Puzzle::new().title("T").author("A").grid(["AB", "CD"]);
        let bytes = crate::to_bytes(&p).unwrap();
        let parsed = Puzzle::from_bytes(&bytes).unwrap();
        assert_eq!(parsed, p);
    }

    #[test]
    fn test_reader_verbose_returns_warnings() {
        // A well-formed puzzle our writer produced has valid checksums, so the
        // verbose parse yields no warnings.
        let p = Puzzle::new().title("T").author("A").grid(["AB", "CD"]);
        let bytes = crate::to_bytes(&p).unwrap();
        let parsed = Puzzle::reader().from_bytes_verbose(&bytes).unwrap();
        assert_eq!(parsed.result, p);
        assert!(parsed.warnings.is_empty());
    }

    #[test]
    fn test_reader_strict_rejects_bad_checksum() {
        let p = Puzzle::new().title("T").author("A").grid(["AB", "CD"]);
        let mut bytes = crate::to_bytes(&p).unwrap();
        // Corrupt the global checksum at offset 0x00.
        bytes[0] ^= 0xFF;

        // Lenient parse records a warning but succeeds.
        let lenient = Puzzle::reader().from_bytes_verbose(&bytes).unwrap();
        assert!(!lenient.warnings.is_empty());

        // Strict parse rejects it.
        let err = Puzzle::reader()
            .strict(true)
            .from_bytes(&bytes)
            .unwrap_err();
        assert!(matches!(err, PuzError::InvalidChecksum { .. }));
    }

    #[test]
    fn test_clue_answers_reads_solution_letters() {
        // 2x2 open grid. Slots: 1A "AB", 1D "AC", 2D "BD", 3A "CD".
        let mut p = Puzzle::new().grid(["AB", "CD"]);
        p.clues.across.set(1, "top row");
        p.clues.across.set(3, "bottom row");
        p.clues.down.set(1, "left col");
        p.clues.down.set(2, "right col");

        let entries = p.clue_answers();

        let find = |dir: Direction, n: u16| {
            entries
                .iter()
                .find(|e| e.direction == dir && e.number == n)
                .unwrap_or_else(|| panic!("missing {dir:?} {n}"))
        };
        assert_eq!(find(Direction::Across, 1).answer, "AB");
        assert_eq!(find(Direction::Across, 1).clue, "top row");
        assert_eq!(find(Direction::Across, 3).answer, "CD");
        assert_eq!(find(Direction::Down, 1).answer, "AC");
        assert_eq!(find(Direction::Down, 2).answer, "BD");
    }

    #[test]
    fn test_clue_answers_stops_at_black_squares() {
        // Row 0: "AB." -> 1A is "AB" (stops before the black square).
        let p = Puzzle::new().grid(["AB.", "CDE"]);
        let entries = p.clue_answers();
        let a1 = entries
            .iter()
            .find(|e| e.direction == Direction::Across && e.number == 1)
            .unwrap();
        assert_eq!(a1.answer, "AB");
    }

    #[test]
    fn test_clue_answers_reading_order_across_before_down() {
        // At a cell that starts both, across comes before down.
        let p = Puzzle::new().grid(["AB", "CD"]);
        let entries = p.clue_answers();
        assert_eq!(entries[0].direction, Direction::Across);
        assert_eq!(entries[0].number, 1);
        assert_eq!(entries[1].direction, Direction::Down);
        assert_eq!(entries[1].number, 1);
    }
}
