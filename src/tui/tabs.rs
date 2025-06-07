#![allow(unused)]

use core::fmt;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use std::{borrow::Cow, io};
use thiserror::Error;

use crate::{
    core::Position,
    tui::{
        ListenForKey, ListenForMouse, Window,
        draw::{Dimensions, ProgramView},
    },
};

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
#[derive(Default)]
pub struct ConsoleView {
    pub scroll_height: u16,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CommandsView {
    pub output: Cow<'static, str>,
    pub input_contents: String,
    pub input_cursor: u16,
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

    pub fn move_to_cursor(&self, window: &mut Window) -> io::Result<()> {
        let Dimensions { cols, rows } = ProgramView::dimensions(window);
        let (x, y) = match self.focused {
            FocusedTab::Console => (1, rows + 4),
            FocusedTab::Commands => (self.commands.input_cursor + 4, window.height() - 2),
            FocusedTab::Timeline => (1, window.height() - 2),
        };
        window.move_to(x, y)?;
        Ok(())
    }
}

impl ListenForKey for Tabs {
    type Output = Option<CommandEvent>;

    fn on_key_event(&mut self, event: KeyEvent) -> Self::Output {
        match event {
            KeyEvent {
                code: KeyCode::BackTab,
                ..
            } => {
                self.focus_previous();
                None
            }
            KeyEvent {
                code: KeyCode::Tab, ..
            } => {
                self.focus_next();
                None
            }
            _ => match self.focused {
                FocusedTab::Console => None,
                FocusedTab::Commands => {
                    self.dirty = true;
                    self.commands.on_key_event(event)
                }
                FocusedTab::Timeline => None,
            },
        }
    }
}

impl ListenForMouse for Tabs {
    type Output = ();

    fn on_mouse_event(&mut self, event: MouseEvent, window: &Window) -> Self::Output {
        if matches!(event.kind, MouseEventKind::Down(_)) {
            let Dimensions { rows, cols } = ProgramView::dimensions(window);
            let tab_min_row = rows + 2;
            let tab_max_row = tab_min_row + 2;
            if event.row >= tab_min_row && event.row <= tab_max_row {
                // ║ Befunge Debugger ║ Console ║ Commands │ Timeline │
                //                     20      28           41       50
                //                               30       39
                match event.column {
                    20..=28 => {
                        if self.focused != FocusedTab::Console {
                            self.focused = FocusedTab::Console;
                            self.dirty = true;
                        }
                    }
                    30..=39 => {
                        if self.focused != FocusedTab::Commands {
                            self.focused = FocusedTab::Commands;
                            self.dirty = true;
                        }
                    }
                    41..=50 => {
                        if self.focused != FocusedTab::Timeline {
                            self.focused = FocusedTab::Timeline;
                            self.dirty = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}


#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    Load { path: String },
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
            Command::Load { path } => write!(f, "Load '{}'", path),
            Command::Step { n } => write!(f, "Step {}", *n),
            Command::Run => write!(f, "Run"),
            Command::Pause => write!(f, "Pause"),
            Command::Breakpoint { pos } => write!(f, "Breakpoint at {}", pos),
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
        }
    }
}

pub enum CommandEvent {
    Load { path: String },
    Step { n: u16 },
    Run,
    Pause,
    Breakpoint { pos: Position },
    Quit,
}

impl ListenForKey for CommandsView {
    type Output = Option<CommandEvent>;

    fn on_key_event(&mut self, event: KeyEvent) -> Self::Output {
        match event {
            KeyEvent {
                code: KeyCode::Left,
                ..
            } => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                }
                None
            }
            KeyEvent {
                code: KeyCode::Right,
                ..
            } => {
                let max_cursor = (self.input_contents.len() - 1) as u16;
                if self.input_cursor < max_cursor {
                    self.input_cursor += 1;
                }
                None
            }
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                if self.input_cursor == 0 {
                    return None;
                }
                let remove_char = self.input_cursor - 1;
                self.input_contents.remove(remove_char as usize);
                self.input_cursor -= 1;
                None
            }
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => match self.parse_command() {
                Ok(None) => None,
                Ok(Some(command)) => {
                    self.input_contents.clear();
                    self.input_cursor = 0;
                    match command {
                        Command::Help => {
                            self.output = Cow::Borrowed(HELP_OUTPUT);
                            None
                        }
                        Command::Load { path } => {
                            self.output = Cow::Owned(format!("Loading {}", path));
                            Some(CommandEvent::Load { path })
                        }
                        Command::Step { n } => {
                            self.output = match n {
                                1 => Cow::Owned(format!("Taking {} steps", n)),
                                _ => Cow::Borrowed("Taking 1 step"),
                            };
                            Some(CommandEvent::Step { n })
                        }
                        Command::Run => {
                            self.output = Cow::Borrowed("Running...");
                            Some(CommandEvent::Run)
                        }
                        Command::Pause => {
                            self.output = Cow::Borrowed("Paused");
                            Some(CommandEvent::Pause)
                        }
                        Command::Breakpoint { pos } => {
                            self.output = Cow::Owned(format!("Setting breakpoint at {}", pos));
                            Some(CommandEvent::Breakpoint { pos })
                        }
                        Command::Quit => Some(CommandEvent::Quit),
                    }
                }
                Err(error) => {
                    let error_string = error.to_string();
                    eprintln!("{}", error_string);
                    self.output = Cow::Owned(error_string);
                    None
                }
            },
            KeyEvent { code, .. } => {
                if let Some(c) = code.as_char() {
                    self.input_contents.insert(self.input_cursor as usize, c);
                    self.input_cursor += 1;
                }
                None
            }
        }
    }
}

impl CommandsView {
    fn parse_command(&mut self) -> Result<Option<Command>, CommandError> {
        let mut args = self.input_contents.split(' ');
        if let Some(first) = args.next() {
            let (command, expected) = match first {
                "h" | "help" => (Command::Help, 0),
                "l" | "load" => {
                    let path = match args.next() {
                        Some(arg) => String::from(arg),
                        None => {
                            return Err(CommandError::TooFewArguments {
                                command: Command::Load { path: "".into() },
                                expected: 1,
                            });
                        }
                    };
                    (Command::Load { path }, 1)
                }
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
                    let x = match args.next() {
                        Some(arg) => arg.parse().unwrap(),
                        None => {
                            return Err(CommandError::TooFewArguments {
                                command: Command::Breakpoint {
                                    pos: Default::default(),
                                },
                                expected: 2,
                            });
                        }
                    };
                    let y = match args.next() {
                        Some(arg) => arg.parse().unwrap(),
                        None => {
                            return Err(CommandError::TooFewArguments {
                                command: Command::Load { path: "".into() },
                                expected: 2,
                            });
                        }
                    };
                    let command = Command::Breakpoint {
                        pos: Position { x, y },
                    };
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
        for arg in args {
            v.push(arg);
        }
        Some(v)
    } else {
        None
    }
}
