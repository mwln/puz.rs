# puz

A Rust library for parsing `.puz` crossword puzzle files.

This library provides functionality to parse the binary `.puz` file format used by crossword puzzle applications like AcrossLite. It extracts puzzle metadata, grids, clues, and advanced features like rebus squares and circled squares.

## Features

- **Complete .puz parsing** - Extracts all puzzle data including metadata, grids, and clues
- **Rebus support** - Handles puzzles with multi-character cell entries
- **Circled squares** - Supports puzzles with circled cells
- **Checksum validation** - Verifies file integrity during parsing
- **Rich metadata** - Extracts title, author, copyright, notes, and more
- **Pure Rust** - Memory-safe with zero-copy optimizations where possible
- **JSON ready** - Optional serde support for serialization

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
puz-parse = "0.1.0"

# For JSON serialization support:
puz-parse = { version = "0.1.0", features = ["json"] }
```

## Quick Start

```rust
use puz_parse::parse_file;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a .puz file
    let puzzle = parse_file("puzzle.puz")?;
    
    // Access puzzle information
    println!("Title: {}", puzzle.info.title);
    println!("Author: {}", puzzle.info.author);
    println!("Size: {}x{}", puzzle.info.width, puzzle.info.height);
    
    // Access clues
    for (num, clue) in &puzzle.clues.across {
        println!("{} Across: {}", num, clue);
    }

    Ok(())
}
```

## Advanced Usage

For more control over parsing and error handling:

```rust
use std::fs::File;
use puz_parse::parse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("puzzle.puz")?;
    let result = parse(file)?;
    let puzzle = result.result;
    
    // Handle any warnings that occurred during parsing
    for warning in &result.warnings {
        eprintln!("Warning: {}", warning);
    }

    Ok(())
}
```

## Data Structure

The parsed puzzle contains:

- **`info`** - Metadata (title, author, dimensions, etc.)
- **`grid`** - Solution and blank grids  
- **`clues`** - Across and down clues by number
- **`extensions`** - Advanced features (rebus, circles, given squares)

## Examples

See the [examples](examples/) directory for more detailed usage examples.

## File Format Support

This library supports the complete `.puz` file format specification, including:

- **Standard puzzles** - Basic crossword grids with clues
- **Rebus squares** - Cells containing multiple characters  
- **Circled squares** - Visual indicators for themed entries
- **Puzzle extensions** - GRBS, RTBL, and GEXT sections
- **Checksum validation** - File integrity verification
- **Scrambled puzzles** - Puzzles with encoded solutions (read-only)

## Error Handling

The library provides detailed error information:

```rust
use puz_parse::parse_file;

match parse_file("puzzle.puz") {
    Ok(puzzle) => {
        println!("Successfully parsed: {}", puzzle.info.title);
    }
    Err(e) => {
        eprintln!("Parse error: {}", e);
    }
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.