#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// Represents a 2d position in the program space
pub struct Position {
    /// The x dimension
    /// Corresponds to the column, indexed left to right.
    pub x: usize,
    /// The y dimension
    /// Corresponds to the row, indexed from top to bottom.
    pub y: usize
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
    Right
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// The mode of the program
pub enum Mode {
    /// Quotation mode
    /// Each command that is not Quote (") is interpreted
    /// as a push of its own ascii value
    Quote,
    /// Normal mode
    /// Commands are interpretted as opcodes
    Normal
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Cursor {
    pub pos: Position,
    pub dir: Direction,
    pub mode: Mode
}