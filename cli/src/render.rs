//! Shared presentation helpers for the CLI.
//!
//! Centralizes how the `puz` commands render output: table construction,
//! grid/byte formatting, and the small set of styling conventions used across
//! `dump` and `inspect`. Command modules extract data (mostly via
//! [`puz_parse::raw`]) and hand it here for display.

use comfy_table::{Table, presets};
use owo_colors::OwoColorize;

/// A borderless table (key/value metadata blocks and numbered lists).
pub(crate) fn borderless_table() -> Table {
    let mut table = Table::new();
    table.load_preset(presets::NOTHING);
    table
}

/// A bordered table with a header row (structured views such as sections and
/// grid mismatches).
pub(crate) fn bordered_table() -> Table {
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table
}

/// Render one grid cell byte for display: printable ASCII as-is, else `?`.
pub(crate) fn render_cell(b: u8) -> char {
    if (0x20..0x7f).contains(&b) {
        b as char
    } else {
        '?'
    }
}

/// Describe a single byte as `'X' (0xNN)`.
pub(crate) fn byte_repr(b: u8) -> String {
    format!("'{}' (0x{b:02X})", render_cell(b))
}

/// Print a byte grid as one line per row, each cell rendered as a character,
/// under a dimmed label.
pub(crate) fn print_grid(label: &str, grid: &[Vec<u8>]) {
    println!("{}", format!("{label}:").dimmed());
    for (i, row) in grid.iter().enumerate() {
        let rendered: String = row.iter().map(|&b| render_cell(b)).collect();
        println!("  {i:>2} {rendered}");
    }
}

/// The distinct byte values in a grid, sorted, each shown as `char(hex)`.
pub(crate) fn unique_bytes(grid: &[Vec<u8>]) -> String {
    let mut seen: Vec<u8> = grid.iter().flatten().copied().collect();
    seen.sort_unstable();
    seen.dedup();
    seen.iter()
        .map(|&b| format!("{}({b:02X})", render_cell(b)))
        .collect::<Vec<_>>()
        .join(" ")
}
