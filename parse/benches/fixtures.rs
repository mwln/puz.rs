//! In-code fixture generation for benchmarks.
//!
//! Rather than bundling `.puz` files, we build `Puzzle` values programmatically
//! and use the library's own writer (`to_bytes`) to produce the byte buffers we
//! parse. This keeps benchmarks focused on core-library performance (no file
//! I/O, no bundled fixtures) and lets us vary size and features freely.
//!
//! Note: benchmarks only *time* code, they don't assert correctness, so using
//! the writer to generate parser inputs is fine here — a shared bug would show
//! up in the correctness test suite, not silently pass a benchmark.

use puz_parse::{Clues, Extensions, Grid, Puzzle, PuzzleInfo, Rebus};
use std::collections::HashMap;

/// A generated puzzle plus its serialized bytes, ready for benchmarking either
/// direction.
#[derive(Debug)]
pub(crate) struct Fixture {
    pub(crate) name: String,
    pub(crate) puzzle: Puzzle,
    pub(crate) bytes: Vec<u8>,
}

/// Build the standard set of benchmark fixtures: a size sweep plus
/// feature-specific variants (rebus, circles, given).
pub(crate) fn all() -> Vec<Fixture> {
    let mut fixtures = Vec::new();

    // Size sweep on plain (all-open) grids.
    for (name, size) in [
        ("small_5x5", 5),
        ("standard_15x15", 15),
        ("large_21x21", 21),
    ] {
        fixtures.push(build(name, plain(size)));
    }

    // Feature variants on a standard-sized grid.
    fixtures.push(build("rebus_15x15", with_rebus(plain(15))));
    fixtures.push(build("circles_15x15", with_circles(plain(15))));
    fixtures.push(build("given_15x15", with_given(plain(15))));

    fixtures
}

/// Serialize a puzzle into a `Fixture`, panicking if the writer rejects it
/// (which would indicate a bug in the generator).
fn build(name: &str, puzzle: Puzzle) -> Fixture {
    let bytes = puz_parse::to_bytes(&puzzle)
        .unwrap_or_else(|e| panic!("fixture {name} failed to serialize: {e}"));
    Fixture {
        name: name.to_string(),
        puzzle,
        bytes,
    }
}

/// Build a plain `size`x`size` puzzle with no black squares and every implied
/// clue filled in. An open grid's numbering is simple: each row starts one
/// across word and each column starts one down word, numbered in reading order.
fn plain(size: usize) -> Puzzle {
    let mut solution = Vec::with_capacity(size);
    for row in 0..size {
        let cells: String = (0..size).map(|col| letter(row, col)).collect();
        solution.push(cells);
    }
    let blank: Vec<String> = (0..size).map(|_| "-".repeat(size)).collect();

    let clues = clues_for_open_grid(size);

    Puzzle {
        info: PuzzleInfo {
            title: "Benchmark".into(),
            author: "Generated".into(),
            copyright: "(c) puz.rs".into(),
            notes: String::new(),
            width: size as u8,
            height: size as u8,
            version: "1.3".into(),
            is_scrambled: false,
            is_diagramless: false,
        },
        grid: Grid { solution, blank },
        clues,
        extensions: Extensions {
            rebus: None,
            circles: None,
            given: None,
        },
    }
}

/// Generate the exact set of clues an all-open `size`x`size` grid requires.
///
/// Walking in reading order, a cell starts an across word when it is in the
/// first column, and a down word when it is in the first row. Each numbered
/// cell advances the clue number once.
fn clues_for_open_grid(size: usize) -> Clues {
    let mut across = HashMap::new();
    let mut down = HashMap::new();
    let mut number = 1u16;

    for row in 0..size {
        for col in 0..size {
            let starts_across = col == 0;
            let starts_down = row == 0;
            if !(starts_across || starts_down) {
                continue;
            }
            if starts_across {
                across.insert(number, format!("across {number}"));
            }
            if starts_down {
                down.insert(number, format!("down {number}"));
            }
            number += 1;
        }
    }

    Clues { across, down }
}

/// A reproducible letter for the cell at `(row, col)`.
///
/// Benchmarks need deterministic, non-uniform grid content: fixed so runs are
/// comparable, but varied so a grid of identical bytes doesn't hand the parser
/// or checksum an unrealistic fast path. The exact scramble is unimportant — we
/// just spread letters across the alphabet.
fn letter(row: usize, col: usize) -> char {
    const ALPHABET: &[u8; 26] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    ALPHABET[(row * 7 + col * 3) % ALPHABET.len()] as char
}

/// Add a single rebus cell (top-left) with a one-entry solution table.
fn with_rebus(mut puzzle: Puzzle) -> Puzzle {
    let width = puzzle.info.width as usize;
    let height = puzzle.info.height as usize;

    let mut rebus_grid = vec![vec![0u8; width]; height];
    rebus_grid[0][0] = 1; // rebus key 1 at the top-left cell

    let mut table = HashMap::new();
    table.insert(1u8, "HEART".to_string());

    puzzle.extensions.rebus = Some(Rebus {
        grid: rebus_grid,
        table,
    });
    puzzle
}

/// Circle the cells along the main diagonal.
fn with_circles(mut puzzle: Puzzle) -> Puzzle {
    let width = puzzle.info.width as usize;
    let height = puzzle.info.height as usize;

    let mut circled = vec![vec![false; width]; height];
    for (row, cells) in circled.iter_mut().enumerate() {
        // A cell is on the main diagonal when its column equals its row.
        if row < width {
            cells[row] = true;
        }
    }

    puzzle.extensions.circles = Some(circled);
    puzzle
}

/// Mark the first row of cells as "given" (pre-filled for the solver).
fn with_given(mut puzzle: Puzzle) -> Puzzle {
    let width = puzzle.info.width as usize;
    let height = puzzle.info.height as usize;

    let mut given = vec![vec![false; width]; height];
    if let Some(first_row) = given.first_mut() {
        first_row.fill(true);
    }

    puzzle.extensions.given = Some(given);
    puzzle
}
