//! `puz`: parse and inspect `.puz` crossword puzzle files.

mod commands;
mod render;

use anyhow::Result;
use clap::{ArgAction, Parser, Subcommand};

use commands::{dump, inspect, parse_json, validate};

#[derive(Parser)]
#[command(
    name = "puz",
    version,
    about = "parse and inspect .puz crossword puzzle files",
    // Allow `puz file.puz ...` with no subcommand to parse to JSON, preserving
    // the original behavior.
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true,
    // Override clap's built-in flags so their help text matches our lowercase
    // style ("print help" instead of "Print help").
    disable_help_flag = true,
    disable_version_flag = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// print help
    #[arg(short, long, action = ArgAction::Help, global = true)]
    help: Option<bool>,

    /// print version
    #[arg(short = 'V', long, action = ArgAction::Version)]
    version: Option<bool>,

    /// puzzle files to parse to JSON
    #[arg(value_name = "PUZZLE", num_args = 1..)]
    files: Vec<String>,

    /// write output to a file instead of stdout
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    /// indent the JSON output
    #[arg(short, long)]
    pretty: bool,

    /// for a single file, print the puzzle object directly, not in an array
    #[arg(short, long)]
    single: bool,

    /// disable color and Unicode output (also honors NO_COLOR)
    #[arg(long, global = true)]
    no_color: bool,
}

#[derive(Subcommand)]
enum Command {
    /// parse puzzles to JSON (same as `puz FILES...`)
    Parse(parse_json::ParseArgs),

    /// validate every .puz file under a directory
    Validate(validate::ValidateArgs),

    /// show a file's raw structure, even if it fails to parse
    Dump {
        #[command(subcommand)]
        what: dump::DumpKind,
    },

    /// show a file's extension sections
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
