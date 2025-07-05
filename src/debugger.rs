use std::collections::HashSet;

use crate::{
    analyze::{self, PathAnalysis},
    core::Position,
    interpreter::{Interpreter, Status},
    record::Timeline,
    space::Space,
    terminal::VirtualTerminal,
};

pub struct Debugger {
    #[allow(dead_code)]
    program: Vec<u8>,
    pub analysis: PathAnalysis,
    pub interpreter: Interpreter<VirtualTerminal, Timeline>,
    pub breakpoints: HashSet<Position>,

    state: State,
    ticks_per_step: u16,
    ticks_since_step: u16,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum State {
    Paused,
    Stepping { steps: u16 },
    Running,
    Halted,
}

impl Debugger {
    pub fn new(program: Vec<u8>) -> Self {
        let space = Space::new(&program);
        let analysis = analyze::analyze_path(&space);
        let interpreter = Interpreter::new(space, VirtualTerminal::default(), Timeline::default());
        Self {
            program,
            analysis,
            interpreter,
            breakpoints: Default::default(),

            state: State::Paused,
            ticks_per_step: 2,
            ticks_since_step: 0,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.ticks_since_step += 1;
        let time_for_step = self.ticks_since_step > self.ticks_per_step;

        let step_now = match self.state {
            State::Paused | State::Halted => false,
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
                let status = self.interpreter.step();
                match status {
                    Status::Completed => {}
                    Status::Waiting => {}
                    Status::Terminated => self.state = State::Halted,
                    Status::Error(interpreter_error) => log::error!("{}", interpreter_error),
                }
            }
        }
        step_now
    }

    pub fn add_steps(&mut self, steps: u16) {
        self.state = match self.state {
            State::Halted => State::Halted,
            State::Stepping { steps: current } => State::Stepping {
                steps: current + steps,
            },
            _ => State::Stepping { steps },
        };
    }

    pub fn start_running(&mut self) {
        if self.state == State::Halted {
            return;
        }
        self.state = State::Running;
    }

    pub fn pause(&mut self) {
        if self.state == State::Halted {
            return;
        }
        self.state = State::Paused;
    }

    pub fn io(&self) -> &VirtualTerminal {
        self.interpreter.io()
    }

    pub fn io_mut(&mut self) -> &mut VirtualTerminal {
        self.interpreter.io_mut()
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
