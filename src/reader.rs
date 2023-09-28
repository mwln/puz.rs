use byteorder::{ByteOrder, LittleEndian};
use std::env;
use std::io;
use std::str;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
};

const NUL_CHAR: char = '\0';

trait ReadByteBuffers {
    fn read_byte_buffers(&mut self, reader: impl Read);
}

trait ReadNulSeperatedStrings {
    fn read_n_nul_separated_strings(&mut self, num_strings: u16) -> Vec<String>;
}

struct Component {
    name: String,
    buffer: Vec<u8>,
}

impl Component {
    fn new(name: String, length: u16) -> Component {
        Component {
            name,
            buffer: vec![0u8; length as usize],
        }
    }
}

pub struct Layout {
    bytes: Vec<Component>,
    pub blank: String,
    pub solution: String,
}

impl Layout {
    fn new(board_size: u16) -> Layout {
        Layout {
            bytes: Self::_bytes(board_size),
            blank: String::from(""),
            solution: String::from(""),
        }
    }

    fn _bytes(board_size: u16) -> Vec<Component> {
        return vec![
            Component::new(String::from("blank_board"), board_size),
            Component::new(String::from("solution_board"), board_size),
        ];
    }
}

struct Header {
    bytes: Vec<Component>,
    board_size: u16,
    num_clues: u16,
}

impl Header {
    fn new() -> Header {
        Header {
            bytes: Self::_bytes(),
            board_size: 0,
            num_clues: 0,
        }
    }

    fn _bytes() -> Vec<Component> {
        return vec![
            Component::new(String::from("checksum"), 0x02),
            Component::new(String::from("file_magic"), 0x0C),
            Component::new(String::from("cib_checksum"), 0x02),
            Component::new(String::from("masked_low_checksum"), 0x04),
            Component::new(String::from("masked_high_checksum"), 0x04),
            Component::new(String::from("version"), 0x04),
            Component::new(String::from("reserved_1c"), 0x02),
            Component::new(String::from("scrambled_checksum"), 0x02),
            Component::new(String::from("reserved_20"), 0x0C),
            Component::new(String::from("width"), 0x01),
            Component::new(String::from("height"), 0x01),
            Component::new(String::from("num_clues"), 0x02),
            Component::new(String::from("unknown_bitmask"), 0x02),
            Component::new(String::from("scrambled_tag"), 0x02),
        ];
    }
}

impl ReadByteBuffers for Header {
    fn read_byte_buffers(&mut self, mut reader: impl Read) {
        for component in self.bytes.iter_mut() {
            reader.read_exact(&mut component.buffer).ok();
        }
        let width = self.bytes[9].buffer.get(0).unwrap();
        let height = self.bytes[10].buffer.get(0).unwrap();
        let num_clues = LittleEndian::read_u16(&self.bytes[11].buffer);
        self.board_size = (width * height).into();
        self.num_clues = num_clues;
    }
}

impl ReadByteBuffers for Layout {
    fn read_byte_buffers(&mut self, mut reader: impl Read) {
        for component in self.bytes.iter_mut() {
            reader.read_exact(&mut component.buffer).ok();
        }
        self.blank = str::from_utf8(&self.bytes[0].buffer).unwrap().to_owned();
        self.solution = str::from_utf8(&self.bytes[1].buffer).unwrap().to_owned();
    }
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
    let mut reader: Box<dyn BufRead> = match input {
        None => Box::new(BufReader::new(io::stdin())),
        Some(filename) => Box::new(BufReader::new(File::open(filename).unwrap())),
    };

    let mut header = Header::new();
    header.read_byte_buffers(&mut reader);

    let mut layout = Layout::new(header.board_size);
    layout.read_byte_buffers(&mut reader);

    let puzzle_details = reader.read_n_nul_separated_strings(3);
    let clues = reader.read_n_nul_separated_strings(header.num_clues);
    let side_note = reader.read_n_nul_separated_strings(1);

    let mut extras_bytes = Vec::new();
    reader.read_to_end(&mut extras_bytes).ok();

    println!("board_size: {:?}", header.board_size);
    println!("num_clues: {:?}", header.num_clues);
    println!("blank board: {:?}", layout.solution);
    println!("solution board: {:?}", layout.blank);
    println!("strings info: {:?}", puzzle_details);
    println!("strings info: {:?}", clues);
    println!("strings info: {:?}", side_note);
    println!("extras_strings: {:?}", extras_bytes);

    Ok(())
}
