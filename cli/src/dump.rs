//! Dump the raw structure of a single `.puz` file.
//!
//! Reads the file bytes via [`puz_parse::raw`] rather than parsing, so these
//! commands still produce useful output for files that fail to parse.

use anyhow::{Context, Result};
use clap::Subcommand;
use comfy_table::{Cell, CellAlignment, Table, presets::UTF8_FULL};
use owo_colors::OwoColorize;
use puz_parse::raw;
use std::path::PathBuf;

#[derive(Subcommand)]
pub(crate) enum DumpKind {
    /// Declared dimensions, clue count, bitmask, version, and scrambled tag.
    Header {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// The solution and blank grids, with any black-square mismatches.
    Grid {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Title, author, copyright, the numbered clue list, and notes.
    Strings {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

pub(crate) fn run(what: DumpKind) -> Result<()> {
    match what {
        DumpKind::Header { file } => dump_header(&file),
        DumpKind::Grid { file } => dump_grid(&file),
        DumpKind::Strings { file } => dump_strings(&file),
    }
}

fn read_file(path: &PathBuf) -> Result<Vec<u8>> {
    std::fs::read(path).with_context(|| format!("failed to read {}", path.display()))
}

/// A two-column key/value table with no outer borders.
fn kv_table() -> Table {
    let mut table = Table::new();
    table.load_preset(comfy_table::presets::NOTHING);
    table
}

fn dump_header(path: &PathBuf) -> Result<()> {
    let data = read_file(path)?;
    let header = raw::read_header(&data)
        .with_context(|| format!("{} is too short for a .puz header", path.display()))?;

    println!("{}", path.display().bold());
    let mut table = kv_table();
    table
        .add_row(vec!["size", &format!("{} bytes", data.len())])
        .add_row(vec!["width", &header.width.to_string()])
        .add_row(vec!["height", &header.height.to_string()])
        .add_row(vec!["num_clues", &header.num_clues.to_string()])
        .add_row(vec!["version", &header.version])
        .add_row(vec!["bitmask", &format!("0x{:04X}", header.bitmask)])
        .add_row(vec![
            "scrambled",
            &format!("0x{:04X}", header.scrambled_tag),
        ]);
    println!("{table}");
    Ok(())
}

fn dump_grid(path: &PathBuf) -> Result<()> {
    let data = read_file(path)?;
    let grids = raw::read_grids(&data)
        .with_context(|| format!("{} is too short for its declared grids", path.display()))?;

    println!(
        "{} {}",
        path.display().bold(),
        format!("{}x{}", grids.width, grids.height).dimmed()
    );

    print_grid("solution", &grids.solution);
    print_grid("blank", &grids.blank);

    let mismatches = grids.black_square_mismatches();
    if mismatches.is_empty() {
        println!("{}", "black squares: consistent".green());
    } else {
        println!(
            "{}",
            format!("black-square mismatches: {} cell(s)", mismatches.len()).yellow()
        );
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec!["row", "col", "solution", "blank"]);
        for m in mismatches.iter().take(16) {
            table.add_row(vec![
                Cell::new(m.row).set_alignment(CellAlignment::Right),
                Cell::new(m.col).set_alignment(CellAlignment::Right),
                Cell::new(byte_repr(m.solution)),
                Cell::new(byte_repr(m.blank)),
            ]);
        }
        println!("{table}");
        if mismatches.len() > 16 {
            println!("{}", format!("... {} more", mismatches.len() - 16).dimmed());
        }
    }

    println!(
        "{} {}",
        "solution bytes:".dimmed(),
        unique_bytes(&grids.solution)
    );
    println!("{} {}", "blank bytes:".dimmed(), unique_bytes(&grids.blank));
    Ok(())
}

fn dump_strings(path: &PathBuf) -> Result<()> {
    let data = read_file(path)?;
    let strings = raw::read_strings(&data)
        .with_context(|| format!("{} is too short for its string section", path.display()))?;

    let mut meta = kv_table();
    meta.add_row(vec!["title", &strings.title])
        .add_row(vec!["author", &strings.author])
        .add_row(vec!["copyright", &strings.copyright])
        .add_row(vec!["notes", &strings.notes]);
    println!("{meta}");

    println!(
        "{}",
        format!("--- {} clues ---", strings.clues.len()).bold()
    );
    let mut table = Table::new();
    table.load_preset(comfy_table::presets::NOTHING);
    for (i, clue) in strings.clues.iter().enumerate() {
        table.add_row(vec![
            Cell::new(i + 1).set_alignment(CellAlignment::Right),
            Cell::new(clue),
        ]);
    }
    println!("{table}");
    Ok(())
}

fn print_grid(label: &str, grid: &[Vec<u8>]) {
    println!("{}", format!("{label}:").dimmed());
    for (i, row) in grid.iter().enumerate() {
        let rendered: String = row.iter().map(|&b| render_cell(b)).collect();
        println!("  {i:>2} {rendered}");
    }
}

fn render_cell(b: u8) -> char {
    if (0x20..0x7f).contains(&b) {
        b as char
    } else {
        '?'
    }
}

fn byte_repr(b: u8) -> String {
    format!("'{}' (0x{b:02X})", render_cell(b))
}

fn unique_bytes(grid: &[Vec<u8>]) -> String {
    let mut seen: Vec<u8> = grid.iter().flatten().copied().collect();
    seen.sort_unstable();
    seen.dedup();
    seen.iter()
        .map(|&b| format!("{}({b:02X})", render_cell(b)))
        .collect::<Vec<_>>()
        .join(" ")
}
