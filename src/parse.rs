use std::io::{BufReader, Read};

use byteorder::{ByteOrder, LittleEndian};
use serde_json::{json, Value};

use serde::{Deserialize, Serialize};

type PuzzleBoard = Vec<Vec<char>>;
type Clues = Vec<Vec<Vec<String>>>;

#[derive(Serialize, Deserialize, Debug)]
struct Puzzle {
    info: PuzzleInfo,
    size: PuzzleSize,
    boards: PuzzleBoards,
    clues: Clues,
    extras: Extras,
}

#[derive(Serialize, Deserialize, Debug)]
struct PuzzleInfo {
    title: String,
    author: String,
    copyright: String,
    note: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct PuzzleSize {
    width: usize,
    height: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct PuzzleBoards {
    blank: PuzzleBoard,
    solution: PuzzleBoard,
}

#[derive(Serialize, Deserialize, Debug)]
struct Extras {
    grbs: Vec<Vec<u8>>,
    gext: Vec<Vec<u8>>,
    rtbl: String,
}

enum PieceKind {
    Number,
    Natural,
}

enum ExtraKind {
    GRBS,
    RTBL,
    GEXT,
}

const FREE_SQUARE: char = '-';
const TAKEN_SQUARE: char = '.';

const EXTRAS: [(&str, ExtraKind); 3] = [
    ("GRBS", ExtraKind::GRBS),
    ("RTBL", ExtraKind::RTBL),
    ("GEXT", ExtraKind::GEXT),
];

pub fn parse_puz(buffer: impl Read) -> std::io::Result<Puzzle> {
    let mut reader = BufReader::new(buffer);

    let header_offsets: Vec<(usize, Option<PieceKind>, &str)> = vec![
        (0x02, None, "checksum"),
        (0x0C, None, "file_magic"),
        (0x02, None, "cib_checksum"),
        (0x04, None, "masked_low_checksum"),
        (0x04, None, "masked_high_checksum"),
        (0x04, None, "version"),
        (0x02, None, "reserved_1c"),
        (0x02, None, "scrambled_checksum"),
        (0x0C, None, "reserved_20"),
        (0x01, Some(PieceKind::Natural), "width"),
        (0x01, Some(PieceKind::Natural), "height"),
        (0x02, Some(PieceKind::Number), "num_clues"),
        (0x02, None, "unknown_bitmask"),
        (0x02, None, "scrambled_tag"),
    ];

    let mut header_data: Vec<usize> = vec![];
    for (offset, conversion, _) in header_offsets.iter() {
        let mut buffer = vec![0; *offset];
        reader.read_exact(&mut buffer)?;
        if let Some(c_type) = conversion {
            match c_type {
                PieceKind::Natural => header_data.push(buffer[0] as usize),
                PieceKind::Number => header_data.push(LittleEndian::read_u16(&buffer) as usize),
            }
        }
    }

    let width = header_data[0];
    let height = header_data[1];
    let board_size = width * height;

    // [solution, blank]
    let mut boards = Vec::new();
    for _ in 0..2 {
        let mut buffer = vec![0; board_size];
        reader.read_exact(&mut buffer)?;
        if let Ok(s) = std::str::from_utf8(&buffer) {
            boards.push(
                s.chars()
                    .collect::<Vec<char>>()
                    .chunks(width)
                    .map(|chunk| chunk.to_vec())
                    .collect(),
            )
        }
    }

    let title = read_string_till_nul(&mut reader);
    let author = read_string_till_nul(&mut reader);
    let copyright = read_string_till_nul(&mut reader);

    let num_clues = header_data[2];
    let mut clue_data: Vec<String> = vec![];
    for _ in 1..=num_clues {
        clue_data.push(read_string_till_nul(&mut reader))
    }

    let note = read_string_till_nul(&mut reader);

    let mut extras_data = Vec::new();
    reader.read_to_end(&mut extras_data)?;

    let mut grbs = Vec::new();
    let mut rtbl = String::new();
    let mut gext = Vec::new();

    for (pattern, kind) in EXTRAS.iter() {
        if let Some(index) = extras_data
            .windows(pattern.len())
            .position(|window| window == pattern.as_bytes())
        {
            let length_start = index + pattern.len();
            let data_length =
                LittleEndian::read_u16(&extras_data[length_start..length_start + 2]) as usize;
            let data_start = length_start + 4;
            let data_end = data_start + data_length;
            let section_data = &extras_data[data_start..data_end];

            let valid = match kind {
                ExtraKind::GEXT => section_data.iter().any(|&u| u != 0u8),
                ExtraKind::GRBS => section_data.iter().any(|&u| u != 0u8),
                ExtraKind::RTBL => std::str::from_utf8(&section_data).unwrap().len() > 0,
            };

            if valid {
                match kind {
                    ExtraKind::RTBL => {
                        rtbl = std::str::from_utf8(&section_data)
                            .unwrap()
                            .trim()
                            .to_owned()
                    }
                    ExtraKind::GEXT => section_data
                        .chunks(width)
                        .for_each(|chunk| gext.push(chunk.to_vec())),
                    ExtraKind::GRBS => section_data
                        .chunks(width)
                        .for_each(|chunk| grbs.push(chunk.to_vec())),
                }
            }
        }
    }

    let mut clues: Clues = vec![vec![vec![String::new(), String::new()]; width]; height];

    for (row, cols) in clues.iter_mut().enumerate() {
        for (col, clue_tuple) in cols.iter_mut().enumerate() {
            if cell_needs_across_clue(&boards[1], row, col) {
                clue_tuple[0] = clue_data.remove(0);
            }
            if cell_needs_down_clue(&boards[1], row, col) {
                clue_tuple[1] = clue_data.remove(0);
            }
        }
    }

    Ok(Puzzle {
        info: PuzzleInfo {
            title,
            author,
            copyright,
            note,
        },
        size: PuzzleSize { width, height },
        boards: PuzzleBoards {
            solution: std::mem::take(&mut boards[0]),
            blank: std::mem::take(&mut boards[1]),
        },
        clues,
        extras: Extras { rtbl, grbs, gext },
    })
}

fn convert(p: Puzzle) -> std::io::Result<Value> {
    Ok(json!({
        "info": {
            "title": p.info.title ,
            "author": p.info.author,
            "copyright": p.info.copyright,
            "note": p.info.note,
        },
        "size": {
            "width": p.size.width,
            "height": p.size.height,
        },
        "boards": {
            "blank": p.boards.blank,
            "solution": p.boards.solution,
        },
        "clues": p.clues,
        "extras": {
            "grbs": p.extras.grbs,
            "gext": p.extras.gext,
            "rtbl": p.extras.rtbl,
        }
    }))
}

fn cell_needs_across_clue(board: &Vec<Vec<char>>, row: usize, col: usize) -> bool {
    if let Some(this_row) = board.get(row) {
        if let Some(this_square) = this_row.get(col) {
            if this_square == &FREE_SQUARE {
                if let Some(next_square) = this_row.get(col + 1) {
                    if next_square == &FREE_SQUARE {
                        if col == 0 {
                            return true;
                        } else if let Some(previous_square) = this_row.get(col - 1) {
                            return previous_square == &TAKEN_SQUARE;
                        }
                    }
                }
            }
        }
    }
    false
}

fn cell_needs_down_clue(board: &Vec<Vec<char>>, row: usize, col: usize) -> bool {
    if let Some(this_row) = board.get(row) {
        if let Some(this_square) = this_row.get(col) {
            if this_square == &FREE_SQUARE {
                if let Some(next_row) = board.get(row + 1) {
                    if let Some(next_square) = next_row.get(col) {
                        if next_square == &FREE_SQUARE {
                            if row == 0 {
                                return true;
                            } else if let Some(previous_row) = board.get(row - 1) {
                                if let Some(previous_square) = previous_row.get(col) {
                                    return previous_square == &TAKEN_SQUARE;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

fn read_string_till_nul(reader: &mut BufReader<impl Read>) -> String {
    let mut text = String::new();
    loop {
        let mut buf = [0u8; 1];
        if reader.read_exact(&mut buf).is_err() {
            break;
        }
        let current_char = buf[0] as char;
        if current_char == '\0' {
            break;
        }
        text.push(current_char);
    }
    text
}
