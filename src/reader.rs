use byteorder::{ByteOrder, LittleEndian};
use std::env;
use std::io;
use std::str;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

#[derive(Debug)]
struct PuzzleBytes<'a> {
    id: &'a str,
    values: Vec<u8>,
}

const NUL_CHAR: char = '\0';

trait ReadNulSeperatedStrings {
    fn read_n_nul_separated_strings(&mut self, num_strings: u16) -> Vec<String>;
}

impl ReadNulSeperatedStrings for Box<dyn BufRead> {
    fn read_n_nul_separated_strings(&mut self, num_strings: u16) -> Vec<String> {
        let mut strings = Vec::new();
        for _ in 1..=num_strings {
            let mut text = String::new();
            let mut read = 1;
            while read != 0 {
                let mut buf = vec![0u8; 1];
                self.read_exact(&mut buf).unwrap();
                let current_char = buf[0] as char;
                if current_char != NUL_CHAR {
                    text.push_str(&current_char.to_string());
                } else {
                    read = 0;
                }
            }
            strings.push(text);
        }
        strings
    }
}

pub fn read() -> std::io::Result<()> {
    let input = env::args().nth(1);
    let reader: Box<dyn BufRead> = match input {
        None => Box::new(BufReader::new(io::stdin())),
        Some(filename) => Box::new(BufReader::new(File::open(filename).unwrap())),
    };
    run(reader)?;
    Ok(())
}

fn run(mut reader: Box<dyn BufRead>) -> std::io::Result<()> {
    let mut board_width = 0;
    let mut board_height = 0;
    let mut num_clues = 0;
    let mut solution_board = "";
    let mut blank_board = "";

    let mut header = vec![
        PuzzleBytes {
            id: "checksum",
            values: vec![0u8; 0x02],
        },
        PuzzleBytes {
            id: "file_magic",
            values: vec![0u8; 0x0C],
        },
        PuzzleBytes {
            id: "cib_checksum",
            values: vec![0u8; 0x02],
        },
        PuzzleBytes {
            id: "masked_low_checksum",
            values: vec![0u8; 0x04],
        },
        PuzzleBytes {
            id: "masked_high_checksum",
            values: vec![0u8; 0x04],
        },
        PuzzleBytes {
            id: "version",
            values: vec![0u8; 0x04],
        },
        PuzzleBytes {
            id: "reserved_1c",
            values: vec![0u8; 0x02],
        },
        PuzzleBytes {
            id: "scrambled_checksum",
            values: vec![0u8; 0x02],
        },
        PuzzleBytes {
            id: "reserved_20",
            values: vec![0u8; 0x0C],
        },
        PuzzleBytes {
            id: "width",
            values: vec![0u8; 0x01],
        },
        PuzzleBytes {
            id: "height",
            values: vec![0u8; 0x01],
        },
        PuzzleBytes {
            id: "num_clues",
            values: vec![0u8; 0x02],
        },
        PuzzleBytes {
            id: "unknown_bitmask",
            values: vec![0u8; 0x02],
        },
        PuzzleBytes {
            id: "scrambled_tag",
            values: vec![0u8; 0x02],
        },
    ];

    for bytes in header.iter_mut() {
        reader.read_exact(&mut bytes.values).ok();

        match bytes.id {
            "width" => board_width = *bytes.values.get(0).unwrap(),
            "height" => board_height = *bytes.values.get(0).unwrap(),
            "num_clues" => num_clues = LittleEndian::read_u16(&bytes.values),
            _ => (),
        }
    }

    // TODO derive properties in a created struct
    let board_size: u16 = (board_width * board_height).into();

    let mut board_layout = vec![
        PuzzleBytes {
            id: "solution",
            values: vec![0u8; board_size.into()],
        },
        PuzzleBytes {
            id: "blank",
            values: vec![0u8; board_size.into()],
        },
    ];

    for bytes in board_layout.iter_mut() {
        reader.read_exact(&mut bytes.values).ok();
        match bytes.id {
            "solution" => solution_board = str::from_utf8(&bytes.values).unwrap(),
            "blank" => blank_board = str::from_utf8(&bytes.values).unwrap(),
            _ => (),
        }
    }

    let info_strings = reader.read_n_nul_separated_strings(3);
    let puzzle_clues = reader.read_n_nul_separated_strings(num_clues);
    let note = reader.read_n_nul_separated_strings(1);

    let gext = reader.read_n_nul_separated_strings(1);
    let _gext_bytes = &gext[0].as_bytes();

    println!("solution: {:?}", solution_board);
    println!("blank: {:?}", blank_board);
    println!("info_strings: {:?}", info_strings);
    println!("info_strings: {:?}", info_strings);
    println!("puzzle_clues: {:?}", puzzle_clues);
    println!("note: {:?}", note);
    println!("gext: {:?}", gext);

    Ok(())
}

#[test]
fn parses_nov2493() {
    run(Box::new(BufReader::new(
        File::open("./example_data/Nov2493.puz").unwrap(),
    )))
    .unwrap();
}
