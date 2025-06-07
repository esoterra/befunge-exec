#![allow(unused)]

use std::io::{self, Write};

use crossterm::{QueueableCommand, cursor::MoveRight};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Text<S> {
    contents: S,
    width: u16,
}

impl<S: AsRef<str>> AsRef<str> for Text<S> {
    fn as_ref(&self) -> &str {
        self.contents.as_ref()
    }
}

impl<S> Text<S> {
    pub const fn width(&self) -> u16 {
        self.width
    }
}

pub const fn t(s: &str) -> Text<&str> {
    assert!(s.is_ascii());
    let width = s.len() as u16;
    Text { contents: s, width }
}

pub const fn tw<S: AsRef<str>>(s: S, w: u16) -> Text<S> {
    Text {
        contents: s,
        width: w,
    }
}

pub fn tc(c: char) -> Text<String> {
    Text {
        contents: format!("{}", c),
        width: 1,
    }
}

pub const fn tn(n: u8) -> Text<&'static str> {
    let numbers = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
    debug_assert!(n < 10);
    t(numbers[n as usize])
}

const SOURCE_UNITS: u16 = 256;

pub struct SliceSource<const STRIDE: u16, S> {
    contents: S,
}

impl<const STRIDE: u16, S> SliceSource<STRIDE, S>
where
    S: AsRef<str>,
{
    fn slice(&self, n: u16) -> &str {
        debug_assert!(n <= SOURCE_UNITS);
        let base = self.contents.as_ref();
        let len = (STRIDE * n) as usize;
        &base[0..len]
    }

    pub fn text(&self, n: u16) -> Text<&str> {
        tw(self.slice(n), n)
    }
}

const fn ts<const STRIDE: u16>(contents: &str) -> SliceSource<STRIDE, &str> {
    let needed = SOURCE_UNITS * STRIDE;
    let needed = needed as usize;
    let found = contents.len();
    assert!(found == needed);
    SliceSource { contents }
}

pub struct Spaces;

pub trait PrintN {
    fn one(&self) -> &str;

    fn print_n(&self, stdout: &mut io::Stdout, n: u16) -> io::Result<()>;
}

impl<const STRIDE: u16, S> PrintN for SliceSource<STRIDE, S>
where
    S: AsRef<str>,
{
    fn one(&self) -> &str {
        self.slice(1)
    }

    fn print_n(&self, stdout: &mut io::Stdout, n: u16) -> io::Result<()> {
        write!(stdout, "{}", self.slice(n))
    }
}

impl PrintN for Spaces {
    fn one(&self) -> &str {
        " "
    }

    fn print_n(&self, stdout: &mut io::Stdout, n: u16) -> io::Result<()> {
        if n != 0 {
            stdout.queue(MoveRight(n))?;
        }
        Ok(())
    }
}

pub type StaticText = Text<&'static str>;
pub type StaticSource<const STRIDE: u16> = SliceSource<STRIDE, &'static str>;

// The "Befunge Debugger" heading and potentially the tabbing hint
pub const CONSOLE_TAB_FRAME_NORMAL: [StaticText; 3] = [
    tw("╟──────────────────╔═════════╗──────────┬──────────┬", 52),
    tw("║                  ║         ║          │          │", 52),
    tw("╠══════════════════╝─────────╚══════════╧══════════╧", 52),
];

pub const COMMANDS_TAB_FRAME_NORMAL: [StaticText; 3] = [
    tw("╟──────────────────┬─────────╔══════════╗──────────┬", 52),
    tw("║                  │         ║          ║          │", 52),
    tw("╠══════════════════╧═════════╝──────────╚══════════╧", 52),
];

pub const TIMELINE_FRAME_NORMAL: [StaticText; 3] = [
    tw("╟──────────────────┬─────────┬──────────╔══════════╗", 52),
    tw("║                  │         │          ║          ║", 52),
    tw("╠══════════════════╧═════════╧══════════╝──────────╚", 52),
];

pub const CONSOLE_TAB_FRAME_NORMAL_TIGHT: [StaticText; 3] = [
    tw("╟──────────────────╔═════════╗──────────┬──────────", 51),
    tw("║                  ║         ║          │          ", 51),
    tw("╠══════════════════╝─────────╚══════════╧══════════", 51),
];

pub const COMMANDS_TAB_FRAME_NORMAL_TIGHT: [StaticText; 3] = [
    tw("╟──────────────────┬─────────╔══════════╗──────────", 51),
    tw("║                  │         ║          ║          ", 51),
    tw("╠══════════════════╧═════════╝──────────╚══════════", 51),
];

