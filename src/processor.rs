use std::fmt;
use crate::reader::Layout;

pub struct GameBoard(Vec<String>);

pub fn process_boards(blank: &str) -> GameBoard {
    let size: f32 = blank.chars().count() as f32;
    let width: i32 = size.sqrt().trunc() as i32;

    let mut board: Vec<String> = Vec::new();
    for i in 0..width {
        let mut temp: String = String::from("");
        for j in 0..width {
            let index = j + (i * width);
            temp = temp + &blank.chars().nth(index as usize).unwrap().to_string();
        }
        board.push(String::from(temp));
    }

    for n in &board {
        println!("{:?}", n);
    }

    return GameBoard(board);
}

impl GameBoard {
    fn is_black_cell(&self, x: i32, y: i32) -> bool {
        let board_width = self.0.len() as i32;
        if x < 0 || y < 0 {
            true
        } else if x >= board_width || y >= board_width {
            false
        } else {
            let board_row: &String = self.0.get(y as usize).unwrap();
            let ch: char = board_row.chars().nth(x as usize).unwrap();
            if ch == '.' {
                true
            } else {
                false
            }
        }
    }

    fn cells_needs_across_number(&self, x: i32, y: i32) -> bool {
        let width = self.0.len() as i32;
        if x==0 || self.is_black_cell(x-1,y) {
            if x+1 < width && self.is_black_cell(x+1, y) {
                true
            }
        }
        false
    }
}
