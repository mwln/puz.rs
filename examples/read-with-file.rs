use puz::parse;
use std::{fs::File, io::ErrorKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "examples/data/rebus.puz";
    let file = match File::open(&path) {
        Err(err) => match err.kind() {
            ErrorKind::NotFound => panic!("File not found at path: {}", &path),
            other_error => panic!("Problem opening the file: {:?}", other_error),
        },
        Ok(file) => file,
    };

    let result = parse(file)?;
    let puzzle = result.result;

    // Print any warnings
    for warning in &result.warnings {
        println!("Warning: {}", warning);
    }

    // Pretty print the puzzle info
    println!("Title: {}", puzzle.info.title);
    println!("Author: {}", puzzle.info.author);
    println!("Size: {}x{}", puzzle.info.width, puzzle.info.height);
    println!("Version: {}", puzzle.info.version);
    println!("Scrambled: {}", puzzle.info.is_scrambled);
    println!("Across clues: {}", puzzle.clues.across.len());
    println!("Down clues: {}", puzzle.clues.down.len());

    // Print some sample clues
    println!("\nSample across clues:");
    for (num, clue) in puzzle.clues.across.iter().take(3) {
        println!("  {}: {}", num, clue);
    }

    println!("\nSample down clues:");
    for (num, clue) in puzzle.clues.down.iter().take(3) {
        println!("  {}: {}", num, clue);
    }

    if let Some(rebus) = &puzzle.extensions.rebus {
        println!("\nRebus entries found: {}", rebus.table.len());
        for (key, value) in &rebus.table {
            println!("  {}: {}", key, value);
        }
    }

    println!("\nPuzzle parsed successfully!");

    Ok(())
}
