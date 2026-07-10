//! Bulk-validate a directory of `.puz` files.

use anyhow::Result;
use clap::Args;
use puz_parse::{PuzWarning, Puzzle};
use std::path::{Path, PathBuf};

#[derive(Args)]
pub(crate) struct ValidateArgs {
    /// Directory to scan recursively for `.puz` files.
    #[arg(value_name = "DIR")]
    dir: PathBuf,

    /// Print a line for every file, including clean ones.
    #[arg(long)]
    verbose: bool,

    /// Print only hard parse failures, not warnings.
    #[arg(long)]
    errors_only: bool,
}

pub(crate) fn run(args: ValidateArgs) -> Result<()> {
    let mut files = Vec::new();
    collect_puz_files(&args.dir, &mut files);
    files.sort();

    if files.is_empty() {
        anyhow::bail!("no .puz files found under {}", args.dir.display());
    }

    let mut parse_errors = 0usize;
    let mut files_with_warnings = 0usize;
    let mut checksum_mismatches = 0usize;
    let mut other_warnings = 0usize;

    for path in &files {
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                parse_errors += 1;
                println!("READ-ERR {}: {e}", path.display());
                continue;
            }
        };

        match Puzzle::reader().from_bytes_verbose(&data) {
            Ok(parsed) => {
                if parsed.warnings.is_empty() {
                    if args.verbose {
                        println!("OK       {}", path.display());
                    }
                } else {
                    files_with_warnings += 1;
                    for w in &parsed.warnings {
                        match w {
                            PuzWarning::ChecksumMismatch { .. } => checksum_mismatches += 1,
                            _ => other_warnings += 1,
                        }
                        if !args.errors_only {
                            println!("WARN     {}: {w}", path.display());
                        }
                    }
                }
            }
            Err(e) => {
                parse_errors += 1;
                println!("PARSE-ERR {}: {e}", path.display());
            }
        }
    }

    let total = files.len();
    let clean = total - parse_errors - files_with_warnings;
    println!("\n=== summary ===");
    println!("scanned:            {total}");
    println!("parse errors:       {parse_errors}");
    println!("files w/ warnings:  {files_with_warnings}");
    println!("  checksum mismatch:  {checksum_mismatches}");
    println!("  other warnings:     {other_warnings}");
    println!("clean:              {clean}");

    Ok(())
}

/// Recursively collect all `.puz` files under `dir`.
fn collect_puz_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_puz_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("puz") {
            out.push(path);
        }
    }
}
