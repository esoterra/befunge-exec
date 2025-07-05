use crate::{
    analyze::{self, Directions},
    core::Position,
    debugger::Debugger,
    terminal::VirtualTerminal,
    tui::{
        Tui,
        layout::{self, ProgramX, ProgramY, SidebarX, SidebarY, TabHeadingY, TabY, program_cols},
        styles,
        tabs::{CommandsView, ConsoleView, FocusedTab, Tabs, TimelineView},
        text::{self, t, tw},
        window::{ConvertToWindowSpace, Window, WindowX, WindowY, window_coord},
    },
};

use core::str;
use std::io;

pub trait DrawBorder {
    fn draw_border(&self, window: &mut Window) -> io::Result<()>;
}

pub trait Draw {
    fn draw(&self, window: &mut Window) -> io::Result<()>;
}

impl DrawBorder for Tui {
    fn draw_border(&self, window: &mut Window) -> io::Result<()> {
        window.set_style(styles::BORDER)?;
        let (x, y) = window_coord(0, 0);
        window.move_to(x, y)?;

        if self.show_sidebar(window) {
            window.line(tw("╔", 1), text::PIPES, tw("╦═══════╗", 9))?;
        } else if self.show_outer_border(window) {
            window.line(tw("╔", 1), text::PIPES, tw("╗", 1))?;
        }

        self.draw_border_main(window)?;
        self.tabs.draw_border(window)?;

        self.draw_border_last(window)?;
        Ok(())
    }
}

impl Draw for Tui {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        StackHeading.draw(window)?;
        ProgramDisplay {
            debugger: &self.debugger,
        }
        .draw(window)?;
        Sidebar {
            debugger: &self.debugger,
        }
        .draw(window)?;
        (self.debugger.io(), &self.tabs).draw(window)?;
        Ok(())
    }
}

pub fn stack_slots(window: &Window) -> u16 {
    let rows = layout::stack_rows(window);
    if rows % 2 == 0 {
        // -3 is for the Stack header and dead row
        // / 2 is because each element requires a divider
        (rows - 3) / 2
    } else {
        // -2 is for the Stack header
        // / 2 is because each element requires a divider
        (rows - 2) / 2
    }
}

impl Tui {
    fn draw_border_main(&self, window: &mut Window) -> io::Result<()> {
        let rows = layout::program_rows(window);
        if self.show_sidebar(window) {
            let even_parity = layout::stack_rows_parity_even(window);
            let collapse = self.debugger.stack_height() > stack_slots(window);
            for i in 0..rows {
                let sidebar = text::sidebar(i, rows, even_parity, collapse);
                window.line(tw("║", 1), text::SPACES, sidebar)?;
            }
        } else {
            for _ in 0..rows {
                window.line(tw("║", 1), text::SPACES, tw("║", 1))?;
            }
        }

        Ok(())
    }

    pub fn draw_border_last(&self, window: &mut Window) -> io::Result<()> {
        if self.show_sidebar(window) {
            window.line(tw("╚", 1), text::PIPES, tw("╩═══════╝", 9))?;
        } else if self.show_outer_border(window) {
            window.line(tw("╚", 1), text::PIPES, tw("╝", 1))?;
        }
        Ok(())
    }
}

pub struct Sidebar<'d> {
    pub debugger: &'d Debugger,
}

impl DrawBorder for Sidebar<'_> {
    fn draw_border(&self, window: &mut Window) -> io::Result<()> {
        let rows = layout::stack_rows(window);
        let even_parity = layout::stack_rows_parity_even(window);
        let collapse = self.debugger.stack_height() > stack_slots(window);
        window.set_style(styles::BORDER)?;
        for i in 0..rows {
            let sidebar = text::sidebar(i, rows, even_parity, collapse);
            window.move_to(SidebarX(0), SidebarY(i))?;
            window.clear_until_newline()?;
            window.print(sidebar)?;
        }
        Ok(())
    }
}

