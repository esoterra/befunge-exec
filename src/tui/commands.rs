use core::fmt;
use crossterm::event::{KeyCode, KeyEvent};
use std::borrow::Cow;
use thiserror::Error;

use crate::core::Position;

pub struct CommandsView {
    pub output: Cow<'static, str>,
    pub input_contents: String,
    pub input_cursor: u16,
    pub mode: Mode,
}

pub enum Mode {
    Running,
    Stepping,
    Stopped,
}

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    Step,
    Run,
    Breakpoint { pos: Position },
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Help => write!(f, "Help 'h'/'help' command"),
            Command::Step => write!(f, "Step 's' command"),
            Command::Run => write!(f, "Run 'r' command"),
            Command::Breakpoint { pos } => write!(f, "Breakpoint 'b {} {}' command", pos.x, pos.y),
        }
    }
}

impl Default for CommandsView {
    fn default() -> Self {
        Self {
            output: Default::default(),
            input_contents: Default::default(),
            input_cursor: 0,
            mode: Mode::Stopped,
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
                        Command::Step => {
                            self.mode = Mode::Stepping;
                            self.output = Cow::Borrowed("Taking a step.");
                        }
                        Command::Run => {
                            self.mode = Mode::Running;
                            self.output = Cow::Borrowed("Running...");
                        }
                        Command::Breakpoint { pos } => {}
                    }
                },
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
                "r" => (Command::Run, 0),
                "s" => (Command::Step, 0),
                "b" => {
                    let command = Command::Breakpoint { pos: todo!() };
                    (command, 2)
                },
                // "exit" => std::process::exit(0),
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

const HELP_OUTPUT: &str =
    "Help: r - runs the program\n      s - takes a step\n      'b <x> <y>' - places a breakpoint";

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