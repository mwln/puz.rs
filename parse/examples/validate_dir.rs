//! Bulk-validate a directory of .puz files, reporting parse errors and
//! warnings (including checksum mismatches).
//!
//! Usage:
//!     cargo run --release --example validate_dir -- <DIR> [FLAGS]
//!
//! Flags:
//!     --verbose      print a line for every file (default: only problems)
//!     --errors-only  print only hard parse failures, not warnings

use puz_parse::PuzWarning;
use std::path::{Path, PathBuf};

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = match args.next() {
        Some(d) => PathBuf::from(d),
        None => {
            eprintln!("usage: validate_dir <DIR> [--verbose] [--errors-only]");
            std::process::exit(2);
        }
    };
    let flags: Vec<String> = args.collect();
    let verbose = flags.iter().any(|f| f == "--verbose");
    let errors_only = flags.iter().any(|f| f == "--errors-only");

    let mut files = Vec::new();
    collect_puz_files(&dir, &mut files);
    files.sort();

    if files.is_empty() {
        eprintln!("no .puz files found under {}", dir.display());
        std::process::exit(1);
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

        match puz_parse::parse(&data[..]) {
            Ok(result) => {
                if result.warnings.is_empty() {
                    if verbose {
                        println!("OK       {}", path.display());
                    }
                } else {
                    files_with_warnings += 1;
                    for w in &result.warnings {
                        match w {
                            PuzWarning::ChecksumMismatch { .. } => checksum_mismatches += 1,
                            _ => other_warnings += 1,
                        }
                        if !errors_only {
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
