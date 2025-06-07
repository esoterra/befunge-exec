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
            interpreter: &self.debugger.interpreter,
            analysis: &self.debugger.analysis,
        }
        .draw(window)?;
        self.tabs.draw(window)?;
        Ok(())
    }
}

impl Tui {
    fn draw_border_main(&self, window: &mut Window) -> io::Result<()> {
        let Dimensions { cols: _, rows } = ProgramView::dimensions(window);
        if self.show_sidebar(window) {
            let even_parity = ProgramView::height_parity_even(window);
            let collapse = false; // TODO
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

struct ProgramDisplay<'a> {
    analysis: &'a PathAnalysis,
    interpreter: &'a Interpreter<VecIO>,
}

impl<'a> Draw for ProgramDisplay<'a> {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        let Dimensions { cols, rows } = ProgramView::dimensions(window);
        let space = self.interpreter.space();
        for y in 0..rows {
            window.move_to(1, y + 1)?;
            let mut skipped = 0;
            for x in 0..cols {
                let pos = Position {
                    x: x as u8,
                    y: y as u8,
                };
                let cell = space.get_cell(pos);
                let state = self.analysis.cell_states.get_cell(pos);
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
                    let c = match state.directions() {
                        analyze::Directions::None => unreachable!(),
                        analyze::Directions::Horizontal => '─',
                        analyze::Directions::Vertical => '│',
                        analyze::Directions::Both => '┼',
                    };
                    window.set_style(styles::VISITED_EMPTY)?;
                    window.print_char(c)?;
                    continue;
                }

                let style = match state.modes() {
                    analyze::Modes::None => styles::PROGRAM_TEXT,
                    analyze::Modes::Quoted => styles::VISITED_QUOTED,
                    analyze::Modes::Normal => match c {
                        c if c.is_ascii_digit() => styles::VISITED_NUMBER,
                        '^' | 'v' | '<' | '>' | '?' | '#' | '_' | '|' => styles::VISITED_DIR,
                        '.' | ',' | '~' | '&' => styles::VISITED_IO,
                        '+' | '-' | '*' | '/' | '%' | ':' | '$' | '\\' | '`' | '!' => {
                            styles::VISITED_STACK
                        }
                        'p' | 'g' => styles::VISITED_PG,
                        '@' => styles::VISITED_RED,
                        _ => styles::VISITED_NORMAL,
                    },
                    analyze::Modes::Both => styles::VISITED_NORMAL,
                };
                window.set_style(style)?;
                window.print_char(c)?;
                window.set_style(styles::PROGRAM_TEXT)?;
            }
        }
        Ok(())
    }
}

struct CursorDisplay {
    pos: Position,
}

impl Draw for CursorDisplay {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        // X row
        window.move_to(window.width() - 8, window.height() - 8)?;
        window.set_style(styles::CYAN_HEADING)?;
        window.print(t("X: "))?;
        window.set_style(styles::PROGRAM_TEXT)?;
        window.print(t(&format!("{}", self.pos.x)))?;
        // Y row
        window.move_to(window.width() - 8, window.height() - 6)?;
        window.set_style(styles::CYAN_HEADING)?;
        window.print(t("Y: "))?;
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
