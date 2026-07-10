//! Inspect a single `.puz` file's extension sections.

use anyhow::{Context, Result};
use clap::Subcommand;
use owo_colors::OwoColorize;
use puz_parse::raw;
use std::path::PathBuf;

use crate::render;

#[derive(Subcommand)]
pub(crate) enum InspectKind {
    /// Extension sections (GRBS, RTBL, GEXT, ...).
    Sections {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

pub(crate) fn run(what: InspectKind) -> Result<()> {
    match what {
        InspectKind::Sections { file } => inspect_sections(&file),
    }
}

fn inspect_sections(path: &PathBuf) -> Result<()> {
    let data = std::fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;

    let header = raw::read_header(&data);
    println!("{}", path.display().bold());
    if let Some(h) = &header {
        println!(
            "{}",
            format!(
                "grid {}x{} = {} cells",
                h.width,
                h.height,
                h.width as usize * h.height as usize
            )
            .dimmed()
        );
    }

    let sections = raw::scan_sections(&data);
    if sections.is_empty() {
        println!("{}", "no extension sections found".dimmed());
        return Ok(());
    }

    let board = header
        .as_ref()
        .map(|h| h.width as usize * h.height as usize)
        .unwrap_or(0);

    let mut table = render::bordered_table();
    table.set_header(vec!["section", "offset", "length", "checksum", "summary"]);
    for s in &sections {
        table.add_row(vec![
            s.tag.clone(),
            format!("0x{:X}", s.offset),
            s.length.to_string(),
            format!("0x{:04X}", s.checksum),
            summarize(s, board),
        ]);
    }
    println!("{table}");
    Ok(())
}

/// A short human summary of a section's contents.
fn summarize(section: &raw::RawSection, board: usize) -> String {
    match section.tag.as_str() {
        "GRBS" => {
            let marked = section.data.iter().filter(|&&b| b != 0).count();
            let note = if board != 0 && section.data.len() != board {
                format!(" (len {} != board {board})", section.data.len())
            } else {
                String::new()
            };
            format!("{marked} marked cell(s){note}")
        }
        "GEXT" => {
            let nz = section.data.iter().filter(|&&b| b != 0).count();
            format!("{nz} nonzero flag cell(s)")
        }
        "RTBL" | "RUSR" => {
            let text = String::from_utf8_lossy(&section.data);
            let trimmed = text.trim_end_matches('\0');
            if trimmed.len() > 60 {
                format!("{}...", &trimmed[..60])
            } else {
                trimmed.to_string()
            }
        }
        _ => format!("{} bytes", section.data.len()),
    }
}
