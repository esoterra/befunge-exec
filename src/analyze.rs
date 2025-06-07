use core::fmt;
use std::collections::VecDeque;

use crate::{
    core::{Cell, Direction, Mode, Position},
    space::Space,
};

pub fn analyze_path(space: &Space<Cell>) -> PathAnalysis {
    PathAnalysisState::new(space).analyze()
}

pub struct PathAnalysis {
    pub cell_states: Space<State>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct State(u8);

// Quoted: Up, Down, Left, and Right masks
const QU_MASK: u8 = 0b0001;
const QD_MASK: u8 = 0b0010;
const QL_MASK: u8 = 0b0100;
const QR_MASK: u8 = 0b1000;
// Normal: Up, Down, Left, and Right masks
const NU_MASK: u8 = 0b00010000;
const ND_MASK: u8 = 0b00100000;
const NL_MASK: u8 = 0b01000000;
const NR_MASK: u8 = 0b10000000;
// Masks of all quoted or all unquoted bits
const Q_MASK: u8 = 0b00001111;
const N_MASK: u8 = 0b11110000;

struct Unquoted<'a>(&'a str);

impl fmt::Debug for Unquoted<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ds = f.debug_set();
        if (self.0 & QU_MASK) != 0 {
            ds.entry(&Unquoted("QU"));
        }
        if (self.0 & QD_MASK) != 0 {
            ds.entry(&Unquoted("QD"));
        }
        if (self.0 & QL_MASK) != 0 {
            ds.entry(&Unquoted("QL"));
        }
        if (self.0 & QR_MASK) != 0 {
            ds.entry(&Unquoted("QR"));
        }
        if (self.0 & NU_MASK) != 0 {
            ds.entry(&Unquoted("NU"));
        }
        if (self.0 & ND_MASK) != 0 {
            ds.entry(&Unquoted("ND"));
        }
        if (self.0 & NL_MASK) != 0 {
            ds.entry(&Unquoted("NL"));
        }
        if (self.0 & NR_MASK) != 0 {
            ds.entry(&Unquoted("NR"));
        }
        ds.finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modes {
    None,
    Quoted,
    Normal,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Directions {
    None,
    Horizontal,
    Vertical,
    Both,
}

impl State {
    pub fn modes(self) -> Modes {
        let normal = (self.0 & N_MASK) != 0;
        let quoted = (self.0 & Q_MASK) != 0;
        match (normal, quoted) {
            (true, true) => Modes::Both,
            (true, false) => Modes::Normal,
            (false, true) => Modes::Quoted,
            (false, false) => Modes::None,
        }
    }

    pub fn directions(self) -> Directions {
        let u = (self.0 & (NU_MASK | QU_MASK)) != 0;
        let d = (self.0 & (ND_MASK | QD_MASK)) != 0;
        let l = (self.0 & (NL_MASK | QL_MASK)) != 0;
        let r = (self.0 & (NR_MASK | QR_MASK)) != 0;
        match (u, d, l, r) {
            (false, false, false, false) => Directions::None,
            (false, false, true, _) => Directions::Horizontal,
            (false, false, false, true) => Directions::Horizontal,
            (true, _, false, false) => Directions::Vertical,
            (false, true, false, false) => Directions::Vertical,
            _ => Directions::Both,
        }
    }

    fn update(self, dir: Direction, mode: Mode) -> Self {
        let mask = match (dir, mode) {
            (Direction::Up, Mode::Quote) => QU_MASK,
            (Direction::Up, Mode::Normal) => NU_MASK,
            (Direction::Down, Mode::Quote) => QD_MASK,
            (Direction::Down, Mode::Normal) => ND_MASK,
            (Direction::Left, Mode::Quote) => QL_MASK,
            (Direction::Left, Mode::Normal) => NL_MASK,
            (Direction::Right, Mode::Quote) => QR_MASK,
            (Direction::Right, Mode::Normal) => NR_MASK,
        };
        Self(self.0 | mask)
    }
}

struct PathAnalysisState<'src> {
    space: &'src Space<Cell>,
    states: Space<State>,
    queue: VecDeque<(Position, Direction, Mode)>,
}

impl<'src> PathAnalysisState<'src> {
    fn new(space: &'src Space<Cell>) -> Self {
        let states: Space<State> = Space::with_size(space.rows(), space.cols());
        let mut queue: VecDeque<(Position, Direction, Mode)> = Default::default();
        queue.push_back((Position::ORIGIN, Direction::Right, Mode::Normal));
        Self {
            space,
            states,
            queue,
        }
    }

    fn analyze(mut self) -> PathAnalysis {
        while let Some((pos, dir, mode)) = self.queue.pop_front() {
            let cell = self.space.get_cell(pos);

            // Fake out the mode so that quotes always show as quoted
            let draw_mode = match (cell.0, mode) {
                (b'"', _) => Mode::Quote,
                (_, mode) => mode,
            };
            let old = self.states.get_cell(pos);
            let new = old.update(dir, draw_mode);
            if old == new {
                continue;
            }
            self.states.set_cell(pos, new);

            // Actually update the mode
            let mode = match (cell.0, mode) {
                (b'"', Mode::Quote) => Mode::Normal,
                (b'"', Mode::Normal) => Mode::Quote,
                (_, mode) => mode,
            };

            if mode == Mode::Quote {
                self.forward(pos, dir, mode);
                continue;
            }

            match cell.0 {
                b'^' => {
                    self.up(pos, mode);
                }
                b'v' => {
                    self.down(pos, mode);
                }
                b'<' => {
                    self.left(pos, mode);
                }
                b'>' => {
                    self.right(pos, mode);
                }
                b'?' => {
                    self.up(pos, mode);
                    self.down(pos, mode);
                    self.left(pos, mode);
                    self.right(pos, mode);
                }
                b'|' => {
                    self.up(pos, mode);
                    self.down(pos, mode);
                }
                b'_' => {
                    self.left(pos, mode);
                    self.right(pos, mode);
                }
                b'#' => {
                    let pos = self.space.move_pos(pos, dir);
                    let pos = self.space.move_pos(pos, dir);
                    self.queue.push_back((pos, dir, mode));
                }
                b'@' => {
                    continue;
                }
                _ => {
                    self.forward(pos, dir, mode);
                }
            }
        }
        PathAnalysis {
            cell_states: self.states,
        }
    }

    fn forward(&mut self, pos: Position, dir: Direction, mode: Mode) {
        let pos = self.space.move_pos(pos, dir);
        self.queue.push_back((pos, dir, mode));
    }

    fn up(&mut self, pos: Position, mode: Mode) {
        let up = self.space.move_pos(pos, Direction::Up);
        self.queue.push_back((up, Direction::Up, mode));
    }

    fn down(&mut self, pos: Position, mode: Mode) {
        let down = self.space.move_pos(pos, Direction::Down);
        self.queue.push_back((down, Direction::Down, mode));
    }

    fn left(&mut self, pos: Position, mode: Mode) {
        let pos = self.space.move_pos(pos, Direction::Left);
        self.queue.push_back((pos, Direction::Left, mode));
    }

    fn right(&mut self, pos: Position, mode: Mode) {
        let pos = self.space.move_pos(pos, Direction::Right);
        self.queue.push_back((pos, Direction::Right, mode));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_masks() {
        let state = State::default();
        assert_eq!(state.directions(), Directions::None);
        assert_eq!(state.modes(), Modes::None);
        let state = state.update(Direction::Right, Mode::Normal);
        assert_eq!(state.directions(), Directions::Horizontal);
        assert_eq!(state.modes(), Modes::Normal);
    }
}
