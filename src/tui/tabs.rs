#![allow(unused)]

use core::fmt;
use crossterm::event::{KeyCode, KeyEvent};
use std::borrow::Cow;
use thiserror::Error;

use crate::core::Position;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Tabs {
    pub focused: FocusedTab,

    pub has_tabbed: bool,
    pub has_back_tabbed: bool,

    pub console: ConsoleView,
    pub commands: CommandsView,
    pub timeline: TimelineView,
    pub position: Position,

    pub dirty: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FocusedTab {
    Console,
    #[default]
    Commands,
    Timeline,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConsoleView {
    pub scroll_height: u16,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CommandsView {
    pub output: Cow<'static, str>,
    pub input_contents: String,
    pub input_cursor: u16,
    pub mode: Mode,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Mode {
    Running,
    Stepping { n: u16 },
    Paused,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TimelineView;

impl Tabs {
    pub fn on_key_event(&mut self, event: KeyEvent) {
        match event {
            KeyEvent {
                code: KeyCode::BackTab,
                ..
            } => {
                self.focus_previous();
            }
            KeyEvent {
                code: KeyCode::Tab, ..
            } => {
                self.focus_next();
            }
            _ => match self.focused {
                FocusedTab::Console => {}
                FocusedTab::Commands => {
                    self.commands.on_key_event(event);
                    self.dirty = true;
                }
                FocusedTab::Timeline => {}
            },
        }
    }

    fn focus_next(&mut self) {
        self.has_tabbed = true;
        self.focused = match self.focused {
            FocusedTab::Console => FocusedTab::Commands,
            FocusedTab::Commands => FocusedTab::Timeline,
            FocusedTab::Timeline => FocusedTab::Console,
        };
        self.dirty = true;
    }

    fn focus_previous(&mut self) {
        self.has_back_tabbed = true;
        self.focused = match self.focused {
            FocusedTab::Console => FocusedTab::Timeline,
            FocusedTab::Commands => FocusedTab::Console,
            FocusedTab::Timeline => FocusedTab::Commands,
        };
        self.dirty = true;
    }

    pub fn has_tabbed_both_ways(&self) -> bool {
        self.has_back_tabbed && self.has_tabbed
    }
}

impl Default for ConsoleView {
    fn default() -> Self {
        Self { scroll_height: 0 }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    Step { n: u16 },
    Run,
    Pause,
    Breakpoint { pos: Position },
    Quit,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Help => write!(f, "Help"),
            Command::Step { n } => write!(f, "Step {}", *n),
            Command::Run => write!(f, "Run"),
            Command::Pause => write!(f, "Pause"),
            Command::Breakpoint { pos } => write!(f, "Breakpoint at ({}, {})", pos.x, pos.y),
            Command::Quit => write!(f, "Quit"),
        }
    }
}

impl Default for CommandsView {
    fn default() -> Self {
        Self {
            output: Cow::Borrowed(HELP_OUTPUT),
            input_contents: Default::default(),
            input_cursor: 0,
            mode: Mode::Paused,
        }
    }
}

impl CommandsView {
    pub fn on_key_event(&mut self, event: KeyEvent) {
        match event {
            KeyEvent {
                code: KeyCode::Left,
                ..
            } => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                }
            }
            KeyEvent {
                code: KeyCode::Right,
                ..
            } => {
                let max_cursor = (self.input_contents.len() - 1) as u16;
                if self.input_cursor < max_cursor {
                    self.input_cursor += 1;
                }
            }
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                if self.input_cursor == 0 {
                    return;
                }
                let remove_char = self.input_cursor - 1;
                self.input_contents.remove(remove_char as usize);
                self.input_cursor -= 1;
            }
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => match self.parse_command() {
                Ok(None) => {}
                Ok(Some(command)) => {
                    self.input_contents.clear();
                    self.input_cursor = 0;
                    match command {
                        Command::Help => {
                            self.output = Cow::Borrowed(HELP_OUTPUT);
                        }
                        Command::Step { n } => {
                            if let Mode::Stepping { n: old } = self.mode {
                                let total = old + n;
                                self.output = match n {
                                    1 => {
                                        Cow::Owned(format!("Taking 1 more step ({} total)", total))
                                    }
                                    _ => Cow::Owned(format!(
                                        "Taking {} more steps ({} total)",
                                        n, total
                                    )),
                                };
                                self.mode = Mode::Stepping { n: total };
                            } else {
                                self.output = match n {
                                    1 => Cow::Owned(format!("Taking {} steps", n)),
                                    _ => Cow::Borrowed("Taking 1 step"),
                                };
                                self.mode = Mode::Stepping { n };
                            }
                        }
                        Command::Run => {
                            self.mode = Mode::Running;
                            self.output = Cow::Borrowed("Running...");
                        }
                        Command::Pause => {
                            self.mode = Mode::Paused;
                            self.output = Cow::Borrowed("Paused");
                        }
                        Command::Breakpoint { pos } => {}
                        Command::Quit => {}
                    }
                }
                Err(error) => {
                    let error_string = error.to_string();
                    eprintln!("{}", error_string);
                    self.output = Cow::Owned(error_string);
                }
            },
            KeyEvent { code, .. } => {
                if let Some(c) = code.as_char() {
                    self.input_contents.insert(self.input_cursor as usize, c);
                    self.input_cursor += 1;
                }
            }
        }
    }

    fn parse_command(&mut self) -> Result<Option<Command>, CommandError> {
        let mut args = self.input_contents.split(' ');
        if let Some(first) = args.next() {
            let (command, expected) = match first {
                "h" | "help" => (Command::Help, 0),
                "s" | "step" => {
                    if let Some(arg) = args.next() {
                        let n = arg.parse().unwrap();
                        (Command::Step { n }, 1)
                    } else {
                        (Command::Step { n: 1 }, 0)
                    }
                }
                "r" | "run" => (Command::Run, 0),
                "p" | "pause" => (Command::Pause, 0),
                "b" | "breakpoint" => {
                    let command = Command::Breakpoint { pos: todo!() };
                    (command, 2)
                }
                "q" | "quit" => (Command::Quit, 0),
                "" => return Ok(None),
                arg => return Err(CommandError::UnknownCommand { arg }),
            };
            if let Some(found) = try_collect(args) {
                return Err(CommandError::TooManyArguments {
                    command,
                    expected,
                    found,
                });
            }
            Ok(Some(command))
        } else {
            Ok(None)
        }
    }
}

#[derive(Error, Debug)]
enum CommandError<'a> {
    #[error("error: {command} accepts {expected} arguments, but found {} extra: {:?}", .found.len(), .found)]
    TooManyArguments {
        command: Command,
        expected: u16,
        found: Vec<&'a str>,
    },
    #[error("error: {command}")]
    TooFewArguments { command: Command, expected: u16 },
    #[error("error: unknown command alias '{arg}'")]
    UnknownCommand { arg: &'a str },
}

const HELP_OUTPUT: &str = "step  │ s [n]     │ takes a step\nrun   │ r [speed] │ runs the program\npause │ p         │ pauses the execution\nbreak │ b <x> <y> │ places a breakpoint\nquit  │ q         │ exits the debugger";

fn try_collect<'a>(mut args: impl Iterator<Item = &'a str>) -> Option<Vec<&'a str>> {
    if let Some(arg) = args.next() {
        let mut v = vec![arg];
        while let Some(arg) = args.next() {
            v.push(arg);
        }
        Some(v)
    } else {
        None
    }
}
