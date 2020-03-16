use std::fs::File;
use std::io::prelude::*;
use std::io::Result;

use crate::core::{ Position, Direction };

/// A Program is executable 
pub trait Program {
    /// Defines where the program wraps around horizontally
    /// This is also 1 larger than the maximum x index
    fn width(&self) -> usize;

    /// Defines where the program wraps around vertically
    /// This is also 1 larger than the maximum y index
    fn height(&self) -> usize;

    /// Retrieve the opcode located at a position
    /// Out of bound gets must return b' '
    fn get(&self, pos: Position) -> u8;

    /// Retrieve the specified row of the program
    fn get_line(&self, row_index: usize) -> Option<&[u8]>;

    /// Create a new position from a position and direction
    /// handling loop around at the maximum and minimum
    /// vertical and horizontal extents
    fn move_pos(&self, pos: Position, dir: Direction) -> Position {
        match dir {
            Direction::Right => {
                let x = pos.x + 1;
                let x = if x >= self.width() { 0 } else { x };
                Position { x, y: pos.y }
            },
            Direction::Left => {
                let x = if pos.x == 0 { self.width() } else { pos.x - 1 };
                Position { x, y: pos.y }
            },
            Direction::Up => {
                let y = if pos.y == 0 { self.height() } else { pos.y - 1 };
                Position { x: pos.x, y }
            },
            Direction::Down  => {
                let y = pos.y + 1;
                let y = if y >= self.height() { 0 } else { y };
                Position { x: pos.x, y }
            }
        }
    }
}

// A program that stores its data in vectors
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct VecProgram {
    data: Vec<Vec<u8>>,
    width: usize,
}

impl VecProgram {
    pub fn from_file(input: File) -> Result<Self> {
        let mut input_file = input;
        let mut file_str = String::new();

        input_file.read_to_string(&mut file_str)?;

        let file_data = file_str.into_bytes();
        let mut data = Vec::new();

        let mut start = 0;
        let mut width = 0;

        for (i, c) in file_data.iter().enumerate() {
            if *c == b'\n' {
                let end = i;
                let row_width = end - start;

                if row_width > width {
                    width = row_width;
                }

                data.push(Vec::from(&file_data[start..end]));

                start = end + 1;
            }
        }

        if let Some(last_char) = file_data.last() {
            if *last_char != b'\n' {
                data.push(Vec::from(&file_data[start..]));

                let row_width = file_data.len() - start;
                if row_width > width {
                    width = row_width;
                }
            }
        }

        Ok(VecProgram { data, width })
    }

    #[cfg(test)]
    pub fn from_vec(input: Vec<Vec<u8>>) -> Self {
        let mut width = 0;

        for row in input.iter() {
            if row.len() > width {
                width = row.len();
            }
        }

        VecProgram {
            data: input,
            width
        }
    }
}

impl Program for VecProgram {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.data.len()
    }

    fn get(&self, pos: Position) -> u8 {
        if let Some(row) = self.data.get(pos.y) {
            if let Some(cell) = row.get(pos.x) {
                *cell
            } else {
                b' '
            }
        } else {
            b' '
        }
    }

    fn get_line(&self, index: usize) -> Option<&[u8]> {
        self.data.get(index).map(|row| &row[..])
    }
}
