mod test;

use std::mem::replace;
use std::convert::TryFrom;
use std::collections::VecDeque;
use std::fs::File;
use std::io::prelude::*;
use std::io::Result;
use std::io::Error;
use rand::seq::SliceRandom;
use std::num::Wrapping;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Program {
    data: Vec<Vec<u8>>,
    width: usize,
    height: usize
}

impl TryFrom<File> for Program {
    type Error = Error;

    fn try_from(input: File) -> Result<Self> {
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

        let height = data.len();

        Ok(Program { data, width, height })
    }
}

impl From<Vec<Vec<u8>>> for Program {
    fn from(input: Vec<Vec<u8>>) -> Self {
        let height = input.len();
        let mut width = 0;

        for row in input.iter() {
            if row.len() > width {
                width = row.len();
            }
        }

        Program {
            data: input,
            width, height
        }
    }
}

impl Program {
    fn get(&self, pos: &Position) -> u8 {
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

    fn set(&mut self, pos: &Position, opcode: u8) {
        let min_height = pos.y + 1;
        let min_width = pos.x + 1;

        if self.data.len() < min_height {
            self.data.resize(min_height, Vec::new());
            self.height = min_height;
        }
        let row = &mut self.data[pos.y];

        if row.len() < min_width {
            row.resize(min_width, b' ');
            
            if min_width > self.width {
                self.width = min_width;
            }
        }
        row[pos.x] = opcode;
    }

    fn move_dir(&self, dir: Direction, pos: &mut Position) {
        match dir {
            Direction::Right => {
                pos.x += 1;
                if pos.x >= self.width {
                    pos.x = 0;
                }
            },
            Direction::Left => {
                if pos.x == 0 {
                    pos.x = self.width;
                } else {
                    pos.x -= 1;
                }
            },
            Direction::Up => {
                if pos.y == 0 {
                    pos.y = self.height;
                } else {
                    pos.y -= 1;
                }
            },
            Direction::Down  => {
                pos.y += 1;
                if pos.y >= self.height {
                    pos.y = 0;
                }
            }
        }
    }

    pub fn get_line(&self, index: usize) -> Option<&[u8]> {
        self.data.get(index).map(|row| &row[..])
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Direction {
    Up, Down, Left, Right
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Status {
    Completed, Waiting, Terminated
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Runtime {
    program: Program,
    current_pos: Position,
    current_dir: Direction,
    stack: Vec<u8>,
    quote_mode: bool,
    input_buffer: VecDeque<u8>,
    output_buffer: Vec<u8>
}

impl From<Program> for Runtime {
    fn from(prog: Program) -> Self {
        Runtime {
            program: prog,
            current_pos: Position { x: 0, y: 0 },
            current_dir: Direction::Right,
            stack: Vec::new(),
            quote_mode: false,
            input_buffer: VecDeque::new(),
            output_buffer: Vec::new()
        }
    }
}

impl Runtime {
    pub fn get_current_pos(&self) -> &Position {
        &self.current_pos
    }

    pub fn get_current_dir(&self) -> &Direction {
        &self.current_dir
    }

    pub fn get_stack(&self) -> &[u8] {
        &self.stack[..]
    }

    pub fn get_opcode(&self, pos: &Position) -> u8 {
        self.program.get(pos)
    }

    pub fn get_line(&self) -> Option<&[u8]> {
        self.program.get_line(self.current_pos.y)
    }

    pub fn write_input(&mut self, input: &[u8]) {
        for byte in input {
            self.input_buffer.push_back(*byte);
        }
    }

    pub fn read_output(&mut self) -> Vec<u8> {
        let result = replace(&mut self.output_buffer, Vec::new());
        result
    }

    fn set_opcode(&mut self, pos: Position, opcode: u8) {
        self.program.set(&pos, opcode);
    }

    fn move_auto(&mut self) {
        self.program.move_dir(self.current_dir, &mut self.current_pos);
    }

    fn pop(&mut self) -> u8 {
        self.stack.pop().unwrap_or(0)
    }

    pub fn step(&mut self) -> Status {
        let opcode = self.get_opcode(&self.current_pos);
    
        if self.quote_mode {
            self.step_quoted(opcode)
        } else {
            self.step_unquoted(opcode)
        }
    }

    fn step_quoted(&mut self, opcode: u8) -> Status {
        match opcode {
            b'"' => self.quote_mode = false,
            _    => self.stack.push(opcode)
        }
        self.move_auto();
        Status::Completed
    }

    fn step_unquoted(&mut self, opcode: u8) -> Status {
        match opcode {
            b'+' => {
                let (e1, e2) = (self.pop(), self.pop());
                let result = Wrapping(e2) + Wrapping(e1);
                self.stack.push(result.0);
                self.move_auto();
                Status::Completed
            },
            b'-' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = Wrapping(lower) - Wrapping(upper);
                self.stack.push(result.0);
                self.move_auto();
                Status::Completed
            },
            b'*' => {
                let (e1, e2) = (self.pop(), self.pop());
                let result = Wrapping(e2) * Wrapping(e1);
                self.stack.push(result.0);
                self.move_auto();
                Status::Completed
            },
            b'/' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = Wrapping(lower) / Wrapping(upper);
                self.stack.push(result.0);
                self.move_auto();
                Status::Completed
            },
            b'%' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = Wrapping(lower) % Wrapping(upper);
                self.stack.push(result.0);
                self.move_auto();
                Status::Completed
            },
            b'!' => {
                if self.pop() == 0 {
                    self.stack.push(1);
                } else {
                    self.stack.push(0);
                }
                self.move_auto();
                Status::Completed
            },
            b'`' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = if lower > upper { 1 } else { 0 };
                self.stack.push(result);
                self.move_auto();
                Status::Completed
            },
            b'>' => {
                self.current_dir = Direction::Right;
                self.move_auto();
                Status::Completed
            },
            b'<' => {
                self.current_dir = Direction::Left;
                self.move_auto();
                Status::Completed
            },
            b'^' => {
                self.current_dir = Direction::Up;
                self.move_auto();
                Status::Completed
            },
            b'v' => {
                self.current_dir = Direction::Down;
                self.move_auto();
                Status::Completed
            },
            b'?' => {
                let dir = [Direction::Right, Direction::Left, Direction::Up, Direction::Down].choose(&mut rand::thread_rng());
                self.current_dir = *(dir.unwrap());
                self.move_auto();
                Status::Completed
            },
            b'_' => {
                self.current_dir = if self.pop() == 0 { Direction::Right } else { Direction::Left };
                self.move_auto();
                Status::Completed
            },
            b'|' => {
                self.current_dir = if self.pop() == 0 { Direction::Down } else { Direction::Up };
                self.move_auto();
                Status::Completed
            },
            b'"' => {
                self.quote_mode = true;
                self.move_auto();
                Status::Completed
            },
            b':' => {
                let value = self.pop();
                self.stack.push(value);
                self.stack.push(value);
                self.move_auto();
                Status::Completed
            },
            b'\\' => {
                let upper = self.pop();
                let lower = self.pop();
                self.stack.push(upper);
                self.stack.push(lower);
                self.move_auto();
                Status::Completed
            },
            b'$' => {
                self.pop();
                self.move_auto();
                Status::Completed
            },
            b'.' => {
                let value = self.pop();
                for byte in format!("{}", value).as_bytes() {
                    self.output_buffer.push(*byte);
                }
                self.output_buffer.push(b' ');
                self.move_auto();
                Status::Completed
            },
            b',' => {
                let value = self.pop();
                self.output_buffer.push(value);
                self.move_auto();
                Status::Completed
            },
            b'#' => {
                self.move_auto();
                self.move_auto();
                Status::Completed
            },
            b'g' => {
                let upper = self.pop();
                let lower = self.pop();
                let value = self.get_opcode(&Position { x: lower as usize, y: upper as usize });
                self.stack.push(value);
                self.move_auto();
                Status::Completed
            },
            b'p' => {
                let upper = self.pop();
                let middle = self.pop();
                let lower = self.pop();
                self.set_opcode(Position { x: middle as usize, y: upper as usize }, lower);
                self.move_auto();
                Status::Completed
            },
            b'&' => {
                if let Some(input_char) = self.input_buffer.pop_front() {
                    let input_num = input_char - (b'0' as u8);
                    self.stack.push(input_num);
                    self.move_auto();
                    Status::Completed
                } else {
                    Status::Waiting
                }
            },
            b'~' => {
                if let Some(input) = self.input_buffer.pop_front() {
                    self.stack.push(input);
                    self.move_auto();
                    Status::Completed
                } else {
                    Status::Waiting
                }
            },
            b'@' => {
                Status::Terminated
            },
            b'0' => {
                self.stack.push(0);
                self.move_auto();
                Status::Completed
            },
            b'1' => {
                self.stack.push(1);
                self.move_auto();
                Status::Completed
            },
            b'2' => {
                self.stack.push(2);
                self.move_auto();
                Status::Completed
            },
            b'3' => {
                self.stack.push(3);
                self.move_auto();
                Status::Completed
            },
            b'4' => {
                self.stack.push(4);
                self.move_auto();
                Status::Completed
            },
            b'5' => {
                self.stack.push(5);
                self.move_auto();
                Status::Completed
            },
            b'6' => {
                self.stack.push(6);
                self.move_auto();
                Status::Completed
            },
            b'7' => {
                self.stack.push(7);
                self.move_auto();
                Status::Completed
            },
            b'8' => {
                self.stack.push(8);
                self.move_auto();
                Status::Completed
            },
            b'9' => {
                self.stack.push(9);
                self.move_auto();
                Status::Completed
            },
            _ => {
                self.move_auto();
                Status::Completed
            }
        }
    }
}
