use std::io;
use super::*;

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

        if self.show_sidebar() {
            window.line(tw("╔", 1), text::PIPES, tw("╦═══════╗", 9))?;
        } else if self.show_outer_border() {
            window.line(tw("╔", 1), text::PIPES, tw("╗", 1))?;
        }

        self.draw_border_main(window)?;
        self.draw_border_tabs(window)?;
        match self.tab {
            FocusedTab::Console => self.console_view.draw_border(window)?,
            FocusedTab::Commands => self.commands_view.draw_border(window)?,
            FocusedTab::Timeline => self.timeline_view.draw_border(window)?,
        }

        if self.show_sidebar() {
            window.line(tw("╚", 1), text::PIPES, tw("╩═══════╝", 9))?;
        } else if self.show_outer_border() {
            window.line(tw("╚", 1), text::PIPES, tw("╝", 1))?;
        }
        Ok(())
    }
}

impl Draw for Tui {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        self.draw_headings(window)?;
        draw_program(window, &self.interpreter)?;
        match self.tab {
            FocusedTab::Console => self.console_view.draw(window)?,
            FocusedTab::Commands => self.commands_view.draw(window)?,
            FocusedTab::Timeline => self.timeline_view.draw(window)?,
        }
        Ok(())
    }
}

impl Tui {
    fn draw_border_main(&self, window: &mut Window) -> io::Result<()> {
        let (_, height) = ProgramView::dimensions(window);
        if self.show_sidebar() {
            let even_parity = ProgramView::height_parity_even(window);
            let collapse = self.collapse_stack();
            for i in 0..height {
                let sidebar = text::sidebar(i, height, even_parity, collapse);
                window.line(tw("║", 1), text::SPACES, tw(sidebar, 9))?;
            }
        } else {
            for _ in 0..height {
                window.line(tw("║", 1), text::SPACES, tw("║", 1))?;
            }
        }

        Ok(())
    }

    fn draw_border_tabs(&self, window: &mut Window) -> io::Result<()> {
        let tight = window.width() == 60;
        let heading_lines = {
            if tight {
                match self.tab {
                    FocusedTab::Console => text::CONSOLE_TAB_FRAME_NORMAL_TIGHT,
                    FocusedTab::Commands => text::COMMANDS_TAB_FRAME_NORMAL_TIGHT,
                    FocusedTab::Timeline => text::TIMELINE_FRAME_NORMAL_TIGHT,
                }
            } else {
                match self.tab {
                    FocusedTab::Console => text::CONSOLE_TAB_FRAME_NORMAL,
                    FocusedTab::Commands => text::COMMANDS_TAB_FRAME_NORMAL,
                    FocusedTab::Timeline => text::TIMELINE_FRAME_NORMAL,
                }
            }
        };

        let text::TabSidebar { top, mid, bot } = {
            let tab = self.tab == FocusedTab::Timeline;
            let even = ProgramView::height_parity_even(window) && !self.collapse_stack();
            text::tabs_sidebar(tight, tab, even)
        };

        window.line(heading_lines[0], text::LINES, top)?;
        window.line(heading_lines[1], text::SPACES, mid)?;
        window.line(heading_lines[2], text::PIPES, bot)?;
        Ok(())
    }

    fn draw_headings(&self, window: &mut Window) -> io::Result<()> {
        StackHeading.draw(window)?;
        TabHeadings {
            tab: self.tab,
            width_bp: self.width_bp,
            tabbed_both_ways: self.has_tabbed && self.has_back_tabbed,
        }
        .draw(window)?;
        CatLogo.draw(window)?;
        Ok(())
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
        let x = window.width() - NON_PROGRAM_WIDTH;
        let y = window.height() - NON_PROGRAM_HEIGHT + 4;
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
        // Draw scrollbar
        // let x = window.width() - NON_PROGRAM_WIDTH;
        // let y = window.height() - NON_PROGRAM_HEIGHT + 4;
        // let total = 5;
        // let bar = 1;
        // let offset = 0;
        // VerticalScrollbar {
        //     x,
        //     y,
        //     total,
        //     bar,
        //     offset,
        // }
        // .draw(window)?;

        // Draw command output
        window.set_style(styles::PROGRAM_TEXT)?;
        let max_width = (window.width() - NON_PROGRAM_WIDTH) as usize;
        for (i, line) in (0..5).zip(self.output.lines()) {
            let x = 1;
            let y = window.height() - 8 + i;
            window.move_to(x, y)?;
            let line = if line.len() > max_width { &line[0..max_width] } else { line };
            window.print(t(line))?;
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
        let y = window.height();
        let total = window.width() - NON_PROGRAM_WIDTH;
        let bar = total / 4;
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

impl Draw for StackHeading {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        window.move_to(window.width() - 7, 1)?;
        window.set_style(styles::CYAN_HEADING)?;
        window.print(t("Stack"))?;
        Ok(())
    }
}

impl Draw for TabHeadings {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        let x = 2;
        let y = window.height() - NON_PROGRAM_HEIGHT + 2;
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

        if self.width_bp == WidthBreakPoint::Wide && !self.tabbed_both_ways {
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
    x: u16,
    y: u16,
    total: u16,
    bar: u16,
    offset: u16,
}

impl VerticalScrollbar {
    fn draw(&self, window: &mut Window) -> io::Result<()> {
        window.set_style(styles::CYAN_HEADING)?;
        window.move_to(self.x, self.y)?;
        for i in 0..self.total {
            window.move_to(
                window.width() - NON_PROGRAM_WIDTH,
                window.height() - NON_PROGRAM_HEIGHT + 4 + i,
            )?;
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

fn draw_program(
    window: &mut Window,
    interpreter: &Interpreter<VecIO>,
) -> io::Result<()> {
    let (width, height) = ProgramView::dimensions(window);
    window.set_style(styles::PROGRAM_TEXT)?;
    for i in 0..height {
        window.move_to(1, i + 1)?;
        let mut skipped = 0;
        for j in 0..width {
            let pos = Position {
                x: j as u8,
                y: i as u8,
            };
            let cell = interpreter.get_cell(pos);
            if let Some(c) = char::from_u32(cell.0 as u32) {
                if c == ' ' {
                    skipped += 1;
                } else {
                    window.move_right(skipped)?;
                    skipped = 0;
                    window.print_char(c)?;
                }
            } else {
                window.move_right(skipped)?;
                skipped = 0;
                window.print(tw("�", 1))?;
            }
        }
    }
    Ok(())
}

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