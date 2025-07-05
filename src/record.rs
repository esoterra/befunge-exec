use crate::core::Position;

pub trait Record {
    fn start_step(&mut self, at: Position, instruction: u8);
    fn rollback_step(&mut self);
    fn commit_step(&mut self);

    fn replace(&mut self, at: Position, old: u8, new: u8);
    fn pop(&mut self, old: u8);
    fn pop_bottom(&mut self);
    fn push(&mut self, new: u8);
    fn enter_quote(&mut self);
    fn exit_quote(&mut self);
}

impl Record for () {
    fn start_step(&mut self, _at: Position, _instruction: u8) {}
    fn rollback_step(&mut self) {}
    fn commit_step(&mut self) {}

    fn replace(&mut self, _at: Position, _old: u8, _new: u8) {}
    fn pop(&mut self, _old: u8) {}
    fn pop_bottom(&mut self) {}
    fn push(&mut self, _new: u8) {}
    fn enter_quote(&mut self) {}
    fn exit_quote(&mut self) {}
}

impl<T1, T2> Record for (T1, T2)
where
    T1: Record,
    T2: Record,
{
    fn start_step(&mut self, at: Position, instruction: u8) {
        self.0.start_step(at, instruction);
        self.1.start_step(at, instruction);
    }

    fn rollback_step(&mut self) {
        self.0.rollback_step();
        self.1.rollback_step();
    }

    fn commit_step(&mut self) {
        self.0.commit_step();
        self.1.commit_step();
    }

    fn replace(&mut self, at: Position, old: u8, new: u8) {
        self.0.replace(at, old, new);
        self.1.replace(at, old, new);
    }

    fn pop(&mut self, old: u8) {
        self.0.pop(old);
        self.1.pop(old);
    }

    fn pop_bottom(&mut self) {
        self.0.pop_bottom();
        self.1.pop_bottom();
    }

    fn push(&mut self, new: u8) {
        self.0.push(new);
        self.1.push(new);
    }

    fn enter_quote(&mut self) {
        self.0.enter_quote();
        self.1.enter_quote();
    }

    fn exit_quote(&mut self) {
        self.0.exit_quote();
        self.1.exit_quote();
    }
}

pub struct StdOutEventLog;

impl Record for StdOutEventLog {
    fn start_step(&mut self, at: Position, instruction: u8) {
        println!("Started step at {} with opcode '{}'", at, instruction);
    }

    fn rollback_step(&mut self) {
        println!("Rollback step");
    }

    fn commit_step(&mut self) {
        println!("Commit step");
    }

    fn replace(&mut self, at: Position, old: u8, new: u8) {
        println!("Replace '{}' with '{}' at {}", old, new, at);
    }

    fn pop(&mut self, old: u8) {
        println!("Popped '{}' from stack", old);
    }

    fn pop_bottom(&mut self) {
        println!("Popped while at bottom of stack")
    }

    fn push(&mut self, new: u8) {
        println!("Pushed '{}' onto the stack", new);
    }

    fn enter_quote(&mut self) {
        println!("Enter quote mode");
    }

    fn exit_quote(&mut self) {
        println!("Exit quote mode")
    }
}

pub struct StdErrEventLog;

impl Record for StdErrEventLog {
    fn start_step(&mut self, at: Position, instruction: u8) {
        eprintln!("Started step at {} with opcode '{}'", at, instruction);
    }

    fn rollback_step(&mut self) {
        eprintln!("Rollback step");
    }

    fn commit_step(&mut self) {
        eprintln!("Commit step");
    }

    fn replace(&mut self, at: Position, old: u8, new: u8) {
        eprintln!("Replace '{}' with '{}' at {}", old, new, at);
    }

    fn pop(&mut self, old: u8) {
        eprintln!("Popped '{}' from stack", old);
    }

    fn pop_bottom(&mut self) {
        eprintln!("Popped while at bottom of stack")
    }

    fn push(&mut self, new: u8) {
        eprintln!("Pushed '{}' onto the stack", new);
    }

    fn enter_quote(&mut self) {
        eprintln!("Enter quote mode");
    }

    fn exit_quote(&mut self) {
        eprintln!("Exit quote mode")
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