pub const TIMELINE_FRAME_NORMAL_TIGHT: [StaticText; 3] = [
    tw("╟──────────────────┬─────────┬──────────╔══════════", 51),
    tw("║                  │         │          ║          ", 51),
    tw("╠══════════════════╧═════════╧══════════╝──────────", 51),
];

// Just tabs
pub const CONSOLE_TAB_FRAME_NARROW: [StaticText; 3] = [
    tw("╠═════════╗──────────┬──────────╢", 33),
    tw("║         ║          │          ║", 33),
    tw("║─────────╚══════════╧══════════╣", 33),
];

pub const COMMANDS_TAB_FRAME_NARROW: [StaticText; 3] = [
    tw("╟─────────╔══════════╗──────────╢", 33),
    tw("║         ║          ║          ║", 33),
    tw("╠═════════╝──────────╚══════════╣", 33),
];

pub const TIMELINE_TAB_FRAME_NARROW: [StaticText; 3] = [
    tw("╟─────────┬──────────╔══════════╣", 33),
    tw("║         │          ║          ║", 33),
    tw("╠═════════╧══════════╝──────────║", 33),
];

pub const BEFUNGE_DEBUGGER: StaticText = t("Befunge Debugger");
pub const CONSOLE: StaticText = t("Console");
pub const COMMANDS: StaticText = t("Commands");
pub const TIMELINE: StaticText = t("Timeline");
pub const TAB_SWITCH_HINT: StaticText = t("switch using [shift] tab");

#[allow(clippy::if_same_then_else)]
pub fn sidebar(i: u16, rows: u16, even: bool, collapse: bool) -> StaticText {
    if i == 0 {
        tw("║       ║", 9)
    } else if i == 1 {
        tw("╟───┬───╢", 9)
    } else {
        let row_last = rows - 1;
        if collapse {
            debug_assert!(rows > 10);
            let pre_collapse = rows - 5;
            let collapse_top = rows - 4;
            let collapse_mid = rows - 3;
            let collapse_bot = rows - 2;
            if even && i == pre_collapse {
                tw("╟───┴───╢", 9)
            } else if even && i == collapse_top {
                tw("╟───────╢", 9)
            } else if i == collapse_top {
                tw("╟───┴───╢", 9)
            } else if i == collapse_mid {
                tw("║[     ]║", 9)
            } else if i == collapse_bot {
                tw("╟───┬───╢", 9)
            } else if even && i == row_last {
                tw("║   │   ║", 9)
            } else if i % 2 == 0 {
                tw("║   │   ║", 9)
            } else {
                tw("╟───┼───╢", 9)
            }
        } else {
            if even && i == row_last {
                tw("╟───┴───╢", 9)
            } else if i % 2 == 0 {
                tw("║   │   ║", 9)
            } else {
                tw("╟───┼───╢", 9)
            }
        }
    }
}

pub struct TabSidebar {
    pub top: StaticText,
    pub mid: StaticText,
    pub bot: StaticText,
}

pub fn tabs_sidebar(tight: bool, tab: bool, even: bool) -> TabSidebar {
    match (tight, tab, even) {
        (true, true, true) => TabSidebar {
            top: tw("╣───────╢", 9),
            mid: tw("║       ║", 9),
            bot: tw("╠═══════╣", 9),
        },
        (true, true, false) => TabSidebar {
            top: tw("╣───┴───╢", 9),
            mid: tw("║       ║", 9),
            bot: tw("╠═══════╣", 9),
        },
        (true, false, true) => TabSidebar {
            top: tw("╫───────╢", 9),
            mid: tw("║       ║", 9),
            bot: tw("╬═══════╣", 9),
        },
        (true, false, false) => TabSidebar {
            top: tw("╫───┴───╢", 9),
            mid: tw("║       ║", 9),
            bot: tw("╬═══════╣", 9),
        },
        (false, _, true) => TabSidebar {
            top: tw("╨───────╢", 9),
            mid: tw("║", 1),
            bot: tw("╦═══════╣", 9),
        },
        (false, _, false) => TabSidebar {
            top: tw("╨───┴───╢", 9),
            mid: tw("║", 1),
            bot: tw("╦═══════╣", 9),
        },
    }
}

#[allow(non_upper_case_globals)]
pub const SCROllBAR_SOLID: StaticText = tw("█", 1);
#[allow(non_upper_case_globals)]
pub const SCROllBAR_EMPTY: StaticText = tw("░", 1);

pub const SOLID: StaticSource<3> = ts(
    "████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████",
);

pub const EMPTY: StaticSource<3> = ts(
    "░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░",
);

pub const PIPES: StaticSource<3> = ts(
    "════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════",
);
pub const LINES: StaticSource<3> = ts(
    "────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────",
);

pub const SPACES: Spaces = Spaces;
