use gloo_console::log;
use gloo_file::FileReadError;
use nom::{bytes::complete::take, IResult};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn read_file(file: web_sys::File) {
    let file_reader = gloo_file::callbacks::read_as_bytes(&file.into(), parse_puz);
}

pub fn read_from_file() {}

fn parse_puz(input: Result<Vec<u8>, FileReadError>) {
    match take_10(&input.expect("should have valid input for file reading")) {
        Ok(bytes) => log!(String::from_utf8(Vec::from(bytes.0)).expect("Yes")),
        Err(err) => log!("cannot read bytes"),
    }
}

fn take_10(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take(10u8)(i)
}
