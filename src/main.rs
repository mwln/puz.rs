use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};
use byteorder::{ByteOrder, LittleEndian};
use std::str;

trait ReadAtPosition {
    fn read_at_position(&mut self, offset: u8, buffer: &mut [u8]) -> std::io::Result<()>;
}

impl ReadAtPosition for File {
    fn read_at_position(&mut self, offset: u8, buffer: &mut [u8]) -> std::io::Result<()> {
        self.seek(SeekFrom::Start(offset.into()))?;
        self.read_exact(buffer)
    }
}

#[derive(Debug)]
struct PuzzleBytes<'a> {
    id: &'a str,
    offset: u8,
    values: Vec<u8>,
}

#[derive(Debug)]
struct Board {
    width: u8,
    height: u8,
}

#[derive(Debug)]
struct Crossword {
    num_clues: u16,
}

fn main() -> std::io::Result<()> {
    let mut file = File::open("Nov2493.puz")?;

    let mut header = vec![
        PuzzleBytes  { id: "checksum", offset: 0x00, values: vec![0u8; 0x02] },
        PuzzleBytes  { id: "file_magic", offset: 0x02, values: vec![0u8; 0x0C] },
        PuzzleBytes  { id: "cib_checksum", offset: 0x0E, values: vec![0u8; 0x02] },
        PuzzleBytes  { id: "masked_low_checksum", offset: 0x10, values: vec![0u8; 0x04]},
        PuzzleBytes  { id: "masked_high_checksum", offset: 0x14, values: vec![0u8; 0x04]},
        PuzzleBytes  { id: "version", offset: 0x18, values: vec![0u8; 0x04]},
        PuzzleBytes  { id: "reserved", offset: 0x1C, values: vec![0u8; 0x02]},
        PuzzleBytes  { id: "scrambled_checksum", offset: 0x1E, values: vec![0u8; 0x02]},
        PuzzleBytes  { id: "width", offset: 0x2C, values: vec![0u8; 0x01]},
        PuzzleBytes  { id: "height", offset: 0x2D, values: vec![0u8; 0x01]},
        PuzzleBytes  { id: "num_clues", offset: 0x2E, values: vec![0u8; 0x02]},
        PuzzleBytes  { id: "unknown_bitmask", offset: 0x30, values: vec![0u8; 0x02]},
        PuzzleBytes  { id: "scrambled_tag", offset: 0x32, values: vec![0u8; 0x02]},
    ];
    
    let mut board_width = 0;
    let mut board_height = 0;
    let mut num_clues = 0;
    
    for bytes in header.iter_mut() {
        file.read_at_position(bytes.offset, &mut bytes.values).ok();
        match bytes.id {
            "width" => { board_width = *bytes.values.get(0).unwrap() },
            "height" => { board_height = *bytes.values.get(0).unwrap() },
            "num_clues" => { num_clues = LittleEndian::read_u16(&bytes.values) },
            _ => (),
        }
    }

    let mut board_layout = vec![
        PuzzleBytes { id: "layout", offset: 0x34, values: vec![0u8; (board_width * board_height).into() ]}
    ];

    let mut solution = "";

    for bytes in board_layout.iter_mut() {
        file.read_at_position(bytes.offset, &mut bytes.values).ok();
        match bytes.id {
            "layout" => solution = str::from_utf8(&bytes.values).unwrap(),
            _ => (),
        }
    }

    println!("{solution:?}");
    println!("{num_clues:?}");

    Ok(())        
}