impl Draw for Sidebar<'_> {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        StackHeading.draw(window)?;

        let even_parity = layout::stack_rows_parity_even(window);
        let room = stack_slots(window);
        let stack_height = self.debugger.stack_height();
        window.set_style(styles::CYAN_HEADING)?;

        let number_x = SidebarX(1);
        let symbol_x = SidebarX(5);
        let last_y = SidebarY::max(window);

        if stack_height > room {
            let skipped = stack_height - room;
            let skip_x = SidebarX(2);

            // Draw bottom value
            let bottom = &self.debugger.interpreter.stack()[0];
            window.move_to(number_x, last_y)?;
            window.print(t(&format!("{}", bottom.0)))?;
            window.move_to(symbol_x, last_y)?;
            if let Some(label) = value_label(bottom.0) {
                window.print(t(label))?;
            } else {
                if let Some(c) = char::from_u32(bottom.0 as u32) {
                    window.print_char('"')?;
                    window.print_char(c)?;
                    window.print_char('"')?;
                }
            }

            // Draw skip count
            window.move_to(skip_x, last_y - 2)?;
            window.print(t(&format!("{}", skipped)))?;

            // Draw top values
            let mut y = if even_parity { last_y - 5 } else { last_y - 4 };
            let top_start = (skipped + 1) as usize;
            let stack_top = &self.debugger.interpreter.stack()[top_start..];
            for cell in stack_top.iter() {
                window.move_to(number_x, y)?;
                window.print(t(&format!("{}", cell.0)))?;
                window.move_to(symbol_x, y)?;
                if let Some(label) = value_label(cell.0) {
                    window.print(t(label))?;
                } else {
                    if let Some(c) = char::from_u32(cell.0 as u32) {
                        window.print_char('"')?;
                        window.print_char(c)?;
                        window.print_char('"')?;
                    }
                }
                y = y - 2;
            }
        } else {
            let mut y = if even_parity { last_y - 1 } else { last_y };
            // Draw values
            for cell in self.debugger.interpreter.stack().iter() {
                window.move_to(number_x, y)?;
                window.print(t(&format!("{}", cell.0)))?;
                window.move_to(symbol_x, y)?;
                if let Some(label) = value_label(cell.0) {
                    window.print(t(label))?;
                } else {
                    if let Some(c) = char::from_u32(cell.0 as u32) {
                        window.print_char('"')?;
                        window.print_char(c)?;
                        window.print_char('"')?;
                    }
                }
                y = y - 2;
            }
        }
        Ok(())
    }
}

fn value_label(value: u8) -> Option<&'static str> {
    // based on https://www.ascii-code.com/
    let code = match value {
        // ASCII Control Characters
        b'\x00' => "NUL",
        b'\x01' => "SOH",
        b'\x02' => "STX",
        b'\x03' => "ETX",
        b'\x04' => "EOT",
        b'\x05' => "ENQ",
        b'\x06' => "ACK",
        b'\x07' => "BEL",
        b'\x08' => "BS",
        b'\x09' => "HT",
        b'\x0A' => "LF",
        b'\x0B' => "VT",
        b'\x0C' => "FF",
        b'\x0D' => "CR",
        b'\x0E' => "SO",
        b'\x0F' => "SI",
        b'\x10' => "DLE",
        b'\x11' => "DC1",
        b'\x12' => "DC2",
        b'\x13' => "DC3",
        b'\x14' => "DC4",
        b'\x15' => "NAK",
        b'\x16' => "SYN",
        b'\x17' => "ETB",
        b'\x18' => "CAN",
        b'\x19' => "EM",
        b'\x1A' => "SUB",
        b'\x1B' => "ESC",
        b'\x1C' => "FS",
        b'\x1D' => "GS",
        b'\x1E' => "RS",
        b'\x1F' => "US",
        // ASCII Printable Characters
        b'\x20' => "SP",
        b'\x22' => " \" ",
        b'\x27' => " \' ",
        b'\x7F' => "DEL",

        _ => return None,
    };
    Some(code)
}

