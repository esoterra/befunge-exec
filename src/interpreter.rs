use thiserror::Error;

use crate::{
    core::{Cell, Cursor, Direction, Mode, Position},
    io::{IO, StdIO},
    record::Record,
    space::Space,
};

#[derive(PartialEq, Eq, Clone, Debug)]
/// An Interpreter represents a step by step executor for befunge code.
/// It contains a program, all necessary state, and IO buffers.
pub struct Interpreter<IOImpl, R> {
    space: Space<Cell>,

    cursor: Cursor,
    stack: Vec<Cell>,

    io: IOImpl,
    recorder: R,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
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
    /// The
    Error(InterpreterError),
}

#[derive(Debug, Error, PartialEq, Eq, Hash, Clone)]
pub enum InterpreterError {
    #[error("Infinite loop detected")]
    InfiniteLoop,
    #[error("Invalid opcode {0} found")]
    InvalidOpcode(u8),
}

impl Interpreter<StdIO, ()> {
    pub fn new_std(space: Space<Cell>) -> Self {
        let cursor = Cursor::default();
        Interpreter {
            space,
            cursor,
            stack: Vec::new(),
            io: StdIO::default(),
            recorder: (),
        }
    }
}

impl<IOImpl: IO, R: Record> Interpreter<IOImpl, R> {
    /// Creates a new Interpreter that executes
    /// the provided program with the provided io
    /// and records events to the provided recorder.
    pub fn new(space: Space<Cell>, io: IOImpl, recorder: R) -> Self {
        let cursor = Cursor::default();
        Interpreter {
            space,
            cursor,
            stack: Vec::new(),
            io,
            recorder,
        }
    }

    pub fn io(&self) -> &IOImpl {
        &self.io
    }

    pub fn io_mut(&mut self) -> &mut IOImpl {
        &mut self.io
    }

    pub fn space(&self) -> &Space<Cell> {
        &self.space
    }

    /// Get the position of the cursor
    pub fn current_position(&self) -> Position {
        self.cursor.pos
    }

    /// Get the direction of the cursor
    #[allow(dead_code)]
    pub fn current_direction(&self) -> Direction {
        self.cursor.dir
    }

    /// Get the current stack contents
    pub fn stack(&self) -> &[Cell] {
        &self.stack[..]
    }

    fn put(&mut self, pos: Position, cell: Cell) {
        let old = self.space.get_cell(pos);
        self.recorder.replace(pos, old.0, cell.0);
        self.space.set_cell(pos, cell);
    }

    fn move_auto(&mut self) {
        let Cursor { pos, dir, mode: _ } = self.cursor;
        self.cursor.pos = self.space.move_pos(pos, dir);
    }

    fn pop(&mut self) -> u8 {
        match self.stack.pop() {
            Some(top) => {
                self.recorder.pop(top.0);
                top.0
            }
            None => {
                self.recorder.pop_bottom();
                0
            }
        }
    }

    fn push(&mut self, cell: u8) {
        self.recorder.push(cell);
        self.stack.push(Cell(cell));
    }

    /// Interprets the next command
    pub fn step(&mut self) -> Status {
        let cell = self.space.get_cell(self.cursor.pos);
        self.recorder.start_step(self.cursor.pos, cell.0);

        let status = match self.cursor.mode {
            Mode::Quote => self.step_quoted(cell),
            Mode::Normal => self.step_unquoted(cell),
        };

        if self.cursor.mode == Mode::Normal {
            if let Some(status) = self.skip_spaces() {
                return status;
            }
        }

        if status == Status::Waiting {
            self.recorder.rollback_step();
        } else {
            self.recorder.commit_step();
        }

        status
    }

    fn step_quoted(&mut self, cell: Cell) -> Status {
        match cell {
            Cell(b'"') => {
                self.cursor.mode = Mode::Normal;
                self.recorder.exit_quote();
            }
            _ => self.stack.push(cell),
        }
        self.move_auto();
        Status::Completed
    }

    fn step_unquoted(&mut self, cell: Cell) -> Status {
        use std::num::Wrapping;

        let status = match cell.0 {
            b'+' => {
                let (e1, e2) = (self.pop(), self.pop());
                let result = Wrapping(e2) + Wrapping(e1);
                self.push(result.0);
                Status::Completed
            }
            b'-' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = Wrapping(lower) - Wrapping(upper);
                self.push(result.0);
                Status::Completed
            }
            b'*' => {
                let (e1, e2) = (self.pop(), self.pop());
                let result = Wrapping(e2) * Wrapping(e1);
                self.push(result.0);
                Status::Completed
            }
            b'/' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = Wrapping(lower) / Wrapping(upper);
                self.push(result.0);
                Status::Completed
            }
            b'%' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = Wrapping(lower) % Wrapping(upper);
                self.push(result.0);
                Status::Completed
            }
            b'!' => {
                let value = self.pop();
                let result = if value == 0 { 1 } else { 0 };
                self.push(result);
                Status::Completed
            }
            b'`' => {
                let upper = self.pop();
                let lower = self.pop();
                let result = if lower > upper { 1 } else { 0 };
                self.push(result);
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
                self.recorder.enter_quote();
                Status::Completed
            }
            b':' => {
                let value = self.pop();
                self.push(value);
                self.push(value);
                Status::Completed
            }
            b'\\' => {
                let upper = self.pop();
                let lower = self.pop();
                self.push(upper);
                self.push(lower);
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
                let value = self.space.get_cell(Position { x: lower, y: upper });
                self.push(value.0);
                Status::Completed
            }
            b'p' => {
                let upper = self.pop();
                let middle = self.pop();
                let lower = self.pop();
                self.put(
                    Position {
                        x: middle,
                        y: upper,
                    },
                    Cell(lower),
                );
                Status::Completed
            }
            b'&' => {
                if let Some(input_number) = self.io.read_number() {
                    self.push(input_number);
                    Status::Completed
                } else {
                    Status::Waiting
                }
            }
            b'~' => {
                if let Some(input) = self.io.read_byte() {
                    self.push(input);
                    Status::Completed
                } else {
                    Status::Waiting
                }
            }
            b'@' => Status::Terminated,
            b'0' => {
                self.push(0);
                Status::Completed
            }
            b'1' => {
                self.push(1);
                Status::Completed
            }
            b'2' => {
                self.push(2);
                Status::Completed
            }
            b'3' => {
                self.push(3);
                Status::Completed
            }
            b'4' => {
                self.push(4);
                Status::Completed
            }
            b'5' => {
                self.push(5);
                Status::Completed
            }
            b'6' => {
                self.push(6);
                Status::Completed
            }
            b'7' => {
                self.push(7);
                Status::Completed
            }
            b'8' => {
                self.push(8);
                Status::Completed
            }
            b'9' => {
                self.push(9);
                Status::Completed
            }
            b' ' => Status::Completed,
            op => {
                eprintln!("Invalid opcode: {}", op);
                return Status::Error(InterpreterError::InvalidOpcode(op));
            }
        };
        if status == Status::Completed {
            self.move_auto()
        }
        status
    }

    fn skip_spaces(&mut self) -> Option<Status> {
        let start = self.cursor.pos;
        loop {
            if self.space.get_cell(self.cursor.pos).0 != b' ' {
                return None;
            }

            self.move_auto();

            if self.cursor.pos == start {
                eprintln!("Infinite loop detected at {:?}", start);
                return Some(Status::Error(InterpreterError::InfiniteLoop));
            }
        }
    }
}
