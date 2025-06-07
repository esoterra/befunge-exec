use std::collections::HashSet;

use crate::{
    analyze::{self, PathAnalysis},
    core::Position,
    interpreter::{Interpreter, Record},
    io::VecIO,
    space::Space,
};

pub struct Debugger {
    #[allow(dead_code)]
    program: Vec<u8>,
    pub analysis: PathAnalysis,
    pub interpreter: Interpreter<VecIO>,
    pub breakpoints: HashSet<Position>,
    pub timeline: Timeline,

    state: State,
    ticks_per_step: u16,
    ticks_since_step: u16,
}

enum State {
    Paused,
    Stepping { steps: u16 },
    Running,
}

impl Debugger {
    pub fn new(program: Vec<u8>) -> Self {
        let space = Space::new(&program);
        let analysis = analyze::analyze_path(&space);
        let interpreter = Interpreter::new(space, VecIO::default());
        Self {
            program,
            analysis,
            interpreter,
            breakpoints: Default::default(),
            timeline: Default::default(),

            state: State::Paused,
            ticks_per_step: 20,
            ticks_since_step: 0,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.ticks_since_step += 1;
        let time_for_step = self.ticks_since_step > self.ticks_per_step;

        let step_now = match self.state {
            State::Paused => false,
            State::Stepping { steps: 1 } => {
                if time_for_step {
                    self.state = State::Paused;
                }
                time_for_step
            }
            State::Stepping { steps } => {
                if time_for_step {
                    self.state = State::Stepping { steps: steps - 1 }
                }
                time_for_step
            }
            State::Running => time_for_step,
        };

        if step_now {
            self.ticks_since_step = 0;
            let pos = self.interpreter.current_position();
            if self.breakpoints.contains(&pos) {
                self.state = State::Paused;
            } else {
                eprintln!("Step");
                self.interpreter.step(&mut self.timeline);
            }
        }
        step_now
    }

    pub fn add_steps(&mut self, steps: u16) {
        self.state = match self.state {
            State::Stepping { steps: current } => State::Stepping {
                steps: current + steps,
            },
            _ => State::Stepping { steps },
        };
    }

    pub fn start_running(&mut self) {
        self.state = State::Running;
    }

    pub fn pause(&mut self) {
        self.state = State::Paused;
    }

    pub fn toggle_breakpoint(&mut self, pos: Position) {
        if !self.breakpoints.remove(&pos) {
            self.breakpoints.insert(pos);
        }
    }

    pub fn stack_height(&self) -> u16 {
        self.interpreter.stack().len() as u16
    }

    pub fn current_position(&self) -> Position {
        self.interpreter.current_position()
    }
}

#[derive(Default)]
pub struct Timeline {
    steps: Vec<Step>,
    events: Vec<Event>,

    pending_events: u8,
}

/// Events contain enough information to apply them to the state either forwards or backwards.
#[allow(dead_code)]
enum Event {
    Replace { at: Position, old: u8, new: u8 },
    Pop { old: u8 },
    PopBottom,
    Push { new: u8 },
    EnterQuote,
    ExitQuote,
}

#[allow(dead_code)]
struct Step {
    at: Position,
    instruction: u8,
    events: u8,
}

impl Record for Timeline {
    fn start_step(&mut self, at: Position, instruction: u8) {
        self.steps.push(Step {
            at,
            instruction,
            events: 0,
        });
    }

    fn rollback_step(&mut self) {
        self.steps.pop();
        self.pending_events = 0;
    }

    fn commit_step(&mut self) {
        self.steps.last_mut().unwrap().events = self.pending_events;
        self.pending_events = 0;
    }

    fn replace(&mut self, at: Position, old: u8, new: u8) {
        self.events.push(Event::Replace { at, old, new });
    }

    fn pop(&mut self, old: u8) {
        self.events.push(Event::Pop { old });
    }

    fn pop_bottom(&mut self) {
        self.events.push(Event::PopBottom);
    }

    fn push(&mut self, new: u8) {
        self.events.push(Event::Push { new });
    }

    fn enter_quote(&mut self) {
        self.events.push(Event::EnterQuote);
    }

    fn exit_quote(&mut self) {
        self.events.push(Event::ExitQuote);
    }
}
