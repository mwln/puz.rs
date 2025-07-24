use crate::{error::{PuzError, ParseResult, PuzWarning}, types::*};
use std::io::{BufReader, Read};

mod header;
mod grids;
mod strings;
mod extensions;
mod clues;
mod io;
mod validation;

use header::parse_header;
use grids::parse_grids;
use strings::parse_strings;
use extensions::parse_extensions_with_recovery;
use clues::process_clues;
use io::{validate_file_magic, read_remaining_data};
use validation::validate_puzzle;

/// Parse a .puz file from a reader with warnings for recoverable issues
pub(crate) fn parse_puzzle<R: Read>(reader: R) -> Result<ParseResult<Puzzle>, PuzError> {
    let mut buf_reader = BufReader::new(reader);
    let mut warnings = Vec::new();

    // Validate file magic and parse header
    validate_file_magic(&mut buf_reader)?;
    let header = parse_header(&mut buf_reader)?;
    
    // Check for scrambled puzzles and add warning
    if header.is_scrambled {
        warnings.push(PuzWarning::ScrambledPuzzle { 
            version: header.version.clone() 
        });
    }

    // Parse the puzzle grids
    let grids = parse_grids(&mut buf_reader, header.width, header.height)?;

    // Parse strings (title, author, copyright, clues, notes)
    let strings = parse_strings(&mut buf_reader, header.num_clues)?;

    // Parse extra sections with recovery
    let extra_data = read_remaining_data(&mut buf_reader)?;
    let (extensions, ext_warnings) = parse_extensions_with_recovery(&extra_data, header.width, header.height)?;
    warnings.extend(ext_warnings);

    // Process clues to map them to grid positions
    let clues = process_clues(&grids.blank, &strings.clues)?;

    let puzzle = Puzzle {
        info: PuzzleInfo {
            title: strings.title,
            author: strings.author,
            copyright: strings.copyright,
            notes: strings.notes,
            width: header.width,
            height: header.height,
            version: header.version,
            is_scrambled: header.is_scrambled,
        },
        grid: grids,
        clues,
        extensions,
    };

    // Validate the complete puzzle (but don't fail on validation warnings)
    match validate_puzzle(&puzzle) {
        Ok(()) => {},
        Err(e) => {
            // For now, validation errors are still hard errors
            // In future versions, some could be converted to warnings
            return Err(e);
        }
    }

    Ok(ParseResult::with_warnings(puzzle, warnings))
}