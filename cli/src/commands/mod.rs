//! Subcommand implementations for the `puz` CLI.
//!
//! Each module implements one top-level command: it defines the command's
//! clap arguments and a `run` entry point. `main` dispatches to these; shared
//! presentation lives in [`crate::render`].

pub(crate) mod dump;
pub(crate) mod export;
pub(crate) mod inspect;
pub(crate) mod parse_json;
pub(crate) mod validate;

use std::path::{Path, PathBuf};

/// Recursively collect every `.puz` file under `dir`, in no particular order.
///
/// Shared by the directory-walking commands (`validate`, `export`).
pub(crate) fn collect_puz_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_into(dir, &mut out);
    out
}

fn collect_into(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        // Use the directory entry's file type, which comes from `readdir` and
        // avoids an extra `stat` syscall per entry (the old `path.is_dir()`
        // stat'd all 48k+ files). Fall back to a stat only if the type is
        // unknown (rare, e.g. some network filesystems).
        let is_dir = match entry.file_type() {
            Ok(ft) => ft.is_dir(),
            Err(_) => entry.path().is_dir(),
        };
        let path = entry.path();
        if is_dir {
            collect_into(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("puz") {
            out.push(path);
        }
    }
}
