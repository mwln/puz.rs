use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};
use byteorder::{ByteOrder, LittleEndian};
use std::str;

trait ReadAtPosition {
    fn read_at_position(&mut self, offset: u16, buffer: &mut [u8]) -> std::io::Result<()>;
}

impl ReadAtPosition for File {
    fn read_at_position(&mut self, offset: u16, buffer: &mut [u8]) -> std::io::Result<()> {
        self.seek(SeekFrom::Start(offset.into()))?;
        self.read_exact(buffer)
    }
}

#[derive(Debug)]
struct PuzzleBytes<'a> {
    id: &'a str,
    offset: u16,
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
    
    let mut board_width = 0;
    let mut board_height = 0;
    let mut num_clues = 0;
    let mut solution_board = "";
    let mut blank_board = "";

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
    
    for bytes in header.iter_mut() {
        file.read_at_position(bytes.offset, &mut bytes.values).ok();
        match bytes.id {
            "width" => { board_width = *bytes.values.get(0).unwrap() },
            "height" => { board_height = *bytes.values.get(0).unwrap() },
            "num_clues" => { num_clues = LittleEndian::read_u16(&bytes.values) },
            _ => (),
        }
    }
    
    // TODO derive properties in a created struct
    let board_size: u16 = (board_width * board_height).into();
    let blank_offset = board_size + 0x34;
    let string_offset = blank_offset + board_size;
    
    let mut board_layout = vec![
        PuzzleBytes { id: "solution", offset: 0x34, values: vec![0u8; board_size.into()]},
        PuzzleBytes { id: "blank", offset: blank_offset, values: vec![0u8; board_size.into()]},
    ]; 
    
    for bytes in board_layout.iter_mut() {
        file.read_at_position(bytes.offset, &mut bytes.values).ok();
        match bytes.id {
            "solution" => solution_board = str::from_utf8(&bytes.values).unwrap(),
            "blank" => blank_board = str::from_utf8(&bytes.values).unwrap(),
            _ => (),
        }
    }

    file.seek(SeekFrom::Start(string_offset.into()))?;

    // TODO get this working
    //let mut the_rest = Vec::new();
    //let mut sparkle_heart = String::new();
    //let reset = vec![0u8; 0x01];
    //while sparkle_heart != "GEXT" {
    //    let mut buf = vec![0u8; 0x01];
    //    let mut read = buf[0]; 
    //    while read != 0 {
    //        file.read_exact(&mut buf)?;
    //        buf = reset;
    //        read = buf[0];
    //    }
    //    the_rest.push(sparkle_heart);
    //}

    println!("{board_size:?}");
    println!("{num_clues:?}");
    println!("{solution_board:?}");
    println!("{blank_board:?}");
    // println!("{the_rest:?}");

    Ok(())        
}

