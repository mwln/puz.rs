# puz

A Rust library for parsing `.puz` crossword puzzle files.

## Installation

```toml
[dependencies]
puz = "0.1.0"
```

## Usage

```rust
use std::fs::File;
use puz::parse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("puzzle.puz")?;
    let result = parse(file)?;
    let puzzle = result.result;

    println!("Title: {}", puzzle.info.title);
    println!("Author: {}", puzzle.info.author);
    println!("Size: {}x{}", puzzle.info.width, puzzle.info.height);
    
    for (num, clue) in &puzzle.clues.across {
        println!("{} Across: {}", num, clue);
    }

    Ok(())
}
```

## License

MIT