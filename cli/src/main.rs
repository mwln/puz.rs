//! `puz`: parse and inspect `.puz` crossword puzzle files.

mod commands;
mod render;

use anyhow::Result;
use clap::{Parser, Subcommand};

use commands::{dump, inspect, parse_json, validate};

#[derive(Parser)]
#[command(
    name = "puz",
    version,
    about = "parse and inspect .puz crossword puzzle files",
    // Allow `puz file.puz ...` with no subcommand to parse to JSON, preserving
    // the original behavior.
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Files to parse to JSON (when no subcommand is given).
    #[arg(value_name = "PUZZLE", num_args = 1..)]
    files: Vec<String>,

    /// Write JSON output to a file instead of stdout.
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    /// Pretty-print the JSON output.
    #[arg(short, long)]
    pretty: bool,

    /// For a single file, output the puzzle object directly (not in an array).
    #[arg(short, long)]
    single: bool,

    /// Disable colored and Unicode-styled output (also honors `NO_COLOR`).
    #[arg(long, global = true)]
    no_color: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Parse puzzles and output JSON (same as running `puz FILES...`).
    Parse(parse_json::ParseArgs),

    /// Bulk-validate every `.puz` file under a directory.
    Validate(validate::ValidateArgs),

    /// Print raw structure of a single file (works even if it fails to parse).
    Dump {
        #[command(subcommand)]
        what: dump::DumpKind,
    },

    /// Inspect a single file's extension sections.
    Inspect {
        #[command(subcommand)]
        what: inspect::InspectKind,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    render::init_styling(cli.no_color);

    match cli.command {
        Some(Command::Parse(args)) => parse_json::run(args),
        Some(Command::Validate(args)) => validate::run(args),
        Some(Command::Dump { what }) => dump::run(what),
        Some(Command::Inspect { what }) => inspect::run(what),
        None => {
            // Bare `puz FILES...` behaves like `puz parse FILES...`.
            parse_json::run(parse_json::ParseArgs {
                files: cli.files,
                output: cli.output,
                pretty: cli.pretty,
                single: cli.single,
            })
        }
    }
}
