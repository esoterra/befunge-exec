use std::fs::File;
use std::io::{Error as IOError, Read};

use crate::core::{Direction, Position};

/// A Program is executable
pub trait Program {
    /// Defines where the program wraps around horizontally
    /// This is also 1 larger than the maximum x index
    fn width(&self) -> u8;

    /// Defines where the program wraps around vertically
    /// This is also 1 larger than the maximum y index
    fn height(&self) -> u8;

    /// Retrieve the opcode located at a position
    /// Out of bound gets must return b' '
    fn get(&self, pos: Position) -> u8;

    /// Retrieve the specified row of the program
    fn get_line(&self, y: u8) -> Option<&[u8]>;

    /// Create a new position from a position and direction
    /// handling loop around at the maximum and minimum
    /// vertical and horizontal extents
    fn move_pos(&self, pos: Position, dir: Direction) -> Position {
        match dir {
            Direction::Right => {
                let x = pos.x + 1;
                let x = if x >= self.width() { 0 } else { x };
                Position { x, y: pos.y }
            }
            Direction::Left => {
                let x = if pos.x == 0 { self.width() } else { pos.x - 1 };
                Position { x, y: pos.y }
            }
            Direction::Up => {
                let y = if pos.y == 0 { self.height() } else { pos.y - 1 };
                Position { x: pos.x, y }
            }
            Direction::Down => {
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
    data: Vec<u8>,
    line_ends: Vec<usize>,
}

impl TryFrom<File> for VecProgram {
    type Error = IOError;
    
    fn try_from(mut file: File) -> std::result::Result<Self, Self::Error> {
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(data.into())
    }
}

impl From<Vec<u8>> for VecProgram {
    fn from(data: Vec<u8>) -> Self {
        let mut line_ends = Vec::new();
        let mut last_feed = None;

        for (i, c) in data.iter().enumerate() {
            if *c == b'\n' {
                line_ends.push(i);
                last_feed = Some(i);
            }
        }

        if let Some(last) = last_feed {
            if last != data.len() - 1 {
                line_ends.push(data.len())
            }
        } else {
            line_ends.push(data.len())
        }

        VecProgram { data, line_ends }
    }
}

impl Program for VecProgram {
    fn width(&self) -> u8 {
        255
    }

    fn height(&self) -> u8 {
        255
    }

    fn get(&self, pos: Position) -> u8 {
        *self.get_line(pos.y).unwrap_or(&[]).get(pos.x as usize).unwrap_or(&b' ')
    }

    fn get_line(&self, y: u8) -> Option<&[u8]> {
        let y = y as usize;
        if y >= self.line_ends.len() {
            None
        } else {
            let lower = match y {
                0 => 0,
                _ => self.line_ends[y-1] + 1,
            };
            let upper = self.line_ends[y];
            Some(&self.data[lower..upper])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_program_get_line() {
        let program = "a\nbb\nccc\ndddd\n".to_owned().into_bytes();
        let program = VecProgram::from(program);
        assert_eq!(Some(b"a".as_slice()), program.get_line(0));
        assert_eq!(Some(b"bb".as_slice()), program.get_line(1));
        assert_eq!(Some(b"ccc".as_slice()), program.get_line(2));
        assert_eq!(Some(b"dddd".as_slice()), program.get_line(3));
        for y in 4..255 {
            assert_eq!(None, program.get_line(y));
        }
    }

    #[test]
    fn test_vec_program_get() {
        let program = "a\nbb\nccc\ndddd\n".to_owned().into_bytes();
        let program = VecProgram::from(program);
        let rows = [
            (0, b'a', 1),
            (1, b'b', 2),
            (2, b'c', 3),
            (3, b'd', 4),
        ];
        // main rows
        for (y, cell, n) in rows.into_iter() {
            for x in 0..n {
                assert_eq!(cell, program.get(Position { x, y }));
            }
            for x in n..255 {
                assert_eq!(b' ', program.get(Position { x, y }));
            }
        }
        // rest
        for y in 4..255 {
            for x in 0..255 {
                assert_eq!(b' ', program.get(Position { x, y }));
            }
        }
    }
}