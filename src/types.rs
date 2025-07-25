use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Puzzle {
    pub info: PuzzleInfo,
    pub grid: Grid,
    pub clues: Clues,
    pub extensions: Extensions,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PuzzleInfo {
    pub title: String,
    pub author: String,
    pub copyright: String,
    pub notes: String,
    pub width: u8,
    pub height: u8,
    pub version: String,
    pub is_scrambled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    pub blank: Vec<String>,
    pub solution: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clues {
    pub across: HashMap<u16, String>,
    pub down: HashMap<u16, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Extensions {
    pub rebus: Option<Rebus>,
    pub circles: Option<Vec<Vec<bool>>>,
    pub given: Option<Vec<Vec<bool>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rebus {
    pub grid: Vec<Vec<u8>>,
    pub table: HashMap<u8, String>,
}

pub(crate) const FREE_SQUARE: char = '-';
pub(crate) const TAKEN_SQUARE: char = '.';
