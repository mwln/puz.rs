use byteorder::{ByteOrder, LittleEndian};
use core::num;
use std::env;
use std::io;
use std::str;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read}
};

trait InterpretBytes {
    fn interpret_bytes(&mut self, reader: Box<dyn BufRead>);
}

impl InterpretBytes for Vec<(String, Vec<u8>)> {
    fn interpret_bytes(&mut self, mut reader: Box<dyn BufRead>) {
        for elem in self.iter_mut() {
            reader.read_exact(&mut elem.1).ok();
        }
    }
}

struct Header {
    bytes: Vec<(String, Vec<u8>)>,
    board_size: u16,
    num_clues: u16,
}

struct Layout {
    bytes: Vec<(String, Vec<u8>)>,
    blank_board: String,
    solution_board: String,
}

struct Strings {}

struct Extras {}

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
    
    let header = read_header(reader);

    // read_layout(reader, header.board_size)?; 
    // read_strings(reader)?;
    // read_extras(reader)?;

    Ok(())
}

fn read_header(mut reader: Box<dyn BufRead>) -> Header {  
    let mut bytes = vec![
        (String::from("checksum"), vec![0u8; 0x02]),
        (String::from("file_magic"), vec![0u8; 0x0C]),
        (String::from("cib_checksum"), vec![0u8; 0x02]),
        (String::from("masked_low_checksum"),vec![0u8; 0x04]),
        (String::from("masked_high_checksum"),vec![0u8; 0x04]),
        (String::from("version"),vec![0u8; 0x04]),
        (String::from("reserved_1c"),vec![0u8; 0x02]),
        (String::from("scrambled_checksum"),vec![0u8; 0x02]),
        (String::from("reserved_20"),vec![0u8; 0x0C]),
        (String::from("width"),vec![0u8; 0x01]),
        (String::from("height"),vec![0u8; 0x01]),
        (String::from("num_clues"),vec![0u8; 0x02]),
        (String::from("unknown_bitmask"),vec![0u8; 0x02]),
        (String::from("scrambled_tag"),vec![0u8; 0x02]),
    ];

    bytes.interpret_bytes(reader);
    
    // hacky way of testing width and height for confirmation

    let board_width = bytes[9].1.get(0).unwrap();
    let board_height = bytes[10].1.get(0).unwrap();
    let num_clues: u16 = LittleEndian::read_u16(&bytes[11].1);
    let board_size: u16 = (board_width * board_height).into();

    // match k {
    //     &mut "width" => board_width = *buffer.get(0).unwrap(),
    //     &mut "height" => board_height = *buffer.get(0).unwrap(),
    //     &mut "num_clues" => num_clues = LittleEndian::read_u16(&buffer),
    //     _ => (),
    // }

    println!("board_size: {:?}", board_size);
    println!("num_clues: {:?}", num_clues);
    
    Header {
        bytes,
        board_size,
        num_clues,
    }
}

// fn read_layout(mut reader: Box<dyn BufRead>, u16: board_size) -> std::io::Result<()> {
//     let blank_board = "";
//     let solution_board = "";
    
    // let mut bytes = [ 
    //     ("blank_board", vec![0u8; board_size.into()]),
    //     ("solution_board", vec![0u8; board_size.into()]),
    // ]
// } 

// fn run2(mut reader: Box<dyn BufRead>) -> std::io::Result<()> { 
//     // TODO derive properties in a created struct
//     let mut board_width: u8 = 0;
//     let mut board_height: u8 = 0;
//     let mut num_clues = 0;
//     let mut solution_board = "";
//     let mut blank_board = "";

//     let board_size: u16 = (board_width * board_height).into();

//     for bytes in board_layout.iter_mut() {
//         reader.read_exact(&mut bytes.values).ok();
//         match bytes.id {
//             "solution" => solution_board = str::from_utf8(&bytes.values).unwrap(),
//             "blank" => blank_board = str::from_utf8(&bytes.values).unwrap(),
//             _ => (),
//         }
//     }

//     let info_strings = reader.read_n_nul_separated_strings(3);
//     let puzzle_clues = reader.read_n_nul_separated_strings(num_clues);
//     let note = reader.read_n_nul_separated_strings(1);

//     let gext = reader.read_n_nul_separated_strings(1);
//     let _gext_bytes = &gext[0].as_bytes();

//     println!("solution: {:?}", solution_board);
//     println!("blank: {:?}", blank_board);
//     println!("info_strings: {:?}", info_strings);
//     println!("info_strings: {:?}", info_strings);
//     println!("puzzle_clues: {:?}", puzzle_clues);
//     println!("note: {:?}", note);
//     println!("gext: {:?}", gext);

//     Ok(())
// }

// fn interpret_from_reader()

#[test]
fn parses_nov2493() {
    run(Box::new(BufReader::new(
        File::open("./example_data/Nov2493.puz").unwrap(),
    )))
    .unwrap();
}

