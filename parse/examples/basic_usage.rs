use puz_parse::{parse, parse_bytes, parse_file};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Simple file parsing
    println!("=== Simple File Parsing ===");
    let puzzle = parse_file("examples/data/standard1.puz")?;
    println!("Title: {}", puzzle.info.title);
    println!("Author: {}", puzzle.info.author);
    println!("Size: {}x{}", puzzle.info.width, puzzle.info.height);
    println!();

    // Example 2: Parsing with error handling and warnings
    println!("=== Advanced Parsing with Warnings ===");
    let file = File::open("examples/data/rebus.puz")?;
    let result = parse(file)?;
    let puzzle = result.result;

    println!("Title: {}", puzzle.info.title);

    // Handle any warnings
    for warning in &result.warnings {
        println!("Warning: {}", warning);
    }
    println!();

    // Example 3: Working with clues
    println!("=== Working with Clues ===");
    println!("Across clues:");
    for (num, clue) in puzzle.clues.across.iter().take(5) {
        println!("  {}: {}", num, clue);
    }

    println!("Down clues:");
    for (num, clue) in puzzle.clues.down.iter().take(5) {
        println!("  {}: {}", num, clue);
    }
    println!();

    // Example 4: Working with the grid
    println!("=== Working with the Grid ===");
    println!("First few rows of solution:");
    for (i, row) in puzzle.grid.solution.iter().take(3).enumerate() {
        println!("  Row {}: {}", i + 1, row);
    }

    println!("First few rows of blank grid:");
    for (i, row) in puzzle.grid.blank.iter().take(3).enumerate() {
        println!("  Row {}: {}", i + 1, row);
    }
    println!();

    // Example 5: Working with extensions (rebus, circles, etc.)
    println!("=== Working with Extensions ===");
    if let Some(rebus) = &puzzle.extensions.rebus {
        println!("Rebus found! Entries:");
        for (key, value) in &rebus.table {
            println!("  {}: {}", key, value);
        }
    } else {
        println!("No rebus in this puzzle");
    }

    if let Some(_circles) = &puzzle.extensions.circles {
        println!("This puzzle has circled squares");
    } else {
        println!("No circled squares in this puzzle");
    }

    if let Some(_given) = &puzzle.extensions.given {
        println!("This puzzle has given squares");
    } else {
        println!("No given squares in this puzzle");
    }
    println!();

    // Example 6: Parsing from bytes
    println!("=== Parsing from Bytes ===");
    let data = std::fs::read("examples/data/standard1.puz")?;
    let puzzle_from_bytes = parse_bytes(&data)?;
    println!("Parsed from bytes: {}", puzzle_from_bytes.info.title);

    Ok(())
}
