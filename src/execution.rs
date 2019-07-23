use std::mem::replace;
use std::convert::TryFrom;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs::File;
use std::io::prelude::*;
use std::io::Result;
use std::io::Error;
use rand::seq::SliceRandom;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize
}

pub struct Program {
    data: Vec<u8>,
    line_offsets: Vec<usize>,
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
        let mut line_offsets = Vec::new();

        let mut last_offset = 0;
        let mut width = 0;

        line_offsets.push(0);

        for (i, c) in data.iter().enumerate() {
            if *c == b'\n' {
                line_offsets.push(i);

                let row_width = i - last_offset;
                last_offset = i;

                if row_width > width {
                    width = row_width;
                }
            }
        }

        let height = line_offsets.len() + 1;

        Ok(Program { data, line_offsets, width, height })
    }
}

impl Program {
    fn get(&self, pos: &Position) -> u8 {
        if let Some(offset) = self.line_offsets.get(pos.y) {
            if let Some(opcode) = self.data.get(offset + pos.x) {
                *opcode
            } else {
                b' '
            }
        } else {
            b' '
        }
        
    }

    pub fn get_line(&self, index: usize) -> Option<&[u8]> {
        let start = self.line_offsets.get(index)?;
        let eof = self.data.len();
        let end = self.line_offsets.get(index + 1).unwrap_or(&eof);
        Some(&self.data[*start..*end])
    }
}

#[derive(Copy, Clone)]
pub enum Direction {
    Up, Down, Left, Right
}

pub enum Status {
    Completed, Waiting, Exception, Terminated
}

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

    pub fn get_current_dir(&self) -> &Direction {
        &self.current_dir
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
    
    fn move_manual(&mut self, dir: Direction, amount: usize) {
        self.current_dir = dir;
        for _ in 0..amount {
            self.move_auto();
        }
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

    fn pop2(&mut self) -> Option<(u8, u8)> {
        let e1 = self.stack.pop();
        let e2 = self.stack.pop();
        match (e1, e2) {
            (Some(v1), Some(v2)) => Some((v1, v2)),
            _ => None
        }
    }
    
    fn pop3(&mut self) -> Option<(u8, u8, u8)> {
        let e1 = self.stack.pop();
        let e2 = self.stack.pop();
        let e3 = self.stack.pop();
        match (e1, e2, e3) {
            (Some(v1), Some(v2), Some(v3)) => Some((v1, v2, v3)),
            _ => None
        }
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
                if let Some((v1, v2)) = self.pop2() {
                    self.stack.push(v2 + v1);
                    self.move_auto();
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b'-' => {
                if let Some((v1, v2)) = self.pop2() {
                    self.stack.push(v2 - v1);
                    self.move_auto();
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b'*' => {
                if let Some((v1, v2)) = self.pop2() {
                    self.stack.push(v2 * v1);
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b'/' => {
                if let Some((v1, v2)) = self.pop2() {
                    self.stack.push(v2 / v1);
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b'%' => {
                if let Some((v1, v2)) = self.pop2() {
                    self.stack.push(v2 % v1);
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b'!' => {
                if let Some(value) = self.stack.pop() {
                    if value == 0 {
                        self.stack.push(1);
                    } else {
                        self.stack.push(0);
                    }
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b'`' => {
                if let Some((v1, v2)) = self.pop2() {
                    let result = if v1 > v2 { 1 } else { 0 };
                    self.stack.push(result);
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b'>' => {
                self.move_manual(Direction::Right, 1);
                Status::Completed
            },
            b'<' => {
                self.move_manual(Direction::Left, 1);
                Status::Completed
            },
            b'^' => {
                self.move_manual(Direction::Up, 1);
                Status::Completed
            },
            b'v' => {
                self.move_manual(Direction::Down, 1);
                Status::Completed
            },
            b'?' => {
                let dir = [Direction::Right, Direction::Left, Direction::Up, Direction::Down].choose(&mut rand::thread_rng());
                self.move_manual(*(dir.unwrap()), 1);
                Status::Completed
            },
            b'_' => {
                if let Some(value) = self.stack.pop() {
                    if value == 0 {
                        self.move_manual(Direction::Right, 1);
                    } else {
                        self.move_manual(Direction::Left, 1);
                    }
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b'|' => {
                if let Some(value) = self.stack.pop() {
                    if value == 0 {
                        self.move_manual(Direction::Down, 1);
                    } else {
                        self.move_manual(Direction::Up, 1);
                    }
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b'"' => {
                self.quote_mode = true;
                self.move_auto();
                Status::Completed
            },
            b':' => {
                if let Some(value) = self.stack.pop() {
                    self.stack.push(value);
                    self.stack.push(value);
                    self.move_auto();
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b'\\' => {
                if let Some((v1, v2)) = self.pop2() {
                    self.stack.push(v2);
                    self.stack.push(v1);
                    self.move_auto();
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b'$' => {
                self.stack.pop();
                self.move_auto();
                Status::Completed
            },
            b'.' => {
                if let Some(value) = self.stack.pop() {
                    self.output_buffer.push(' ' as u8);
                    for byte in format!("{}", value).as_bytes() {
                        self.output_buffer.push(*byte);
                    }
                    self.move_auto();
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b',' => {
                if let Some(value) = self.stack.pop() {
                    self.output_buffer.push(' ' as u8);
                    self.output_buffer.push(value);
                    print!("{}", (value as char));
                    self.move_auto();
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b'#' => {
                self.move_auto();
                self.move_auto();
                Status::Completed
            },
            b'g' => {
                if let Some((v1, v2)) = self.pop2() {
                    let value = self.get_opcode(&Position { x: v1 as usize, y: v2 as usize });
                    self.stack.push(value);
                    self.move_auto();
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b'p' => {
                if let Some((v1, v2, v3)) = self.pop3() {
                    self.set_opcode(Position { x: v1 as usize, y: v2 as usize }, v3);
                    self.move_auto();
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b'&' => {
                if let Some(input) = self.input_buffer.pop_front() {
                    self.stack.push(input);
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