pub const WIDE_WIDTH: u16 = 80;

/// Show title, tabs, hint, and sidebar
/// ║ Befunge Debugger ║ Console ║ Commands │ Timeline │  switch using [shift] tab  ║ <- 81
/// Range: w > 80
///
/// Show title, tabs, and sidebar
/// ║ Befunge Debugger ║ Console ║ Commands │ Timeline ║ <- 52
/// Range: 80 >= w > 51
///
/// Show tabs
/// ║ Console ║ Commands │ Timeline ║ <- 33
/// Range: 51 >= w > 32
///
/// Don't show any tab section or headings
/// ║                   ║ <- 21
/// Range: 32 > w
impl DrawBorder for Tabs {
    fn draw_border(&self, window: &mut Window) -> io::Result<()> {
        let tight = window.width() == 60;
        let heading_lines = {
            if tight {
                match self.focused {
                    FocusedTab::Console => text::CONSOLE_TAB_FRAME_NORMAL_TIGHT,
                    FocusedTab::Commands => text::COMMANDS_TAB_FRAME_NORMAL_TIGHT,
                    FocusedTab::Timeline => text::TIMELINE_FRAME_NORMAL_TIGHT,
                }
            } else {
                match self.focused {
                    FocusedTab::Console => text::CONSOLE_TAB_FRAME_NORMAL,
                    FocusedTab::Commands => text::COMMANDS_TAB_FRAME_NORMAL,
                    FocusedTab::Timeline => text::TIMELINE_FRAME_NORMAL,
                }
            }
        };

        let text::TabSidebar { top, mid, bot } = {
            let tab = self.focused == FocusedTab::Timeline;
            let collapse_stack = false; // TODO
            let even = layout::stack_rows_parity_even(window) && !collapse_stack;
            text::tabs_sidebar(tight, tab, even)
        };

        window.line(heading_lines[0], text::LINES, top)?;
        window.line(heading_lines[1], text::SPACES, mid)?;
        window.line(heading_lines[2], text::PIPES, bot)?;

        match self.focused {
            FocusedTab::Console => self.console.draw_border(window),
            FocusedTab::Commands => self.commands.draw_border(window),
            FocusedTab::Timeline => self.timeline.draw_border(window),
        }
    }
}

impl<'a> Draw for (&'a VirtualTerminal, &'a Tabs) {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        let (term, tabs) = self;
        TabHeadings {
            tab: tabs.focused,
            tabbed_both_ways: tabs.has_tabbed_both_ways(),
        }
        .draw(window)?;

        CatLogo.draw(window)?;

        CursorDisplay { pos: tabs.position }.draw(window)?;

        // We draw the tab contents last so the cursor is left
        // on the focused input prompt
        match tabs.focused {
            FocusedTab::Console => {
                tabs.console.draw(window)?;
                term.draw(window)
            }
            FocusedTab::Commands => tabs.commands.draw(window),
            FocusedTab::Timeline => tabs.timeline.draw(window),
        }
    }
}

impl DrawBorder for ConsoleView {
    fn draw_border(&self, window: &mut Window) -> io::Result<()> {
        window.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("╠═══════╣", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("╠═══════╣", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        Ok(())
    }
}

impl Draw for ConsoleView {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        let cols = layout::program_cols(window);
        let x = WindowX(cols);
        let y = TabY(0).convert(window);
        let total = 7;
        let bar = 1;
        let offset = 0;
        VerticalScrollbar {
            x,
            y,
            total,
            bar,
            offset,
        }
        .draw(window)
    }
}

impl Draw for VirtualTerminal {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        window.set_style(styles::PROGRAM_TEXT)?;
        let cols = layout::program_cols(window) as usize;
        let num_lines = self.num_lines();
        let start = if num_lines > 7 { num_lines - 7 } else { 0 };
        VirtualTerminalDisplay {
            cols,
            num_lines,
            start,
            term: self,
        }
        .draw(window)
    }
}

