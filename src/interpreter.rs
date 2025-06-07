use crate::core::{Cell, Cursor, Direction, Mode, Position};
use crate::io::IO;
use crate::space::Space;

#[derive(PartialEq, Eq, Clone, Debug)]
/// An Interpreter represents a step by step executor for befunge code.
/// It contains a program, all necessary state, and IO buffers.
pub struct Interpreter<IOImpl> {
    space: Space<Cell>,

    cursor: Cursor,
    stack: Vec<Cell>,

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

pub trait Record {
    fn start_step(&mut self, at: Position, instruction: u8);
    fn finish_step(&mut self);

    fn replace(&mut self, at: Position, old: u8, new: u8);
    fn pop(&mut self, old: u8);
    fn pop_bottom(&mut self);
    fn push(&mut self, new: u8);
    fn enter_quote(&mut self);
    fn exit_quote(&mut self);
}

impl Record for () {
    fn start_step(&mut self, _at: Position, _instruction: u8) {}
    fn finish_step(&mut self) {}

    fn replace(&mut self, _at: Position, _old: u8, _new: u8) {}
    fn pop(&mut self, _old: u8) {}
    fn pop_bottom(&mut self) {}
    fn push(&mut self, _new: u8) {}
    fn enter_quote(&mut self) {}
    fn exit_quote(&mut self) {}
}

impl<IOImpl> Interpreter<IOImpl>
where
    IOImpl: IO,
{
    /// Creates a new Interpreter that executes
    /// the provided program with the provided io.
    pub fn new(space: Space<Cell>, io: IOImpl) -> Self {
        let cursor = Cursor {
            pos: Position::ORIGIN,
            dir: Direction::Right,
            mode: Mode::Normal,
        };

        Interpreter {
            space,
            cursor,
            stack: Vec::new(),
            io,
        }
    }

    pub fn io(&mut self) -> &mut IOImpl {
        &mut self.io
    }

    pub fn space(&self) -> &Space<Cell> {
        &self.space
    }

    /// Get the position of the cursor
    pub fn current_position(&self) -> Position {
        self.cursor.pos
    }

    /// Get the direction of teh cursor
    pub fn current_direction(&self) -> Direction {
        self.cursor.dir
    }

    /// Get the current stack contents
    #[cfg(test)]
    pub fn stack(&self) -> &[Cell] {
        &self.stack[..]
    }

    fn put(&mut self, pos: Position, cell: Cell, recorder: &mut impl Record) {
        let old = self.space.get_cell(pos);
        recorder.replace(pos, old.0, cell.0);
        self.space.set_cell(pos, cell);
    }

    fn move_auto(&mut self) {
        let pos = self.current_position();
        let dir = self.current_direction();
        self.cursor.pos = self.space.move_pos(pos, dir);
    }

    fn pop(&mut self, recorder: &mut impl Record) -> u8 {
        match self.stack.pop() {
            Some(top) => {
                recorder.pop(top.0);
                top.0
            }
            None => {
                recorder.pop_bottom();
                0
            }
        }
    }

    fn push(&mut self, cell: u8, recorder: &mut impl Record) {
        recorder.push(cell);
        self.stack.push(Cell(cell));
    }

    /// Interprets the next command
    pub fn step(&mut self, recorder: &mut impl Record) -> Status {
        let cell = self.space.get_cell(self.cursor.pos);

        match self.cursor.mode {
            Mode::Quote => self.step_quoted(cell, recorder),
            Mode::Normal => self.step_unquoted(cell, recorder),
        }
    }

    fn step_quoted(&mut self, cell: Cell, recorder: &mut impl Record) -> Status {
        match cell {
            Cell(b'"') => {
                self.cursor.mode = Mode::Normal;
                recorder.exit_quote();
            }
            _ => self.stack.push(cell),
        }
        self.move_auto();
        Status::Completed
    }

    fn step_unquoted(&mut self, cell: Cell, recorder: &mut impl Record) -> Status {
        use std::num::Wrapping;

        let status = match cell.0 {
            b'+' => {
                let (e1, e2) = (self.pop(recorder), self.pop(recorder));
                recorder.pop(e1);
                recorder.pop(e2);
                let result = Wrapping(e2) + Wrapping(e1);
                self.push(result.0, recorder);
                Status::Completed
            }
            b'-' => {
                let upper = self.pop(recorder);
                let lower = self.pop(recorder);
                let result = Wrapping(lower) - Wrapping(upper);
                self.push(result.0, recorder);
                Status::Completed
            }
            b'*' => {
                let (e1, e2) = (self.pop(recorder), self.pop(recorder));
                let result = Wrapping(e2) * Wrapping(e1);
                self.push(result.0, recorder);
                Status::Completed
            }
            b'/' => {
                let upper = self.pop(recorder);
                let lower = self.pop(recorder);
                let result = Wrapping(lower) / Wrapping(upper);
                self.push(result.0, recorder);
                Status::Completed
            }
            b'%' => {
                let upper = self.pop(recorder);
                let lower = self.pop(recorder);
                let result = Wrapping(lower) % Wrapping(upper);
                self.push(result.0, recorder);
                Status::Completed
            }
            b'!' => {
                let value = self.pop(recorder);
                let result = if value == 0 { 1 } else { 0 };
                self.push(result, recorder);
                Status::Completed
            }
            b'`' => {
                let upper = self.pop(recorder);
                let lower = self.pop(recorder);
                let result = if lower > upper { 1 } else { 0 };
                self.push(result, recorder);
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
                self.cursor.dir = if self.pop(recorder) == 0 {
                    Direction::Right
                } else {
                    Direction::Left
                };
                Status::Completed
            }
            b'|' => {
                self.cursor.dir = if self.pop(recorder) == 0 {
                    Direction::Down
                } else {
                    Direction::Up
                };
                Status::Completed
            }
            b'"' => {
                self.cursor.mode = Mode::Quote;
                recorder.enter_quote();
                Status::Completed
            }
            b':' => {
                let value = self.pop(recorder);
                self.push(value, recorder);
                self.push(value, recorder);
                Status::Completed
            }
            b'\\' => {
                let upper = self.pop(recorder);
                let lower = self.pop(recorder);
                self.push(upper, recorder);
                self.push(lower, recorder);
                Status::Completed
            }
            b'$' => {
                self.pop(recorder);
                Status::Completed
            }
            b'.' => {
                let number_string = format!("{} ", self.pop(recorder));
                let buf = number_string.as_bytes();
                self.io.write(buf);
                Status::Completed
            }
            b',' => {
                let buf = &[self.pop(recorder)];
                self.io.write(buf);
                Status::Completed
            }
            b'#' => {
                self.move_auto();
                Status::Completed
            }
            b'g' => {
                let upper = self.pop(recorder);
                let lower = self.pop(recorder);
                let value = self.space.get_cell(Position { x: lower, y: upper });
                self.push(value.0, recorder);
                Status::Completed
            }
            b'p' => {
                let upper = self.pop(recorder);
                let middle = self.pop(recorder);
                let lower = self.pop(recorder);
                self.put(
                    Position {
                        x: middle,
                        y: upper,
                    },
                    Cell(lower),
                    recorder,
                );
                Status::Completed
            }
            b'&' => {
                if let Some(input_number) = self.io.read_number() {
                    self.push(input_number, recorder);
                    Status::Completed
                } else {
                    Status::Waiting
                }
            }
            b'~' => {
                if let Some(input) = self.io.read_byte() {
                    self.push(input, recorder);
                    Status::Completed
                } else {
                    Status::Waiting
                }
            }
            b'@' => Status::Terminated,
            b'0' => {
                self.push(0, recorder);
                Status::Completed
            }
            b'1' => {
                self.push(1, recorder);
                Status::Completed
            }
            b'2' => {
                self.push(2, recorder);
                Status::Completed
            }
            b'3' => {
                self.push(3, recorder);
                Status::Completed
            }
            b'4' => {
                self.push(4, recorder);
                Status::Completed
            }
            b'5' => {
                self.push(5, recorder);
                Status::Completed
            }
            b'6' => {
                self.push(6, recorder);
                Status::Completed
            }
            b'7' => {
                self.push(7, recorder);
                Status::Completed
            }
            b'8' => {
                self.push(8, recorder);
                Status::Completed
            }
            b'9' => {
                self.push(9, recorder);
                Status::Completed
            }
            b' ' => Status::Completed,
            op => {
                eprintln!("Invalid opcode: {}", op);
                std::process::exit(1);
            }
        };
        match status {
            Status::Completed => self.move_auto(),
            Status::Waiting => {}
            Status::Terminated => {}
        }
        status
    }
}
