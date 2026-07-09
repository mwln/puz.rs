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
    parse_puzzle_inner(reader, false)
}

/// Parse and require all stored checksums to match; a mismatch is a hard error.
pub(crate) fn parse_puzzle_strict<R: Read>(reader: R) -> Result<ParseResult<Puzzle>, PuzError> {
    parse_puzzle_inner(reader, true)
}

fn parse_puzzle_inner<R: Read>(reader: R, strict: bool) -> Result<ParseResult<Puzzle>, PuzError> {
    let mut buf_reader = BufReader::new(reader);
    let mut warnings = Vec::new();

    let global_cksum = validate_file_magic(&mut buf_reader)?;
    let header = parse_header(&mut buf_reader)?;

    if header.is_scrambled {
        warnings.push(PuzWarning::ScrambledPuzzle {
            version: header.version.clone(),
        });
    }

    // Capture stored checksums before consuming the header fields we need.
    let stored = crate::checksums::Stored {
        global: global_cksum,
        cib: header.cib_cksum,
        masked: header.masked_cksum,
    };
    let bitmask = header.bitmask;
    let scrambled_tag = header.scrambled_tag;

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

    validate_puzzle(&puzzle)?;

    // Checksum validation: reconstruct the clue order and recompute checksums
    // independently of the writer, then compare with the stored values.
    let ordered_clues = crate::grid::order_clues(&puzzle.grid.blank, &puzzle.clues)?;
    match crate::checksums::verify(
        &puzzle.info,
        &puzzle.grid,
        &ordered_clues,
        bitmask,
        scrambled_tag,
        &stored,
    ) {
        Ok(()) => {}
        Err(e) => {
            if strict {
                return Err(e);
            }
            if let PuzError::InvalidChecksum {
                expected,
                found,
                context,
            } = &e
            {
                warnings.push(PuzWarning::ChecksumMismatch {
                    context: context.clone(),
                    expected: *expected,
                    found: *found,
                });
            }
        }
    }

    Ok(ParseResult::with_warnings(puzzle, warnings))
}
