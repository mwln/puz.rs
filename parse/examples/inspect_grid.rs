//! Dump the raw grid bytes of one or more `.puz` files.
//!
//! Reads the header and both grids directly from the file bytes (so it works
//! even when parsing fails), then prints:
//!   - declared width/height, clue count, and the header bitmask,
//!   - the solution and blank grids as character rows,
//!   - any cells where the two grids disagree about black squares,
//!   - the distinct byte values present in each grid.
//!
//! This is a debugging aid for files that fail to parse with grid or clue
//! errors (e.g. "blocked squares don't match" or clue-count mismatches).
//!
//! Usage:
//!     cargo run --release --example inspect_grid -- <FILE> [<FILE> ...]

use std::path::PathBuf;

/// Header offsets in a `.puz` file.
const WIDTH: usize = 0x2C;
const HEIGHT: usize = 0x2D;
const NUM_CLUES: usize = 0x2E;
const BITMASK: usize = 0x30;
const GRID_START: usize = 0x34;

fn main() {
    let files: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
    if files.is_empty() {
        eprintln!("usage: inspect_grid <FILE> [<FILE> ...]");
        std::process::exit(2);
    }

    for path in &files {
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                println!("READ-ERR {}: {e}", path.display());
                continue;
            }
        };

        println!("=== {} ({} bytes) ===", path.display(), data.len());
        if data.len() < GRID_START {
            println!("  too short for a header");
            continue;
        }

        let width = data[WIDTH] as usize;
        let height = data[HEIGHT] as usize;
        let num_clues = u16::from_le_bytes([data[NUM_CLUES], data[NUM_CLUES + 1]]);
        let bitmask = u16::from_le_bytes([data[BITMASK], data[BITMASK + 1]]);
        let board = width * height;
        println!(
            "  {width}x{height} = {board} cells, num_clues={num_clues}, bitmask=0x{bitmask:04X}"
        );

        let sol_end = GRID_START + board;
        let fill_end = sol_end + board;
        if fill_end > data.len() {
            println!("  file too short for two {board}-byte grids");
            continue;
        }
        let solution = &data[GRID_START..sol_end];
        let blank = &data[sol_end..fill_end];

        print_grid("solution", solution, width);
        print_grid("blank", blank, width);

        // Cells where exactly one grid has a black square ('.').
        let mut mismatches = Vec::new();
        for i in 0..board {
            let s = solution[i];
            let f = blank[i];
            if (s == b'.') != (f == b'.') {
                mismatches.push((i / width, i % width, s, f));
            }
        }
        if mismatches.is_empty() {
            println!("  black squares: consistent");
        } else {
            println!("  black-square mismatches: {} cell(s)", mismatches.len());
            for (r, c, s, f) in mismatches.iter().take(16) {
                println!(
                    "    ({r},{c}) solution={} / blank={}",
                    byte_repr(*s),
                    byte_repr(*f)
                );
            }
            if mismatches.len() > 16 {
                println!("    ... {} more", mismatches.len() - 16);
            }
        }

        println!("  solution bytes: {}", unique_bytes(solution));
        println!("  blank bytes:    {}", unique_bytes(blank));
        println!();
    }
}

/// Print a grid as one line per row, rendering each cell byte as a character.
fn print_grid(label: &str, grid: &[u8], width: usize) {
    println!("  {label}:");
    for (i, row) in grid.chunks(width).enumerate() {
        let rendered: String = row.iter().map(|&b| render_cell(b)).collect();
        println!("    {i:2} {rendered}");
    }
}

/// Render one cell byte for the grid view: printable ASCII as-is, else '?'.
fn render_cell(b: u8) -> char {
    if (0x20..0x7f).contains(&b) {
        b as char
    } else {
        '?'
    }
}

/// Describe a single byte as `'X' (0xNN)`.
fn byte_repr(b: u8) -> String {
    format!("'{}' (0x{b:02X})", render_cell(b))
}

/// Distinct byte values in a grid, sorted, each shown as char + hex.
fn unique_bytes(grid: &[u8]) -> String {
    let mut seen: Vec<u8> = grid.to_vec();
    seen.sort_unstable();
    seen.dedup();
    seen.iter()
        .map(|&b| format!("{}({b:02X})", render_cell(b)))
        .collect::<Vec<_>>()
        .join(" ")
}
