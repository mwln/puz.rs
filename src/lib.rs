use std::io::{BufReader, Read};
use wasm_bindgen::prelude::*;

use byteorder::{ByteOrder, LittleEndian};
use serde_json::{json, Value};

enum PieceKind {
    Number,
    Natural,
}

enum ExtraKind {
    GRBS,
    RTBL,
    GEXT,
}

#[derive(Debug)]
enum BoardKind {
    Blank,
    Solution,
}

const EXTRAS: [(&str, ExtraKind); 3] = [
    ("GRBS", ExtraKind::GRBS),
    ("RTBL", ExtraKind::RTBL),
    ("GEXT", ExtraKind::GEXT),
];

#[wasm_bindgen]
pub async fn read_file(file: web_sys::File) -> JsValue {
    let data = gloo_file::futures::read_as_bytes(&file.into())
        .await
        .expect_throw("Error while reading the file");
    return match parse_puz(data.as_slice()) {
        Ok(parsed) => JsValue::from_str(&parsed.to_string()),
        Err(e) => JsValue::from_str(&e.to_string()),
    };
}

pub fn parse_puz(buffer: impl Read) -> std::io::Result<Value> {
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
    let mut header_data: Vec<(&str, String)> = vec![];
    for (offset, conversion, label) in header_offsets.iter() {
        let mut buffer = vec![0; *offset];
        reader.read_exact(&mut buffer)?;
        if let Some(c_type) = conversion {
            match c_type {
                PieceKind::Natural => header_data.push((label, buffer[0].to_string())),
                PieceKind::Number => {
                    header_data.push((label, LittleEndian::read_u16(&buffer).to_string()))
                }
            }
        }
    }

    let width = header_data[0]
        .1
        .parse::<usize>()
        .expect("Width was unable to be converted to a number.");
    let height = header_data[1]
        .1
        .parse::<usize>()
        .expect("Height was unable to be converted to a number.");
    let board_size = width * height;

    let mut board_data: Vec<(&BoardKind, String)> = vec![];
    let boards = vec![BoardKind::Solution, BoardKind::Blank];
    for board in boards.iter() {
        let mut buffer = vec![0; board_size];
        reader.read_exact(&mut buffer)?;
        if let Ok(s) = std::str::from_utf8(&buffer) {
            board_data.push((board, s.to_owned()));
        } else {
            println!("Board (type: {:?}) could not read from the buffer.", board);
        }
    }

    let mut info_data: Vec<(&str, String)> = vec![];
    let info_items = vec!["title", "author", "copyright"];
    for item in info_items.iter() {
        info_data.push((item, read_string_till_nul(&mut reader)));
    }

    let num_clues = header_data[2]
        .1
        .parse::<usize>()
        .expect("Cannot parse number of clues as a number.");
    let mut clue_data: Vec<String> = vec![];
    for _ in 1..=num_clues {
        clue_data.push(read_string_till_nul(&mut reader))
    }

    // add in the note that's at the end of the clues
    info_data.push(("note", read_string_till_nul(&mut reader)));

    let mut extras_data: Vec<u8> = Vec::new();
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

    let mut empty: Vec<String> = Vec::new();
    let mut solution: Vec<String> = Vec::new();
    let mut clues: Vec<Vec<Vec<String>>> =
        vec![vec![vec![String::new(), String::new()]; width]; height];

    for (kind, data) in board_data.iter() {
        let mut board_rows = data
            .chars()
            .collect::<Vec<char>>()
            .chunks(15)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<String>>();

        match kind {
            BoardKind::Blank => empty.append(&mut board_rows),
            BoardKind::Solution => solution.append(&mut board_rows),
        }
    }

    for (row, cols) in clues.iter_mut().enumerate() {
        for (col, clue_tuple) in cols.iter_mut().enumerate() {
            if let Some(current_tile) = empty[row].chars().nth(col) {
                if current_tile != '.' {
                    if row == 0 {
                        clue_tuple[0] = clue_data.remove(0);
                    } else {
                        if let Some(square) = empty[row - 1].chars().nth(col) {
                            if square == '.' {
                                clue_tuple[0] = clue_data.remove(0);
                            }
                        }
                    }
                    if col == 0 {
                        clue_tuple[1] = clue_data.remove(0);
                    } else {
                        if let Some(square) = empty[row].chars().nth(col - 1) {
                            if square == '.' {
                                clue_tuple[1] = clue_data.remove(0);
                            }
                        }
                    }
                }
            }
        }
    }

    let puz = json!({
        "info": {
            "title": info_data[0].1,
            "author": info_data[1].1,
        },
        "size": {
            "width": width,
            "height": height,
        },
        "boards": {
            "blank": empty,
            "solution": solution,
        },
        "clues": clues,
        "extras": {
            "grbs": grbs,
            "gext": gext,
            "rtbl": rtbl,
        }
    });

    Ok(puz)
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
