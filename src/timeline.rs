#![allow(unused)]
use crate::core::{Direction, Position};

pub struct Timeline {
    last_event: usize,
    ticks: Vec<Tick>,
    events: Vec<Event>
}

/// Events contain enough information to apply them to the state either forwards or backwards.
enum Event {
    Turn {
        from: Direction,
        to: Direction,
    },
    Replace {
        at: Position,
        old: u8,
        new: u8,
    },
    Pop {
        old: u8,
    },
    Push {
        new: u8,
    }
}

struct Tick {
    from: Position,
    to: Position,
    instruction: u8,
    events: u8
}