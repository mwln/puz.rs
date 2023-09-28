use std::fmt;

pub struct GameBoard(Vec<String>);
pub struct Clues(Vec<String>);
pub struct Cell {
    x: i32,
    y: i32,
}

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
    fn width(&self) -> i32 {
        self.0.get(0).unwrap().chars().count() as i32
    }

    fn height(&self) -> i32 {
        self.0.len() as i32
    }

    fn is_black_cell(&self, x: i32, y: i32) -> bool {
        return if x < 0 || y < 0 {
            true
        } else if x >= self.width() || y >= self.height() {
            false
        } else {
            let board_row: &String = self.0.get(y as usize).unwrap();
            let ch: char = board_row.chars().nth(x as usize).unwrap();
            if ch == '.' {
                true
            } else {
                false
            }
        };
    }

    fn cell_needs_across_number(&self, x: i32, y: i32) -> bool {
        self.is_black_cell(x - 1, y) && !self.is_black_cell(x + 1, y) && !self.is_black_cell(x, y)
    }

    fn cell_needs_down_number(&self, x: i32, y: i32) -> bool {
        self.is_black_cell(x, y - 1) && !self.is_black_cell(x, y + 1) && !self.is_black_cell(x, y)
    }
}

pub fn assign_clues(board: GameBoard) -> (Vec<Vec<i32>>, Vec<i32>, Vec<i32>) {
    let mut across_numbers: Vec<i32> = vec![];
    let mut down_numbers: Vec<i32> = vec![];
    let mut cell_numbers: Vec<Vec<i32>> =
        vec![vec![0i32; board.width() as usize]; board.height() as usize];
    let mut clue_number = 1;
    for i in 0..board.height() {
        for j in 0..board.width() {
            let cell_number = j + (i * board.width());
            let mut assigned_clue = false;
            if board.cell_needs_across_number(j, i) {
                across_numbers.push(cell_number);
                cell_numbers[i as usize][j as usize] = clue_number;
                assigned_clue = true;
            }
            if board.cell_needs_down_number(j, i) {
                down_numbers.push(cell_number);
                cell_numbers[j as usize][i as usize] = clue_number;
                assigned_clue = true;
            }
            if assigned_clue {
                clue_number += 1;
            }
        }
    }
    return (cell_numbers, across_numbers, down_numbers);
}
