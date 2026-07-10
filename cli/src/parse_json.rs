//! The default command: parse `.puz` files and emit JSON.

use anyhow::{Context, Result};
use clap::Args;
use puz_parse::Puzzle;
use std::io::{self, Write};
use std::path::Path;

#[derive(Args)]
pub(crate) struct ParseArgs {
    /// One or more `.puz` files to parse.
    #[arg(value_name = "PUZZLE", required = true, num_args = 1..)]
    pub(crate) files: Vec<String>,

    /// Write output to a file instead of stdout.
    #[arg(short, long, value_name = "FILE")]
    pub(crate) output: Option<String>,

    /// Pretty-print the JSON output.
    #[arg(short, long)]
    pub(crate) pretty: bool,

    /// For a single file, output the puzzle object directly (not in an array).
    #[arg(short, long)]
    pub(crate) single: bool,
}

#[derive(serde::Serialize)]
struct PuzzleResult {
    file: String,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    puzzle: Option<Puzzle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

pub(crate) fn run(args: ParseArgs) -> Result<()> {
    let mut results = Vec::new();

    for file_path in &args.files {
        match process_file(file_path) {
            Ok(puzzle) => results.push(PuzzleResult {
                file: file_path.clone(),
                success: true,
                puzzle: Some(puzzle),
                error: None,
            }),
            Err(e) => {
                eprintln!("Error processing {file_path}: {e}");
                results.push(PuzzleResult {
                    file: file_path.clone(),
                    success: false,
                    puzzle: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    let output_data = if args.single && results.len() == 1 {
        let result = results.into_iter().next().expect("len checked above");
        if result.success {
            serde_json::to_value(result.puzzle.expect("success implies puzzle"))?
        } else {
            serde_json::to_value(result)?
        }
    } else {
        serde_json::to_value(results)?
    };

    let json_output = if args.pretty {
        serde_json::to_string_pretty(&output_data)?
    } else {
        serde_json::to_string(&output_data)?
    };

    match args.output.as_deref() {
        Some(path) => {
            std::fs::write(path, json_output)
                .with_context(|| format!("Failed to write to {path}"))?;
        }
        None => {
            io::stdout()
                .write_all(json_output.as_bytes())
                .context("Failed to write to stdout")?;
            println!();
        }
    }

    Ok(())
}

fn process_file(path: &str) -> Result<Puzzle> {
    let file_path = Path::new(path);
    if !file_path.exists() {
        anyhow::bail!("File does not exist: {}", path);
    }
    if !file_path.is_file() {
        anyhow::bail!("Path is not a file: {}", path);
    }

    let parsed = Puzzle::reader()
        .from_file_verbose(path)
        .with_context(|| format!("Failed to parse .puz file: {path}"))?;

    for warning in &parsed.warnings {
        eprintln!("Warning in {path}: {warning}");
    }

    Ok(parsed.result)
}
