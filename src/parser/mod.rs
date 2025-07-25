use crate::{
    error::{ParseResult, PuzError, PuzWarning},
    types::*,
};
use std::io::{BufReader, Read};

mod clues;
mod extensions;
mod grids;
mod header;
mod io;
mod strings;
mod validation;

use clues::process_clues;
use extensions::parse_extensions_with_recovery;
use grids::parse_grids;
use header::parse_header;
use io::{read_remaining_data, validate_file_magic};
use strings::parse_strings;
use validation::validate_puzzle;

pub(crate) fn parse_puzzle<R: Read>(reader: R) -> Result<ParseResult<Puzzle>, PuzError> {
    let mut buf_reader = BufReader::new(reader);
    let mut warnings = Vec::new();

    validate_file_magic(&mut buf_reader)?;
    let header = parse_header(&mut buf_reader)?;

    if header.is_scrambled {
        warnings.push(PuzWarning::ScrambledPuzzle {
            version: header.version.clone(),
        });
    }

    let grids = parse_grids(&mut buf_reader, header.width, header.height)?;

    let strings = parse_strings(&mut buf_reader, header.num_clues)?;

    let extra_data = read_remaining_data(&mut buf_reader)?;
    let (extensions, ext_warnings) =
        parse_extensions_with_recovery(&extra_data, header.width, header.height)?;
    warnings.extend(ext_warnings);

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

    match validate_puzzle(&puzzle) {
        Ok(()) => {}
        Err(e) => {
            return Err(e);
        }
    }

    Ok(ParseResult::with_warnings(puzzle, warnings))
}
