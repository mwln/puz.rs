use std::collections::HashMap;

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
    /// Whether the puzzle is diagramless (solver isn't shown the black squares).
    pub is_diagramless: bool,
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

/// The direction of a clue or word: across or down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    /// A horizontal word, read left to right.
    Across,
    /// A vertical word, read top to bottom.
    Down,
}

/// The clues for one direction, keyed by clue number.
///
/// Wraps a `HashMap<u16, String>` with a small, ordered interface. Use
/// [`ClueSet::get`] and [`ClueSet::set`] to read and write a clue by number,
/// and [`ClueSet::iter`] to walk the clues in ascending number order. For full
/// map access, use [`ClueSet::as_map`], [`ClueSet::as_map_mut`], or
/// [`ClueSet::into_inner`].
///
/// # Examples
///
/// ```rust
/// use puz_parse::ClueSet;
///
/// let mut across = ClueSet::default();
/// across.set(1, "First across");
/// assert_eq!(across.get(1), Some("First across"));
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json", serde(transparent))]
pub struct ClueSet {
    entries: HashMap<u16, String>,
}

impl ClueSet {
    /// Build a `ClueSet` from `(number, text)` pairs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use puz_parse::ClueSet;
    ///
    /// let across = ClueSet::new([(1, "First across"), (3, "Third across")]);
    /// assert_eq!(across.get(1), Some("First across"));
    /// ```
    pub fn new<I, S>(entries: I) -> Self
    where
        I: IntoIterator<Item = (u16, S)>,
        S: Into<String>,
    {
        Self {
            entries: entries
                .into_iter()
                .map(|(number, text)| (number, text.into()))
                .collect(),
        }
    }

    /// Read the clue at `number`, if present.
    pub fn get(&self, number: u16) -> Option<&str> {
        self.entries.get(&number).map(String::as_str)
    }

    /// Set (or overwrite) the clue at `number`.
    ///
    /// Returns the previous text for that number, if any.
    pub fn set(&mut self, number: u16, text: impl Into<String>) -> Option<String> {
        self.entries.insert(number, text.into())
    }

    /// Remove the clue at `number`. Returns the removed text, if any.
    pub fn remove(&mut self, number: u16) -> Option<String> {
        self.entries.remove(&number)
    }

    /// Whether a clue exists at `number`.
    pub fn contains(&self, number: u16) -> bool {
        self.entries.contains_key(&number)
    }

    /// The number of clues.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether there are no clues.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate the clues as `(number, text)`, in ascending number order.
    pub fn iter(&self) -> impl Iterator<Item = (u16, &str)> {
        let mut entries: Vec<(u16, &str)> = self
            .entries
            .iter()
            .map(|(&n, text)| (n, text.as_str()))
            .collect();
        entries.sort_by_key(|&(n, _)| n);
        entries.into_iter()
    }

    /// Borrow the underlying map.
    pub fn as_map(&self) -> &HashMap<u16, String> {
        &self.entries
    }

    /// Mutably borrow the underlying map.
    pub fn as_map_mut(&mut self) -> &mut HashMap<u16, String> {
        &mut self.entries
    }

    /// Consume the set and return the underlying map.
    pub fn into_inner(self) -> HashMap<u16, String> {
        self.entries
    }
}

impl From<HashMap<u16, String>> for ClueSet {
    fn from(entries: HashMap<u16, String>) -> Self {
        Self { entries }
    }
}

impl FromIterator<(u16, String)> for ClueSet {
    fn from_iter<I: IntoIterator<Item = (u16, String)>>(iter: I) -> Self {
        Self {
            entries: iter.into_iter().collect(),
        }
    }
}

/// Clues organized by direction and number.
///
/// Clue numbers correspond to the starting squares in the grid. Each direction
/// is a [`ClueSet`], so reading and writing a single clue reads naturally:
///
/// ```rust
/// use puz_parse::Clues;
///
/// let mut clues = Clues::default();
/// clues.across.set(1, "First across");
/// clues.down.set(2, "Second down");
///
/// assert_eq!(clues.across.get(1), Some("First across"));
/// for (number, text) in clues.across.iter() {
///     println!("{number}A. {text}");
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "json", derive(serde::Serialize, serde::Deserialize))]
pub struct Clues {
    /// Across clues, keyed by clue number.
    pub across: ClueSet,
    /// Down clues, keyed by clue number.
    pub down: ClueSet,
}

impl Clues {
    /// Assemble clues from an across and a down [`ClueSet`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use puz_parse::{Clues, ClueSet};
    ///
    /// let clues = Clues::new(
    ///     ClueSet::new([(1, "First across"), (3, "Third across")]),
    ///     ClueSet::new([(1, "First down"), (2, "Second down")]),
    /// );
    /// assert_eq!(clues.across.get(3), Some("Third across"));
    /// ```
    pub fn new(across: ClueSet, down: ClueSet) -> Self {
        Self { across, down }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let mut clues = Clues::default();
        assert_eq!(clues.across.set(1, "First across"), None);
        assert_eq!(clues.down.set(2, "Second down"), None);

        assert_eq!(clues.across.get(1), Some("First across"));
        assert_eq!(clues.down.get(2), Some("Second down"));
        // A number with no clue returns None, and directions are independent.
        assert_eq!(clues.down.get(1), None);
        assert_eq!(clues.across.get(2), None);
    }

    #[test]
    fn test_set_overwrites_and_returns_previous() {
        let mut clues = Clues::default();
        clues.across.set(1, "old");
        assert_eq!(clues.across.set(1, "new"), Some("old".to_string()));
        assert_eq!(clues.across.get(1), Some("new"));
    }

    #[test]
    fn test_remove_and_contains() {
        let mut clues = Clues::default();
        clues.down.set(3, "gone");
        assert!(clues.down.contains(3));
        assert_eq!(clues.down.remove(3), Some("gone".to_string()));
        assert_eq!(clues.down.remove(3), None);
        assert!(!clues.down.contains(3));
        assert_eq!(clues.down.get(3), None);
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut clues = Clues::default();
        assert!(clues.across.is_empty());
        assert_eq!(clues.across.len(), 0);

        clues.across.set(1, "a");
        clues.across.set(3, "b");
        assert_eq!(clues.across.len(), 2);
        assert!(!clues.across.is_empty());
        // The other direction is independent.
        assert!(clues.down.is_empty());
    }

    #[test]
    fn test_iter_is_ascending_by_number() {
        let mut clues = Clues::default();
        clues.across.set(5, "five");
        clues.across.set(1, "one");
        clues.across.set(3, "three");

        let ordered: Vec<(u16, &str)> = clues.across.iter().collect();
        assert_eq!(ordered, vec![(1, "one"), (3, "three"), (5, "five")]);
        // Down is empty, so its iterator yields nothing.
        assert_eq!(clues.down.iter().count(), 0);
    }

    #[test]
    fn test_raw_map_access() {
        let mut map = HashMap::new();
        map.insert(1u16, "one".to_string());
        let set = ClueSet::from(map);
        assert_eq!(set.get(1), Some("one"));
        assert_eq!(set.as_map().len(), 1);
        assert_eq!(set.into_inner().get(&1).map(String::as_str), Some("one"));
    }
}
