use crate::analyze::Directions;

use super::*;
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
        window.move_to(0, 0)?;

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
        self.tabs.draw(window)?;
        Ok(())
    }
}

pub fn stack_room(window: &Window) -> u16 {
    let Dimensions { cols: _, rows } = ProgramView::dimensions(window);
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
        let Dimensions { cols: _, rows } = ProgramView::dimensions(window);
        if self.show_sidebar(window) {
            let even_parity = ProgramView::height_parity_even(window);
            let collapse = self.debugger.stack_height() > stack_room(window);
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

impl<'d> DrawBorder for Sidebar<'d> {
    fn draw_border(&self, window: &mut Window) -> io::Result<()> {
        let Dimensions { cols, rows } = ProgramView::dimensions(window);
        let even_parity = ProgramView::height_parity_even(window);
        let collapse = self.debugger.stack_height() > stack_room(window);
        window.set_style(styles::BORDER)?;
        for i in 0..rows {
            let sidebar = text::sidebar(i, rows, even_parity, collapse);
            window.move_to(cols + 1, i + 1)?;
            window.clear_until_newline()?;
            window.print(sidebar)?;
        }
        Ok(())
    }
}

impl<'d> Draw for Sidebar<'d> {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        StackHeading.draw(window)?;

        let Dimensions { cols, rows } = ProgramView::dimensions(window);
        let even_parity = ProgramView::height_parity_even(window);
        let room = stack_room(window);
        let stack_height = self.debugger.stack_height();
        window.set_style(styles::CYAN_HEADING)?;

        let number_x = cols + 2;
        let symbol_x = cols + 6;

        if stack_height > room {
            let skipped = stack_height - room;
            let skip_x = cols + 3;

            // Draw bottom value
            let bottom = &self.debugger.interpreter.stack()[0];
            window.move_to(number_x, rows)?;
            window.print(t(&format!("{}", bottom.0)))?;
            window.move_to(symbol_x, rows)?;
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
            window.move_to(skip_x, rows - 2)?;
            window.print(t(&format!("{}", skipped)))?;

            // Draw top values
            let mut y = if even_parity { rows - 5 } else { rows - 4 };
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
                y -= 2;
            }
        } else {
            let mut y = if even_parity { rows - 1 } else { rows };
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
                y -= 2;
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
        b'\x27' => " \' ",
        b'\x7F' => "DEL",

        _ => return None,
    };
    Some(code)
}

pub struct ProgramView;

pub struct Dimensions {
    pub cols: u16,
    pub rows: u16,
}

impl ProgramView {
    pub fn dimensions(window: &Window) -> Dimensions {
        Self::dimensions_for_size(window.width(), window.height())
    }

    pub fn dimensions_for_size(width: u16, height: u16) -> Dimensions {
        let cols = width - NON_PROGRAM_WIDTH;
        let rows = height - NON_PROGRAM_HEIGHT;
        Dimensions { cols, rows }
    }

    pub fn height_parity_even(window: &Window) -> bool {
        let height = window.height() - NON_PROGRAM_HEIGHT;
        height % 2 == 0
    }
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
            let even = ProgramView::height_parity_even(window) && !collapse_stack;
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

impl Draw for Tabs {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        TabHeadings {
            tab: self.focused,
            tabbed_both_ways: self.has_tabbed_both_ways(),
        }
        .draw(window)?;

        CatLogo.draw(window)?;

        CursorDisplay { pos: self.position }.draw(window)?;

        // We draw the tab contents last so the cursor is left
        // on the focused input prompt
        match self.focused {
            FocusedTab::Console => self.console.draw(window),
            FocusedTab::Commands => self.commands.draw(window),
            FocusedTab::Timeline => self.timeline.draw(window),
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
        let Dimensions { cols, rows } = ProgramView::dimensions(window);
        let x = cols;
        let y = rows + 4;
        let total = 7;
        let bar = 1;
        let offset = total - self.scroll_height;
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
        let max_width = (window.width() - NON_PROGRAM_WIDTH) as usize;
        for (i, line) in (0..5).zip(self.output.lines()) {
            let x = 1;
            let y = window.height() - 8 + i;
            window.move_to(x, y)?;
            let line = if line.len() > max_width {
                &line[0..max_width]
            } else {
                line
            };
            window.print(tw(line, line.chars().count() as u16))?;
        }

        // Draw command input
        window.move_to(2, window.height() - 2)?;
        window.set_style(styles::CYAN_HEADING)?;
        window.print(t("$ "))?;
        window.set_style(styles::PROGRAM_TEXT)?;
        let buf = self.input_contents.to_string();
        window.print(t(&buf))?;
        window.move_to(4 + self.input_cursor, window.height() - 2)?;
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
        let x = 1;
        let y = window.height() - 3;
        let total = window.width() - NON_PROGRAM_WIDTH;
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
        window.move_to(window.width() - 7, 1)?;
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
        let Dimensions {
            cols: _,
            rows: program_rows,
        } = ProgramView::dimensions(window);
        let x = 2;
        let y = program_rows + 2;
        window.move_to(x, y)?;
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
    x: u16,
    y: u16,
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
    x: u16,
    /// y position in overall window space
    y: u16,
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

impl<'a> Draw for ProgramDisplay<'a> {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        let Dimensions { cols, rows } = ProgramView::dimensions(window);
        let space = self.debugger.interpreter.space();
        for y in 0..rows {
            window.move_to(1, y + 1)?;
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

impl<'d> Draw for ProgramCellReset<'d> {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        // Skip drawing if out of bounds
        let Dimensions { cols, rows } = ProgramView::dimensions(window);
        if self.pos.x as u16 >= cols || self.pos.y as u16 >= rows {
            return Ok(());
        }
        // Move to position
        window.move_to((self.pos.x + 1) as u16, (self.pos.y + 1) as u16)?;
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

impl<'d> Draw for ProgramCellCursor<'d> {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        // Skip drawing if out of bounds
        let Dimensions { cols, rows } = ProgramView::dimensions(window);
        if self.pos.x as u16 >= cols || self.pos.y as u16 >= rows {
            return Ok(());
        }
        // Move to position
        window.move_to((self.pos.x + 1) as u16, (self.pos.y + 1) as u16)?;
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
        window.move_to(window.width() - 8, window.height() - 8)?;
        window.set_style(styles::CYAN_HEADING)?;
        window.print(t("X:    "))?;
        window.move_to(window.width() - 5, window.height() - 8)?;
        window.set_style(styles::PROGRAM_TEXT)?;
        window.print(t(&format!("{}", self.pos.x)))?;
        // Y row
        window.move_to(window.width() - 8, window.height() - 6)?;
        window.set_style(styles::CYAN_HEADING)?;
        window.print(t("Y: "))?;
        window.move_to(window.width() - 5, window.height() - 6)?;
        window.set_style(styles::PROGRAM_TEXT)?;
        window.print(t(&format!("{}", self.pos.y)))?;

        Ok(())
    }
}

struct CatLogo;

impl Draw for CatLogo {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        window.move_to(window.width() - 7, window.height() - 4)?;
        window.set_style(styles::LOGO_OUTLINE)?;
        window.print(t("/\\_/\\"))?;
        window.move_to(window.width() - 8, window.height() - 3)?;
        window.print(t("(  .  )"))?;
        window.move_to(window.width() - 6, window.height() - 3)?;
        window.set_style(styles::LOGO_EYES)?;
        window.print(t("o o"))?;
        window.move_to(window.width() - 8, window.height() - 2)?;
        window.set_style(styles::LOGO_OUTLINE)?;
        window.print(t("befunge"))?;
        Ok(())
    }
}
