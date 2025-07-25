# puz

CLI tool for processing `.puz` crossword puzzle files and outputting structured JSON.

## Installation

```bash
cargo install puz
```

## Usage

```bash
# Parse a single file to JSON
puz puzzle.puz

# Parse with pretty formatting
puz puzzle.puz --pretty

# Parse single file without array wrapper
puz puzzle.puz --single --pretty

# Parse multiple files
puz puzzle1.puz puzzle2.puz --pretty

# Save to file
puz puzzle.puz --output output.json
```

## Output Format

The tool outputs JSON with the complete puzzle structure including:
- Puzzle metadata (title, author, dimensions, etc.)
- Grid data (solution and blank grids)
- Clues (across and down)
- Extensions (rebus, circles, given squares)

## Options

- `-p, --pretty` - Pretty-print JSON output
- `-s, --single` - Output single object instead of array for single file
- `-o, --output <FILE>` - Write to file instead of stdout