struct VirtualTerminalDisplay<'t> {
    cols: usize,
    start: usize,
    num_lines: usize,
    term: &'t VirtualTerminal,
}

impl Draw for VirtualTerminalDisplay<'_> {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        let n = self.num_lines - self.start;
        for i in 0..n {
            self.draw_line(i, window)?;
        }
        Ok(())
    }
}

impl VirtualTerminalDisplay<'_> {
    fn draw_line(&self, i: usize, window: &mut Window) -> io::Result<()> {
        // Find the line to print or return if there is none
        let line_index = self.start + i;
        let Some(line) = self.term.get_line(line_index) else {
            return Ok(());
        };
        // Slice it to the correct length
        let line_len = std::cmp::min(line.len(), self.cols);
        let line = &line[0..line_len];
        // Move to the right position and write the line
        let y = TabY(i as u16);
        window.move_to(WindowX(1), y)?;
        window.write(line)?;

        // Draw uncommitted if necessary
        if line_index == self.num_lines - 1 {
            self.draw_uncommitted(y, line_len, window)?;
        }
        Ok(())
    }

    fn draw_uncommitted(&self, y: TabY, line_len: usize, window: &mut Window) -> io::Result<()> {
        // Retrieve and truncate uncommitted buffer if necessary
        let buf = self.term.uncommitted();
        let buf_len = std::cmp::min(buf.len(), self.cols - line_len);
        let buf = &buf[0..buf_len];
        // Move to just after the line and write
        let after_line = WindowX(1 + (line_len as u16));
        window.move_to(after_line, y)?;
        window.write(buf)
    }
}

impl DrawBorder for CommandsView {
    fn draw_border(&self, window: &mut Window) -> io::Result<()> {
        window.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("╠═══════╣", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("╠═══════╣", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        window.line(tw("╠", 1), text::PIPES, tw("╣       ║", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        Ok(())
    }
}

impl Draw for CommandsView {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        // Draw command output
        window.set_style(styles::PROGRAM_TEXT)?;
        let max_width = program_cols(window) as usize;
        for (i, line) in (0..5).zip(self.output.lines()) {
            let x = WindowX(1);
            let y = TabY(i);
            window.move_to(x, y)?;
            let line = if line.len() > max_width {
                &line[0..max_width]
            } else {
                line
            };
            window.print(tw(line, line.chars().count() as u16))?;
        }

        // Draw command input
        let prompt_y = TabY(6);
        window.move_to(WindowX(2), prompt_y)?;
        window.set_style(styles::CYAN_HEADING)?;
        window.print(t("$ "))?;
        window.set_style(styles::PROGRAM_TEXT)?;
        let buf = self.input_contents.to_string();
        window.print(t(&buf))?;
        window.move_to(WindowX(4 + self.input_cursor), prompt_y)?;
        Ok(())
    }
}

impl DrawBorder for TimelineView {
    fn draw_border(&self, window: &mut Window) -> io::Result<()> {
        window.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("╠═══════╣", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("╠═══════╣", 9))?;
        window.line(tw("╠", 1), text::PIPES, tw("╣       ║", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        window.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        Ok(())
    }
}

impl Draw for TimelineView {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        let x = WindowX(1);
        let y = TabY(5).convert(window);
        let total = program_cols(window);
        let bar = 0;
        let offset = 0;
        HorizontalScrollbar {
            x,
            y,
            total,
            bar,
            offset,
        }
        .draw(window)
    }
}

struct StackHeading;

impl Draw for StackHeading {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        window.move_to(SidebarX(2), SidebarY(0))?;
        window.set_style(styles::CYAN_HEADING)?;
        window.print(t("Stack"))?;
        Ok(())
    }
}

struct TabHeadings {
    tab: FocusedTab,
    tabbed_both_ways: bool,
}

impl Draw for TabHeadings {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        window.move_to(WindowX(2), TabHeadingY(1))?;
        window.set_style(styles::CYAN_HEADING)?;
        window.print(text::BEFUNGE_DEBUGGER)?;

        window.set_style(styles::tab_heading(FocusedTab::Console, self.tab))?;
        window.move_right(3)?;
        window.print(text::CONSOLE)?;

        window.set_style(styles::tab_heading(FocusedTab::Commands, self.tab))?;
        window.move_right(3)?;
        window.print(text::COMMANDS)?;

        window.set_style(styles::tab_heading(FocusedTab::Timeline, self.tab))?;
        window.move_right(3)?;
        window.print(text::TIMELINE)?;

        if window.width() > WIDE_WIDTH && !self.tabbed_both_ways {
            window.move_right(4)?;
            window.set_style(styles::GRAY_HEADING)?;
            window.print(text::TAB_SWITCH_HINT)?;
        }

        Ok(())
    }
}

struct HorizontalScrollbar {
    x: WindowX,
    y: WindowY,
    total: u16,
    bar: u16,
    offset: u16,
}

impl HorizontalScrollbar {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        window.set_style(styles::CYAN_HEADING)?;
        window.move_to(self.x, self.y)?;
        let pre = self.offset;
        let mid = self.bar;
        let end = self.total - pre - mid;
        window.print(text::EMPTY.text(pre))?;
        window.print(text::SOLID.text(mid))?;
        window.print(text::EMPTY.text(end))?;
        Ok(())
    }
}

struct VerticalScrollbar {
    /// x position in overall window space
    x: WindowX,
    /// y position in overall window space
    y: WindowY,
    total: u16,
    bar: u16,
    offset: u16,
}

impl VerticalScrollbar {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        window.set_style(styles::CYAN_HEADING)?;
        for i in 0..self.total {
            window.move_to(self.x, self.y + i)?;
            if i < self.offset {
                window.print(text::SCROllBAR_EMPTY)?;
            } else if i < self.offset + self.bar {
                window.print(text::SCROllBAR_SOLID)?;
            } else {
                window.print(text::SCROllBAR_EMPTY)?;
            }
        }
        Ok(())
    }
}

