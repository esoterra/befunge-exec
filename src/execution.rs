use std::mem::replace;
use std::convert::TryFrom;
use std::collections::HashMap;
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

#[derive(Debug)]
pub struct Program {
    data: Vec<u8>,
    line_starts: Vec<usize>,
    line_ends: Vec<usize>,
    width: usize,
    height: usize
}

impl TryFrom<File> for Program {
    type Error = Error;

    fn try_from(input: File) -> Result<Self> {
        let mut input_file = input;
        let mut file_str = String::new();

        input_file.read_to_string(&mut file_str)?;

        let data = file_str.into_bytes();
        let mut line_starts = Vec::new();
        let mut line_ends = Vec::new();

        let mut width = 0;

        line_starts.push(0);

        for (i, c) in data.iter().enumerate() {
            if *c == b'\n' {
                let new_end = i;
                let new_start = i + 1;

                let last_start = line_starts.last().unwrap();
                let row_width = new_end - last_start;

                if row_width > width {
                    width = row_width;
                }
                
                line_ends.push(new_end);
                line_starts.push(new_start);
            }
        }

        if let Some(c) = data.last() {
            if *c != b'\n' {
                let last_end = data.len();
                line_ends.push(last_end);

                if last_end - line_starts.last().unwrap() > width {
                    width = last_end - line_starts.last().unwrap();
                }
            }
        } else {
            line_ends.push(1);
        }

        let height = line_starts.len();

        Ok(Program { data, line_starts, line_ends, width, height })
    }
}

impl Program {
    fn get(&self, pos: &Position) -> u8 {
        if let Some(line_start) = self.line_starts.get(pos.y) {
            if let Some(line_end) = self.line_ends.get(pos.y) {
                let line_width = line_end - line_start;
                if pos.x > line_width {
                    b' '
                } else {
                    self.data[line_start + pos.x]
                }
            } else {
                b' '
            }
        } else {
            b' '
        }
    }

    pub fn get_line(&self, index: usize) -> Option<&[u8]> {
        let start = self.line_starts.get(index)?;
        let end = self.line_ends.get(index)?;
        Some(&self.data[*start..*end])
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Up, Down, Left, Right
}

pub enum Status {
    Completed, Waiting, Terminated
}

#[derive(Debug)]
pub struct Runtime<'a> {
    program: &'a Program,
    overlay: HashMap<Position, u8>,
    width: usize,
    height: usize,
    current_pos: Position,
    current_dir: Direction,
    stack: Vec<u8>,
    quote_mode: bool,
    input_buffer: VecDeque<u8>,
    output_buffer: Vec<u8>
}

impl<'a> From<&'a Program> for Runtime<'a> {
    fn from(prog: &'a Program) -> Self {
        Runtime {
            program: prog,
            overlay: HashMap::new(),
            width: prog.width,
            height: prog.height,
            current_pos: Position { x: 0, y: 0 },
            current_dir: Direction::Right,
            stack: Vec::new(),
            quote_mode: false,
            input_buffer: VecDeque::new(),
            output_buffer: Vec::new()
        }
    }
}

impl<'a> Runtime<'a> {
    pub fn get_current_pos(&self) -> &Position {
        &self.current_pos
    }

    pub fn get_opcode(&self, pos: &Position) -> u8 {
        match self.overlay.get(pos) {
            Some(opcode) => *opcode,
            None => self.program.get(pos)
        }
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
        self.overlay.insert(pos, opcode);
    }

    fn move_auto(&mut self) {
        match self.current_dir {
            Direction::Right => self.current_pos.x += 1,
            Direction::Left  => self.current_pos.x -= 1,
            Direction::Up    => self.current_pos.y -= 1,
            Direction::Down  => self.current_pos.y += 1
        }

        if self.current_pos.x >= self.width {
            self.current_pos.x = 0;
        }
        if self.current_pos.y >= self.height {
            self.current_pos.y = 0;
        }
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
                self.stack.push(lower);
                self.stack.push(upper);
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
