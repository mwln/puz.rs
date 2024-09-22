use std::{fs::File, io::Write, path::Path};

mod parse;
mod play;

use clap::Parser;
use color_eyre::eyre;
use parse::{convert, parse_puz};

/// Parse or play crossword puzzles
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(required = true)]
    puzzle: String,
}

fn main() -> color_eyre::Result<()> {
    let cli = Cli::parse();
    let file_path = Path::new(&cli.puzzle);

    if !file_path.is_file() {
        return Err(eyre::eyre!(
            "You didn't provide a valid file path. Are you sure you passed the correct file?"
        ));
    }

    let file = File::open(file_path)?;
    let puzzle = parse_puz(file)?;
    let parsed_json = convert(puzzle)?;

    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| eyre::eyre!("Invalid file path"))?;

    let output_file_name = format!("examples/{}_out.json", file_stem);
    let mut output_file = File::create(output_file_name)?;

    output_file.write_all(parsed_json.to_string().as_bytes())?;

    play::start()?;

    Ok(())
}
