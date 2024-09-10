use puz_rs::parse_puz;
use std::{
    fs::File,
    io::{ErrorKind, Write},
};

fn main() -> std::io::Result<()> {
    let path = "examples/data/rebus.puz";
    let file = match File::open(&path) {
        Err(err) => match err.kind() {
            ErrorKind::NotFound => panic!("File not found at path: {}", &path),
            other_error => panic!("Problem opening the file: {:?}", other_error),
        },
        Ok(file) => file,
    };

    let puzzle = parse_puz(&file).expect("Error while parsing the file.");
    let file_path = "examples/output.json";
    let mut file = File::create(file_path)?;
    file.write_all(puzzle.to_string().as_bytes())?;

    Ok(())
}
