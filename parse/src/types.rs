use std::collections::HashMap;

/// A complete crossword puzzle parsed from a .puz file.
///
/// This is the main data structure returned by the parsing functions.
/// It contains all the information needed to display and interact with
/// a crossword puzzle.
///
/// # Examples
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

/// Basic information about the puzzle.
///
/// Contains metadata like title, author, dimensions, and format information.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct PuzzleInfo {
    /// Puzzle title
    pub title: String,
    /// Author name(s)
    pub author: String,
    /// Copyright information
    pub copyright: String,
    /// Additional notes or instructions
    pub notes: String,
    /// Grid width (number of columns)
    pub width: u8,
    /// Grid height (number of rows)
    pub height: u8,
    /// File format version
    pub version: String,
    /// Whether the puzzle solution is scrambled
    pub is_scrambled: bool,
}

/// The puzzle grid containing both solution and blank layouts.
///
/// The grid is represented as vectors of strings, where each string is a row.
/// Characters represent:
/// - `.` = black/blocked square
/// - `-` = empty square (in blank grid)
/// - Letters/numbers = cell content
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct Grid {
    /// The blank grid as presented to the solver (with `-` for empty squares)
    pub blank: Vec<String>,
    /// The solution grid with all answers filled in
    pub solution: Vec<String>,
}

/// Clues organized by direction and number.
///
/// Clue numbers correspond to the starting squares in the grid.
/// The ordering follows standard crossword conventions.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct Clues {
    /// Across clues mapped by clue number
    pub across: HashMap<u16, String>,
    /// Down clues mapped by clue number
    pub down: HashMap<u16, String>,
}

/// Optional puzzle extensions for advanced features.
///
/// These fields contain additional puzzle information like rebus squares
/// (multi-letter cells), circled squares, and given squares.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct Extensions {
    /// Rebus squares (cells with multiple letters), if present
    pub rebus: Option<Rebus>,
    /// Grid indicating which squares are circled, if any
    pub circles: Option<Vec<Vec<bool>>>,
    /// Grid indicating which squares were given to the solver, if any
    pub given: Option<Vec<Vec<bool>>>,
}

/// Rebus information for squares containing multiple letters.
///
/// A rebus allows a single square to contain multiple letters or words.
/// The grid indicates which squares are rebus squares, and the table
/// maps rebus keys to their string values.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct Rebus {
    /// Grid indicating rebus keys (0 = no rebus, 1-255 = rebus key)
    pub grid: Vec<Vec<u8>>,
    /// Mapping of rebus keys to their string values
    pub table: HashMap<u8, String>,
}

pub(crate) const FREE_SQUARE: char = '-';
pub(crate) const TAKEN_SQUARE: char = '.';
