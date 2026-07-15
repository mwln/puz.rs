//! Shared presentation helpers for the CLI.
//!
//! Centralizes how the `puz` commands render output: table construction,
//! grid/byte formatting, and the small set of styling conventions used across
//! `dump` and `inspect`. Command modules extract data (mostly via
//! [`puz_parse::raw`]) and hand it here for display.

use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};

use comfy_table::{Table, presets};
use owo_colors::OwoColorize;

/// Whether styled output (ANSI colors + Unicode table borders) is enabled.
/// Set once at startup by [`init_styling`]; defaults to on.
static STYLED: AtomicBool = AtomicBool::new(true);

/// Decide whether to use styled output and configure coloring accordingly.
///
/// Styling is disabled when `--no-color` is passed, when the `NO_COLOR`
/// environment variable is set (see <https://no-color.org>), or when stdout is
/// not a terminal (e.g. piped to a file). This keeps escape codes and Unicode
/// borders out of redirected output.
pub(crate) fn init_styling(no_color_flag: bool) {
    let disabled =
        no_color_flag || std::env::var_os("NO_COLOR").is_some() || !std::io::stdout().is_terminal();
    let styled = !disabled;
    STYLED.store(styled, Ordering::Relaxed);
    owo_colors::set_override(styled);
}

fn styled() -> bool {
    STYLED.load(Ordering::Relaxed)
}

// Styling helpers. Each returns the plain string when styling is disabled, so
// callers never emit escape codes into redirected output. Direct owo-colors
// methods (`.bold()` etc.) always emit codes regardless of `set_override`, so
// styling must be gated here rather than at the call sites.

/// Bold text (plain when styling is disabled).
pub(crate) fn bold(s: impl std::fmt::Display) -> String {
    if styled() {
        s.bold().to_string()
    } else {
        s.to_string()
    }
}

/// Dimmed text (plain when styling is disabled).
pub(crate) fn dim(s: impl std::fmt::Display) -> String {
    if styled() {
        s.dimmed().to_string()
    } else {
        s.to_string()
    }
}

/// Green text (plain when styling is disabled).
pub(crate) fn green(s: impl std::fmt::Display) -> String {
    if styled() {
        s.green().to_string()
    } else {
        s.to_string()
    }
}

/// Yellow text (plain when styling is disabled).
pub(crate) fn yellow(s: impl std::fmt::Display) -> String {
    if styled() {
        s.yellow().to_string()
    } else {
        s.to_string()
    }
}

/// A borderless table (key/value metadata blocks and numbered lists).
pub(crate) fn borderless_table() -> Table {
    let mut table = Table::new();
    table.load_preset(presets::NOTHING);
    table
}

/// A bordered table with a header row (structured views such as sections and
/// grid mismatches). Uses Unicode borders when styling is enabled, plain ASCII
/// borders otherwise (e.g. when output is redirected).
pub(crate) fn bordered_table() -> Table {
    let mut table = Table::new();
    let preset = if styled() {
        presets::UTF8_FULL
    } else {
        presets::ASCII_FULL
    };
    table.load_preset(preset);
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
    println!("{}", dim(format!("{label}:")));
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
