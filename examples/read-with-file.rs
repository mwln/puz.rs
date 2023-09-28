use std::{
    fs::File,
    io::{BufReader, ErrorKind, Read},
};

use byteorder::{ByteOrder, LittleEndian};

enum PieceKind {
    None,
    String,
    Number,
    Natural,
}

fn main() -> std::io::Result<()> {
    let path = "example_data/Nov2493.puz";
    let file = match File::open(&path) {
        Err(err) => match err.kind() {
            ErrorKind::NotFound => panic!("File not found at path: {}", &path),
            other_error => panic!("Problem opening the file: {:?}", other_error),
        },
        Ok(file) => file,
    };
    let mut reader = BufReader::new(file);
    let header_offsets: Vec<(usize, Option<PieceKind>, &str)> = vec![
        (0x02, None, "checksum"),
        (0x0C, None, "file_magic"),
        (0x02, None, "cib_checksum"),
        (0x04, None, "masked_low_checksum"),
        (0x04, None, "masked_high_checksum"),
        (0x04, Some(PieceKind::String), "version"),
        (0x02, None, "reserved_1c"),
        (0x02, None, "scrambled_checksum"),
        (0x0C, None, "reserved_20"),
        (0x01, Some(PieceKind::Natural), "width"),
        (0x01, Some(PieceKind::Natural), "height"),
        (0x02, Some(PieceKind::Number), "num_clues"),
        (0x02, None, "unknown_bitmask"),
        (0x02, None, "scrambled_tag"),
    ];
    for (offset, conversion, label) in header_offsets.iter() {
        let mut buffer = vec![0; *offset];
        reader.read_exact(&mut buffer)?;
        if let Some(c_type) = conversion {
            match c_type {
                PieceKind::Natural => print_puz_piece(label, &buffer),
                PieceKind::Number => print_puz_piece(label, &LittleEndian::read_u16(&buffer)),
                PieceKind::String => {
                    if let Ok(s) = std::str::from_utf8(&buffer) {
                        print_puz_piece(label, s);
                    } else {
                        println!("Puz::String listed in header but cannot convert to String. Check offsets or validity of file.");
                    }
                }
                PieceKind::None => {}
            }
        }
    }

    Ok(())
}

fn print_puz_piece<T: std::fmt::Debug>(label: &str, value: T) {
    println!("{}: {:?}", label, value);
}
