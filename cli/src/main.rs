use anyhow::{Context, Result};
use clap::{Arg, Command};
use puz_parse::parse;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

fn main() -> Result<()> {
    let matches = Command::new("puz")
        .about("parse .puz crossword puzzle files")
        .long_about("parse .puz crossword puzzle files into structured data\n\nsupports all puzzle features including rebus squares, circled cells,\nand metadata extraction")
        .after_help("examples:\n    puz puzzle.puz                  # parse and output to stdout\n    puz *.puz --pretty              # parse multiple files with formatting\n    puz daily.puz -o output.json    # save output to file\n    puz puzzle.puz --single         # output single object for one file")
        .version("0.1.0")
        .arg(
            Arg::new("files")
                .help("puzzle files to process")
                .long_help("one or more .puz files to parse, supports glob patterns")
                .required(true)
                .num_args(1..)
                .value_name("PUZZLE"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("write output to file instead of stdout")
                .long_help("write output to the specified file instead of printing to stdout")
                .value_name("FILE"),
        )
        .arg(
            Arg::new("pretty")
                .short('p')
                .long("pretty")
                .help("format output with indentation and newlines")
                .long_help("make output human-readable with proper indentation")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("single")
                .short('s')
                .long("single")
                .help("output object directly (not wrapped in array)")
                .long_help("for single files, output the puzzle object directly instead of wrapping it in an array")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let files: Vec<&String> = matches.get_many::<String>("files").unwrap().collect();
    let output_file = matches.get_one::<String>("output");
    let pretty = matches.get_flag("pretty");
    let single = matches.get_flag("single");

    let mut results = Vec::new();

    for file_path in &files {
        match process_file(file_path) {
            Ok(puzzle) => {
                results.push(PuzzleResult {
                    file: file_path.to_string(),
                    success: true,
                    puzzle: Some(puzzle),
                    error: None,
                });
            }
            Err(e) => {
                eprintln!("Error processing {}: {}", file_path, e);
                results.push(PuzzleResult {
                    file: file_path.to_string(),
                    success: false,
                    puzzle: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    let output_data = if single && results.len() == 1 {
        if let Some(result) = results.into_iter().next() {
            if result.success {
                serde_json::to_value(result.puzzle.unwrap())?
            } else {
                serde_json::to_value(result)?
            }
        } else {
            serde_json::Value::Null
        }
    } else {
        serde_json::to_value(results)?
    };

    let json_output = if pretty {
        serde_json::to_string_pretty(&output_data)?
    } else {
        serde_json::to_string(&output_data)?
    };

    match output_file {
        Some(path) => {
            std::fs::write(path, json_output)
                .with_context(|| format!("Failed to write to {}", path))?;
        }
        None => {
            io::stdout()
                .write_all(json_output.as_bytes())
                .context("Failed to write to stdout")?;
            println!(); // Add newline
        }
    }

    Ok(())
}

fn process_file(path: &str) -> Result<puz_parse::Puzzle> {
    let file_path = Path::new(path);

    if !file_path.exists() {
        anyhow::bail!("File does not exist: {}", path);
    }

    if !file_path.is_file() {
        anyhow::bail!("Path is not a file: {}", path);
    }

    let file = File::open(file_path).with_context(|| format!("Failed to open file: {}", path))?;

    let result = parse(file).with_context(|| format!("Failed to parse .puz file: {}", path))?;

    // Print warnings to stderr
    for warning in &result.warnings {
        eprintln!("Warning in {}: {}", path, warning);
    }

    Ok(result.result)
}

#[derive(serde::Serialize)]
struct PuzzleResult {
    file: String,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    puzzle: Option<puz_parse::Puzzle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}
