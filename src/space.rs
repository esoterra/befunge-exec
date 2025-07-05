use std::collections::HashMap;

use grid::Grid;

use crate::core::{Direction, Position};

/// The program space
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Space<Cell> {
    grid: Grid<Cell>,
    map: HashMap<Position, Cell>,
    rows: usize,
    cols: usize,
}

impl<Cell> Space<Cell>
where
    Cell: From<u8> + Default,
{
    pub fn new(program: &[u8]) -> Self {
        let mut cols = 0;
        let mut rows = 0;
        let mut last_line = 0;
        for (i, c) in program.iter().enumerate() {
            if *c == b'\n' {
                cols = std::cmp::max(i - last_line, cols);
                last_line = i + 1;
                rows += 1;
            }
        }
        if last_line != program.len() {
            cols = std::cmp::max(program.len() - last_line, cols);
            rows += 1;
        }

        let mut grid = Grid::new(rows, cols);

        let mut last_line = 0;
        let mut row = 0;
        for (i, c) in program.iter().enumerate() {
            if *c == b'\n' {
                last_line = i + 1;
                row += 1;
                continue;
            }
            grid[(row, i - last_line)] = Cell::from(*c);
        }

        Self {
            grid,
            map: HashMap::new(),
            cols,
            rows,
        }
    }
}

impl<Cell> Space<Cell>
where
    Cell: Copy + Default,
{
    pub fn with_size(rows: u16, cols: u16) -> Self {
        let rows = rows as usize;
        let cols = cols as usize;
        let grid = Grid::new(rows, cols);
        Self {
            grid,
            map: Default::default(),
            rows,
            cols,
        }
    }

    pub fn rows(&self) -> u16 {
        self.rows as u16
    }

    pub fn cols(&self) -> u16 {
        self.cols as u16
    }

    /// Retrieves the cell located at a position in the program
    pub fn get_cell(&self, pos: Position) -> Cell {
        self.lookup_cell(pos).copied().unwrap_or_default()
    }
}

impl<Cell> Space<Cell> {
    /// Gets a reference to the specified cell if it exists
    pub fn lookup_cell(&self, pos: Position) -> Option<&Cell> {
        let x = pos.x as usize;
        let y = pos.y as usize;
        if x >= self.grid.cols() || y >= self.grid.rows() {
            self.map.get(&pos)
        } else {
            self.grid.get(y, x)
        }
    }

    /// Updates the opcode at a specific position in the program
    pub fn set_cell(&mut self, pos: Position, cell: Cell) {
        let x = pos.x as usize;
        let y = pos.y as usize;
        if x >= self.grid.cols() || y >= self.grid.rows() {
            // eprintln!("Insert into map {:?} -> {:?}", pos, cell);
            self.map.insert(pos, cell);
        } else {
            // eprintln!("Insert into grid ({}, {}) -> {:?}", x, y, cell);
            self.grid[(y, x)] = cell;
        }
        self.cols = std::cmp::max(self.cols, x + 1);
        self.rows = std::cmp::max(self.rows, y + 1);
    }

    pub fn move_pos(&self, pos: Position, dir: Direction) -> Position {
        let Position { x, y } = pos;
        let cols = self.cols as u8;
        let rows = self.rows as u8;
        match dir {
            Direction::Right => {
                let x = x + 1;
                let x = if x >= cols { 0 } else { x };
                Position { x, y }
            }
            Direction::Left => {
                let x = if x == 0 { cols } else { x - 1 };
                Position { x, y }
            }
            Direction::Up => {
                let y = if y == 0 { rows } else { y - 1 };
                Position { x, y }
            }
            Direction::Down => {
                let y = y + 1;
                let y = if y >= rows { 0 } else { y };
                Position { x, y }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_origin() {
        let mut space: Space<u8> = Space::with_size(100, 100);
        space.set_cell(Position::ORIGIN, 2);
        assert_eq!(space.get_cell(Position::ORIGIN), 2);
    }

    #[test]
    fn test_insert_unit_square() {
        let mut space: Space<u8> = Space::with_size(100, 100);
        // insert 0 at 0,0
        let pos = Position::ORIGIN;
        space.set_cell(pos, 0);
        assert_eq!(space.get_cell(pos), 0);
        // insert 1 at 1,0
        let pos = Position { x: 1, y: 0 };
        space.set_cell(pos, 1);
        assert_eq!(space.get_cell(pos), 1);
        // insert 2 at 1,1
        let pos = Position { x: 1, y: 1 };
        space.set_cell(pos, 2);
        assert_eq!(space.get_cell(pos), 2);
        // insert 3 at 0,1
        let pos = Position { x: 0, y: 1 };
        space.set_cell(pos, 3);
        assert_eq!(space.get_cell(pos), 3);
    }

    #[test]
    fn test_insert_one_one() {
        let mut space: Space<u8> = Space::with_size(2, 2);
        // insert 2 at 1,1
        let pos = Position { x: 2, y: 1 };
        space.set_cell(pos, 2);
        assert_eq!(space.get_cell(pos), 2);
    }

    #[test]
    fn test_insert_outside() {
        let mut space: Space<u8> = Space::with_size(10, 10);
        // insert 2 at 1,1
        let pos = Position { x: 20, y: 20 };
        space.set_cell(pos, 2);
        assert_eq!(space.get_cell(pos), 2);
    }
}