struct ProgramDisplay<'d> {
    debugger: &'d Debugger,
}

impl Draw for ProgramDisplay<'_> {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        let cols = layout::program_cols(window);
        let rows = layout::program_rows(window);
        let space = self.debugger.interpreter.space();
        for y in 0..rows {
            window.move_to(ProgramX(0), ProgramY(y))?;
            let mut skipped = 0;
            for x in 0..cols {
                let pos = Position {
                    x: x as u8,
                    y: y as u8,
                };
                let cell = space.get_cell(pos);
                let state = self.debugger.analysis.cell_states.get_cell(pos);
                let c = char::from_u32(cell.0 as u32).unwrap_or('�');

                if c == ' ' && state.modes() == analyze::Modes::None {
                    skipped += 1;
                    continue;
                }

                if skipped != 0 {
                    window.set_style(styles::PROGRAM_TEXT)?;
                    window.move_right(skipped)?;
                    skipped = 0;
                }

                if c == ' ' {
                    if state.modes() == analyze::Modes::Quoted {
                        window.set_style(styles::VISITED_QUOTED)?;
                        window.print_char(' ')?;
                        continue;
                    }
                    let c = state.directions().blank_char();
                    window.set_style(styles::VISITED_EMPTY)?;
                    window.print_char(c)?;
                    continue;
                }

                let style = styles::for_cell(state.modes(), c);
                window.set_style(style)?;
                window.print_char(c)?;
                window.set_style(styles::PROGRAM_TEXT)?;
            }
        }
        Ok(())
    }
}

impl Directions {
    fn blank_char(self) -> char {
        match self {
            Directions::None => ' ',
            Directions::Horizontal => '─',
            Directions::Vertical => '│',
            Directions::Both => '┼',
        }
    }
}

pub struct ProgramCellReset<'d> {
    pub debugger: &'d Debugger,
    pub pos: Position,
}

