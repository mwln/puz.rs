//! Export clue/answer pairs from a directory of `.puz` files as JSON Lines.

use anyhow::Result;
use clap::Args;
use puz_parse::Puzzle;
use serde::Serialize;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use crate::commands::collect_puz_files;

#[derive(Args)]
pub(crate) struct ExportArgs {
    /// directory to scan recursively for .puz files
    #[arg(value_name = "DIR")]
    dir: PathBuf,
}

/// One clue/answer occurrence, with the metadata reliably found in the file.
///
/// Outlet and date are intentionally omitted: they are inconsistent inside
/// `.puz` files and usually live in the directory layout instead. `file` is
/// emitted so a consumer can derive those downstream.
#[derive(Serialize)]
struct Row<'a> {
    file: &'a str,
    title: &'a str,
    author: &'a str,
    direction: puz_parse::Direction,
    number: u16,
    clue: &'a str,
    answer: &'a str,
}

pub(crate) fn run(args: ExportArgs) -> Result<()> {
    let mut files = collect_puz_files(&args.dir);
    files.sort();
    if files.is_empty() {
        anyhow::bail!("no .puz files found under {}", args.dir.display());
    }

    // Buffer stdout: this can emit millions of lines across a large corpus.
    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    let mut exported = 0usize;
    let mut skipped = 0usize;
    for path in &files {
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("skip {}: {e}", path.display());
                skipped += 1;
                continue;
            }
        };
        // Parse leniently; warnings (checksums, extra clues, ...) don't block
        // extracting clues and answers.
        let puzzle = match Puzzle::from_bytes(&data) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("skip {}: {e}", path.display());
                skipped += 1;
                continue;
            }
        };

        let file = path.to_string_lossy();
        for entry in puzzle.clue_answers() {
            let row = Row {
                file: &file,
                title: &puzzle.info.title,
                author: &puzzle.info.author,
                direction: entry.direction,
                number: entry.number,
                clue: &entry.clue,
                answer: &entry.answer,
            };
            match write_row(&mut out, &row) {
                Ok(()) => exported += 1,
                // A closed downstream pipe (e.g. `puz export ... | head`) is a
                // normal way to stop; exit cleanly instead of erroring.
                Err(e) if is_broken_pipe(&e) => return Ok(()),
                Err(e) => return Err(e.into()),
            }
        }
    }

    if let Err(e) = out.flush()
        && !is_broken_pipe(&e)
    {
        return Err(e.into());
    }
    eprintln!(
        "exported {exported} clue/answer rows from {} files ({skipped} skipped)",
        files.len() - skipped
    );
    Ok(())
}

fn write_row<W: Write>(out: &mut W, row: &Row) -> std::io::Result<()> {
    serde_json::to_writer(&mut *out, row).map_err(std::io::Error::from)?;
    out.write_all(b"\n")
}

fn is_broken_pipe(e: &std::io::Error) -> bool {
    e.kind() == std::io::ErrorKind::BrokenPipe
}
