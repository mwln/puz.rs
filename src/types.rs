use std::collections::HashMap;

/// A complete crossword puzzle parsed from a .puz file.
#[derive(Debug, Clone, PartialEq)]
pub struct Puzzle {
    /// Basic puzzle information (title, author, etc.)
    pub info: PuzzleInfo,
    /// Grid dimensions and layout
    pub grid: Grid,
    /// Clues for across and down
    pub clues: Clues,
    /// Optional puzzle extensions (rebus, circles, etc.)
    pub extensions: Extensions,
}

/// Basic information about the puzzle.
#[derive(Debug, Clone, PartialEq)]
pub struct PuzzleInfo {
    /// Puzzle title
    pub title: String,
    /// Author name
    pub author: String,
    /// Copyright information
    pub copyright: String,
    /// Additional notes
    pub notes: String,
    /// Grid width
    pub width: u8,
    /// Grid height
    pub height: u8,
    /// File format version
    pub version: String,
    /// Whether the puzzle is scrambled
    pub is_scrambled: bool,
}

/// The puzzle grid containing the layout and solution.
#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    /// The blank grid showing black squares ('.') and open squares ('-')
    pub blank: Vec<String>,
    /// The solution grid with filled letters
    pub solution: Vec<String>,
}

/// Clues organized by direction and number.
#[derive(Debug, Clone, PartialEq)]
pub struct Clues {
    /// Across clues mapped by clue number
    pub across: HashMap<u16, String>,
    /// Down clues mapped by clue number
    pub down: HashMap<u16, String>,
}

/// Optional puzzle extensions for advanced features.
#[derive(Debug, Clone, PartialEq)]
pub struct Extensions {
    /// Rebus squares (squares with multiple letters)
    pub rebus: Option<Rebus>,
    /// Circled or marked squares
    pub circles: Option<Vec<Vec<bool>>>,
    /// Squares that were given to the solver
    pub given: Option<Vec<Vec<bool>>>,
}

/// Rebus information for squares containing multiple letters.
#[derive(Debug, Clone, PartialEq)]
pub struct Rebus {
    /// Grid indicating which squares are rebus (0 = no rebus, n = rebus key)
    pub grid: Vec<Vec<u8>>,
    /// Mapping of rebus keys to their string values
    pub table: HashMap<u8, String>,
}

/// Constants used in puzzle parsing.
pub(crate) const FREE_SQUARE: char = '-';
pub(crate) const TAKEN_SQUARE: char = '.';
