use crate::{analyze::{self, PathAnalysis}, core::Position, interpreter::{Interpreter, Record}, io::VecIO, space::Space};



pub struct Debugger {
    program: Vec<u8>,
    pub analysis: PathAnalysis,
    pub interpreter: Interpreter<VecIO>,
    pub timeline: Timeline,

    pub state: State,
    pub ticks_per_step: u16,
    pub ticks_since_step: u16,
}

enum State {
    Paused,
    Stepping { steps: u16 },
    Running
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
            timeline: Timeline::default(),

            state: State::Paused,
            ticks_per_step: 20,
            ticks_since_step: 0,
        }
    }

    pub fn tick(&mut self) {
        self.ticks_since_step += 1;
        if self.ticks_since_step > self.ticks_per_step {
            self.ticks_since_step = 0;
            self.interpreter.step(&mut self.timeline);
        }
    }

    pub fn current_position(&self) -> Position {
        self.interpreter.current_position()
    }
}

#[derive(Default)]
pub struct Timeline {
    step_offset: usize,
    event_offset: usize,
    
    steps: Vec<Step>,
    events: Vec<Event>,

    pending_events: u8,
}

/// Events contain enough information to apply them to the state either forwards or backwards.
enum Event {
    Replace { at: Position, old: u8, new: u8 },
    Pop { old: u8 },
    PopBottom,
    Push { new: u8 },
    EnterQuote,
    ExitQuote,
}

struct Step {
    at: Position,
    instruction: u8,
    events: u8,
}

impl Record for Timeline {
    fn start_step(&mut self, at: Position, instruction: u8) {
        self.steps.push(Step { at, instruction, events: 0 });
    }

    fn finish_step(&mut self) {
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
