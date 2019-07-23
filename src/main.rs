use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::collections::HashMap;
use std::collections::VecDeque;
use rand::seq::SliceRandom;

fn main() {
    println!("Hello, world!");
}

#[derive(PartialEq, Eq, Hash)]
struct Position {
    x: usize,
    y: usize
}

struct Program {
    data: Vec<u8>,
    line_offsets: Vec<usize>
}

impl TryFrom<File> for Program {
    type Error = Error;

    fn try_from(input: File) -> Result<Self, Self::Error> {
        let mut input_file = input;
        let mut file_str = String::new();

        input_file.read_to_string(&mut file_str)?;

        let data = file_str.into_bytes();
        let mut line_offsets = Vec::new();

        for (i, c) in data.iter().enumerate() {
            if *c == ('\n' as u8) {
                line_offsets.push(i);
            }
        }

        Ok(Program { data, line_offsets })
    }
}

impl Program {
    fn get(&self, pos: &Position) -> Option<u8> {
        let offset = self.line_offsets.get(pos.y)?;
        self.data.get(offset + pos.x).map(|op| *op)
    }
}

#[derive(Copy, Clone)]
enum Direction {
    Up, Down, Left, Right
}

enum Status {
    Completed, Waiting, Exception, Terminated
}

struct Runtime<'a> {
    program: &'a Program,
    overlay: HashMap<Position, u8>,
    current_pos: Position,
    current_dir: Direction,
    stack: Vec<u8>,
    quote_mode: bool,
    input_buffer: VecDeque<u8>,
    output_buffer: VecDeque<u8>
}

impl<'a> From<&'a Program> for Runtime<'a> {
    fn from(prog: &'a Program) -> Self {
        Runtime {
            program: prog,
            overlay: HashMap::new(),
            current_pos: Position { x: 0, y: 0 },
            current_dir: Direction::Right,
            stack: Vec::new(),
            quote_mode: false,
            input_buffer: VecDeque::new(),
            output_buffer: VecDeque::new()
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

    pub fn get_opcode(&self, pos: &Position) -> Option<u8> {
        match self.overlay.get(pos) {
            Some(opcode) => Some(*opcode),
            None => self.program.get(pos)
        }
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
            Direction::Right => {

            },
            Direction::Left => {

            },
            Direction::Up => {

            },
            Direction::Down => {

            }
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
        if let Some(opcode) = self.get_opcode(&self.current_pos) {
            if self.quote_mode {
                self.step_quoted(opcode)
            } else {
                self.step_unquoted(opcode)
            }
        } else {
            Status::Exception
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
                    self.output_buffer.push_back(' ' as u8);
                    for byte in format!("{}", value).as_bytes() {
                        self.output_buffer.push_back(*byte);
                    }
                    self.move_auto();
                    Status::Completed
                } else {
                    Status::Exception
                }
            },
            b',' => {
                if let Some(value) = self.stack.pop() {
                    self.output_buffer.push_back(' ' as u8);
                    self.output_buffer.push_back(value);
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
                    if let Some(value) = self.get_opcode(&Position { x: v1 as usize, y: v2 as usize }) {
                        self.stack.push(value);
                        self.move_auto();
                        Status::Completed
                    } else {
                        Status::Exception
                    }
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
