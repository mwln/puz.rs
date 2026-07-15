//! Dump the raw structure of a single `.puz` file.
//!
//! Reads the file bytes via [`puz_parse::raw`] rather than parsing, so these
//! commands still produce useful output for files that fail to parse.

use anyhow::{Context, Result};
use clap::Subcommand;
use comfy_table::{Cell, CellAlignment};
use puz_parse::raw;
use std::path::PathBuf;

use crate::render;

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
    /// The clue numbering our geometry computes, cross-checked against the
    /// file's declared clue count and provided clue strings.
    Clues {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

pub(crate) fn run(what: DumpKind) -> Result<()> {
    match what {
        DumpKind::Header { file } => dump_header(&file),
        DumpKind::Grid { file } => dump_grid(&file),
        DumpKind::Strings { file } => dump_strings(&file),
        DumpKind::Clues { file } => dump_clues(&file),
    }
}

fn read_file(path: &PathBuf) -> Result<Vec<u8>> {
    std::fs::read(path).with_context(|| format!("failed to read {}", path.display()))
}

fn dump_header(path: &PathBuf) -> Result<()> {
    let data = read_file(path)?;
    let header = raw::read_header(&data)
        .with_context(|| format!("{} is too short for a .puz header", path.display()))?;

    println!("{}", render::bold(path.display()));
    let mut table = render::borderless_table();
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
        render::bold(path.display()),
        render::dim(format!("{}x{}", grids.width, grids.height))
    );

    render::print_grid("solution", &grids.solution);
    render::print_grid("blank", &grids.blank);

    let mismatches = grids.black_square_mismatches();
    if mismatches.is_empty() {
        println!("{}", render::green("black squares: consistent"));
    } else {
        println!(
            "{}",
            render::yellow(format!(
                "black-square mismatches: {} cell(s)",
                mismatches.len()
            ))
        );
        let mut table = render::bordered_table();
        table.set_header(vec!["row", "col", "solution", "blank"]);
        for m in mismatches.iter().take(16) {
            table.add_row(vec![
                Cell::new(m.row).set_alignment(CellAlignment::Right),
                Cell::new(m.col).set_alignment(CellAlignment::Right),
                Cell::new(render::byte_repr(m.solution)),
                Cell::new(render::byte_repr(m.blank)),
            ]);
        }
        println!("{table}");
        if mismatches.len() > 16 {
            println!(
                "{}",
                render::dim(format!("... {} more", mismatches.len() - 16))
            );
        }
    }

    println!(
        "{} {}",
        render::dim("solution bytes:"),
        render::unique_bytes(&grids.solution)
    );
    println!(
        "{} {}",
        render::dim("blank bytes:"),
        render::unique_bytes(&grids.blank)
    );
    Ok(())
}

fn dump_strings(path: &PathBuf) -> Result<()> {
    let data = read_file(path)?;
    let strings = raw::read_strings(&data)
        .with_context(|| format!("{} is too short for its string section", path.display()))?;

    let mut meta = render::borderless_table();
    meta.add_row(vec!["title", &strings.title])
        .add_row(vec!["author", &strings.author])
        .add_row(vec!["copyright", &strings.copyright])
        .add_row(vec!["notes", &strings.notes]);
    println!("{meta}");

    println!(
        "{}",
        render::bold(format!("--- {} clues ---", strings.clues.len()))
    );
    let mut table = render::borderless_table();
    for (i, clue) in strings.clues.iter().enumerate() {
        table.add_row(vec![
            Cell::new(i + 1).set_alignment(CellAlignment::Right),
            Cell::new(clue),
        ]);
    }
    println!("{table}");
    Ok(())
}

fn dump_clues(path: &PathBuf) -> Result<()> {
    let data = read_file(path)?;
    let header = raw::read_header(&data)
        .with_context(|| format!("{} is too short for a .puz header", path.display()))?;
    let grids = raw::read_grids(&data)
        .with_context(|| format!("{} is too short for its declared grids", path.display()))?;
    let strings = raw::read_strings(&data);

    let numbers = grids.clue_numbers();
    let (across, down) = grids.clue_counts();
    let geometric = across + down;
    let declared = header.num_clues as usize;

    println!("{}", render::bold(path.display()));
    let mut summary = render::borderless_table();
    summary
        .add_row(vec!["across slots", &across.to_string()])
        .add_row(vec!["down slots", &down.to_string()])
        .add_row(vec!["geometric total", &geometric.to_string()])
        .add_row(vec!["declared num_clues", &declared.to_string()]);
    if let Some(s) = &strings {
        summary.add_row(vec!["clue strings in file", &s.clues.len().to_string()]);
        let placeholders = s.clues.iter().filter(|c| is_placeholder(c)).count();
        summary.add_row(vec![
            "placeholder ('-'/empty) clues",
            &placeholders.to_string(),
        ]);
    }
    println!("{summary}");

    if geometric == declared {
        println!("{}", render::green("geometry matches declared clue count"));
    } else {
        println!(
            "{}",
            render::yellow(format!(
                "MISMATCH: geometry {geometric} vs declared {declared}"
            ))
        );
    }

    // Numbered cells alongside the clue text the file provides in reading order.
    let clues = strings.as_ref().map(|s| &s.clues);
    let mut table = render::borderless_table();
    table.set_header(vec!["num", "cell", "dir", "clue"]);
    let mut clue_idx = 0usize;
    for cell in &numbers {
        let dirs = match (cell.across, cell.down) {
            (true, true) => "A+D",
            (true, false) => "A",
            (false, true) => "D",
            (false, false) => "-",
        };
        // Consume one clue string per slot (across first, then down) in reading
        // order, matching how .puz lays out the clue list.
        let mut texts = Vec::new();
        if cell.across {
            texts.push(nth_clue(clues, clue_idx));
            clue_idx += 1;
        }
        if cell.down {
            texts.push(nth_clue(clues, clue_idx));
            clue_idx += 1;
        }
        table.add_row(vec![
            Cell::new(cell.number).set_alignment(CellAlignment::Right),
            Cell::new(format!("({},{})", cell.row, cell.col)),
            Cell::new(dirs),
            Cell::new(texts.join("  |  ")),
        ]);
    }
    println!("{table}");

    if let Some(s) = &strings
        && clue_idx < s.clues.len()
    {
        println!(
            "{}",
            render::yellow(format!(
                "{} extra clue string(s) in file beyond geometric slots:",
                s.clues.len() - clue_idx
            ))
        );
        for (i, extra) in s.clues[clue_idx..].iter().enumerate() {
            println!("  [{}] {}", clue_idx + i, render_clue(extra));
        }
    }
    Ok(())
}

fn nth_clue(clues: Option<&Vec<String>>, idx: usize) -> String {
    match clues.and_then(|c| c.get(idx)) {
        Some(c) => render_clue(c),
        None => "<missing>".to_string(),
    }
}

fn render_clue(c: &str) -> String {
    if is_placeholder(c) {
        format!("<placeholder {c:?}>")
    } else {
        c.to_string()
    }
}

fn is_placeholder(c: &str) -> bool {
    c.trim().is_empty() || c.trim() == "-"
}
