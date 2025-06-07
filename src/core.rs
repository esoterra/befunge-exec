use core::fmt;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Default)]
/// Represents a 2d position in the program space
pub struct Position {
    /// The x dimension
    /// Corresponds to the column, indexed left to right.
    pub x: u8,
    /// The y dimension
    /// Corresponds to the row, indexed from top to bottom.
    pub y: u8,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Position {
    pub const ORIGIN: Position = Position { x: 0, y: 0 };
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// A direction in the 2d program space
pub enum Direction {
    /// The negative y direction
    Up,
    /// The positive y direction
    Down,
    /// The negative x direction
    Left,
    /// The positive x direction
    Right,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// The mode of the program
pub enum Mode {
    /// Quotation mode
    /// Each command that is not a double quote (") is interpreted as a push of its own ascii value.
    /// The double quote command returns the cursor to normal mode
    Quote,
    /// Normal mode
    /// Commands are interpreted as opcodes
    Normal,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// A Cursor represents the necessary information to
/// understand how to execute the next opcode
pub struct Cursor {
    /// The position of the cursor
    pub pos: Position,
    /// The direction the cursor is going
    pub dir: Direction,
    /// The mode of the cursor
    pub mode: Mode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell(pub u8);

impl Default for Cell {
    fn default() -> Self {
        Self(b' ')
    }
}

impl From<u8> for Cell {
    fn from(value: u8) -> Self {
        Cell(value)
    }
}
