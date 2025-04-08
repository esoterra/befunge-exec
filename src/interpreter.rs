use std::collections::HashMap;

use crate::core::{Cursor, Direction, Mode, Position};
use crate::io::IO;
use crate::program::Program;

#[derive(PartialEq, Eq, Clone, Debug)]
/// An Interpreter represents a step by step executor for befunge code.
/// It contains a program, all necessary state, and IO buffers.
pub struct Interpreter<P: Program, IOImpl: IO> {
    program: P,
    overlay: HashMap<Position, u8>,

    cursor: Cursor,
    stack: Vec<u8>,

    io: IOImpl,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// The status of the Interpreter after it
/// has executed an instruction
pub enum Status {
    /// The result of most normal instructions
    Completed,
    /// The result of executing an input instruction
    /// with an empty input buffer
    Waiting,
    /// The result of executing the "@" termination instruction
    Terminated,
}

impl<P, IOImpl> Interpreter<P, IOImpl>
where
    P: Program,
    IOImpl: IO,
{
    /// Creates a new Interpreter that executes
    /// the provided program with the provided io.
    pub fn new(program: P, io: IOImpl) -> Self {
        let cursor = Cursor {
            pos: Position { x: 0, y: 0 },
            dir: Direction::Right,
            mode: Mode::Normal,
        };

        Interpreter {
            program: program,
            overlay: HashMap::new(),
            cursor,
            stack: Vec::new(),
            io,
        }
    }

    pub fn io(&mut self) -> &mut IOImpl {
        &mut self.io
    }

    /// Get the position of the cursor
    pub fn current_position(&self) -> Position {
        self.cursor.pos
    }

    /// Get the direction of teh cursor
    #[cfg(test)]
    pub fn current_direction(&self) -> Direction {
        self.cursor.dir
    }

    /// Get the current stack contents
    #[cfg(test)]
    pub fn stack(&self) -> &[u8] {
        &self.stack[..]
    }

    /// Retrieves the opcode located at a position in the program
    pub fn get_opcode(&self, pos: Position) -> u8 {
        if let Some(overlay_val) = self.overlay.get(&pos) {
            *overlay_val
        } else {
            self.program.get(pos)
        }
    }

    /// Updates the opcode at a specific position in the program
    fn set_opcode(&mut self, pos: Position, opcode: u8) {
        self.overlay.insert(pos, opcode);
    }

    /// Retrieves the current line the interpreter is on
    pub fn get_line(&self) -> Option<&[u8]> {
        self.program.get_line(self.cursor.pos.y)
    }

    fn move_auto(&mut self) {
        self.cursor.pos = self.program.move_pos(self.cursor.pos, self.cursor.dir);
    }

    fn pop(&mut self) -> u8 {
        self.stack.pop().unwrap_or(0)
    }

    /// Interprets the next command
    pub fn step(&mut self) -> Status {
        let opcode = self.get_opcode(self.cursor.pos);

        match self.cursor.mode {
            Mode::Quote => self.step_quoted(opcode),
            Mode::Normal => self.step_unquoted(opcode),
        }
    }

    fn step_quoted(&mut self, opcode: u8) -> Status {
        match opcode {
            b'"' => self.cursor.mode = Mode::Normal,
            _ => self.stack.push(opcode),
        }
        self.move_auto();
        Status::Completed
    }

    fn step_unquoted(&mut self, opcode: u8) -> Status {
        use std::num::Wrapping;

        let status = match opcode {
            b'+' => {
                let (e1, e2) = (self.pop(), self.pop());
                let result = Wrapping(e2) + Wrapping(e1);
                self.stack.push(result.0);
                Status::Completed
            }
            b'-' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = Wrapping(lower) - Wrapping(upper);
                self.stack.push(result.0);
                Status::Completed
            }
            b'*' => {
                let (e1, e2) = (self.pop(), self.pop());
                let result = Wrapping(e2) * Wrapping(e1);
                self.stack.push(result.0);
                Status::Completed
            }
            b'/' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = Wrapping(lower) / Wrapping(upper);
                self.stack.push(result.0);
                Status::Completed
            }
            b'%' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = Wrapping(lower) % Wrapping(upper);
                self.stack.push(result.0);
                Status::Completed
            }
            b'!' => {
                let value = self.pop();
                let result = if value == 0 { 1 } else { 0 };
                self.stack.push(result);
                Status::Completed
            }
            b'`' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = if lower > upper { 1 } else { 0 };
                self.stack.push(result);
                Status::Completed
            }
            b'>' => {
                self.cursor.dir = Direction::Right;
                Status::Completed
            }
            b'<' => {
                self.cursor.dir = Direction::Left;
                Status::Completed
            }
            b'^' => {
                self.cursor.dir = Direction::Up;
                Status::Completed
            }
            b'v' => {
                self.cursor.dir = Direction::Down;
                Status::Completed
            }
            b'?' => {
                use rand::seq::IndexedRandom;
                let dir = [
                    Direction::Right,
                    Direction::Left,
                    Direction::Up,
                    Direction::Down,
                ]
                .choose(&mut rand::rng());
                self.cursor.dir = *(dir.unwrap());
                Status::Completed
            }
            b'_' => {
                self.cursor.dir = if self.pop() == 0 {
                    Direction::Right
                } else {
                    Direction::Left
                };
                Status::Completed
            }
            b'|' => {
                self.cursor.dir = if self.pop() == 0 {
                    Direction::Down
                } else {
                    Direction::Up
                };
                Status::Completed
            }
            b'"' => {
                self.cursor.mode = Mode::Quote;
                Status::Completed
            }
            b':' => {
                let value = self.pop();
                self.stack.push(value);
                self.stack.push(value);
                Status::Completed
            }
            b'\\' => {
                let upper = self.pop();
                let lower = self.pop();
                self.stack.push(upper);
                self.stack.push(lower);
                Status::Completed
            }
            b'$' => {
                self.pop();
                Status::Completed
            }
            b'.' => {
                let number_string = format!("{} ", self.pop());
                let buf = number_string.as_bytes();
                self.io.write(buf);
                Status::Completed
            }
            b',' => {
                let buf = &[self.pop()];
                self.io.write(buf);
                Status::Completed
            }
            b'#' => {
                self.move_auto();
                Status::Completed
            }
            b'g' => {
                let upper = self.pop();
                let lower = self.pop();
                let value = self.get_opcode(Position { x: lower, y: upper });
                self.stack.push(value);
                Status::Completed
            }
            b'p' => {
                let upper = self.pop();
                let middle = self.pop();
                let lower = self.pop();
                self.set_opcode(
                    Position {
                        x: middle,
                        y: upper,
                    },
                    lower,
                );
                Status::Completed
            }
            b'&' => {
                if let Some(input_number) = self.io.read_number() {
                    self.stack.push(input_number);
                    Status::Completed
                } else {
                    Status::Waiting
                }
            }
            b'~' => {
                if let Some(input) = self.io.read_byte() {
                    self.stack.push(input);
                    Status::Completed
                } else {
                    Status::Waiting
                }
            }
            b'@' => Status::Terminated,
            b'0' => {
                self.stack.push(0);
                Status::Completed
            }
            b'1' => {
                self.stack.push(1);
                Status::Completed
            }
            b'2' => {
                self.stack.push(2);
                Status::Completed
            }
            b'3' => {
                self.stack.push(3);
                Status::Completed
            }
            b'4' => {
                self.stack.push(4);
                Status::Completed
            }
            b'5' => {
                self.stack.push(5);
                Status::Completed
            }
            b'6' => {
                self.stack.push(6);
                Status::Completed
            }
            b'7' => {
                self.stack.push(7);
                Status::Completed
            }
            b'8' => {
                self.stack.push(8);
                Status::Completed
            }
            b'9' => {
                self.stack.push(9);
                Status::Completed
            }
            b' ' => {
                Status::Completed
            }
            op => {
                eprintln!("Invalid opcode: {}", op);
                std::process::exit(1);
            }
        };
        match status {
            Status::Completed => self.move_auto(),
            Status::Waiting => {},
            Status::Terminated => {},
        }
        status
    }
}