impl Draw for ProgramCellReset<'_> {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        // Skip drawing if out of bounds
        let cols = layout::program_cols(window);
        let rows = layout::program_rows(window);
        if self.pos.x as u16 >= cols || self.pos.y as u16 >= rows {
            return Ok(());
        }
        // Move to position
        window.move_to(ProgramX(self.pos.x as u16), ProgramY(self.pos.y as u16))?;
        // Get cell info
        let cell = self.debugger.interpreter.space().get_cell(self.pos);
        let state = self.debugger.analysis.cell_states.get_cell(self.pos);
        let c = char::from_u32(cell.0 as u32).unwrap_or('�');
        // Select character and style
        let (style, c) = match (c, state.modes()) {
            (' ', analyze::Modes::Quoted) => (styles::VISITED_QUOTED, ' '),
            (' ', _) => (styles::VISITED_EMPTY, state.directions().blank_char()),
            _ => (styles::for_cell(state.modes(), c), c),
        };
        window.set_style(style)?;
        window.print_char(c)?;
        window.set_style(styles::BORDER)?;
        Ok(())
    }
}

pub struct ProgramCellCursor<'d> {
    pub debugger: &'d Debugger,
    pub pos: Position,
    pub background_on: bool,
}

impl Draw for ProgramCellCursor<'_> {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        // Skip drawing if out of bounds
        let cols = layout::program_cols(window);
        let rows = layout::program_rows(window);
        if self.pos.x as u16 >= cols || self.pos.y as u16 >= rows {
            return Ok(());
        }
        // Move to position
        window.move_to(ProgramX(self.pos.x as u16), ProgramY(self.pos.y as u16))?;
        // Get cell info
        let cell = self.debugger.interpreter.space().get_cell(self.pos);
        let state = self.debugger.analysis.cell_states.get_cell(self.pos);
        let c = char::from_u32(cell.0 as u32).unwrap_or('�');
        // Select character and style
        let (mut style, c) = match (c, state.modes()) {
            (' ', analyze::Modes::Quoted) => (styles::VISITED_QUOTED, ' '),
            (' ', _) => (styles::VISITED_EMPTY, state.directions().blank_char()),
            _ => (styles::for_cell(state.modes(), c), c),
        };
        if self.background_on {
            style.background_color = styles::CURSOR_ON;
        } else {
            style.background_color = styles::CURSOR_OFF;
        }
        window.set_style(style)?;
        window.print_char(c)?;
        window.set_style(styles::BORDER)?;
        Ok(())
    }
}

pub struct CursorDisplay {
    pub pos: Position,
}

impl Draw for CursorDisplay {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        // X row
        window.move_to(SidebarX(1), TabY(0))?;
        window.set_style(styles::CYAN_HEADING)?;
        window.print(t("X:    "))?;
        window.move_to(SidebarX(4), TabY(0))?;
        window.set_style(styles::PROGRAM_TEXT)?;
        window.print(t(&format!("{}", self.pos.x)))?;
        // Y row
        window.move_to(SidebarX(1), TabY(2))?;
        window.set_style(styles::CYAN_HEADING)?;
        window.print(t("Y: "))?;
        window.move_to(SidebarX(4), TabY(2))?;
        window.set_style(styles::PROGRAM_TEXT)?;
        window.print(t(&format!("{}", self.pos.y)))?;

        Ok(())
    }
}

struct CatLogo;

impl Draw for CatLogo {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        window.move_to(SidebarX(2), TabY(4))?;
        window.set_style(styles::LOGO_OUTLINE)?;
        window.print(t("/\\_/\\"))?;
        window.move_to(SidebarX(1), TabY(5))?;
        window.print(t("(  .  )"))?;
        window.move_to(SidebarX(3), TabY(5))?;
        window.set_style(styles::LOGO_EYES)?;
        window.print(t("o o"))?;
        window.move_to(SidebarX(1), TabY(6))?;
        window.set_style(styles::LOGO_OUTLINE)?;
        window.print(t("befunge"))?;
        Ok(())
    }
}
