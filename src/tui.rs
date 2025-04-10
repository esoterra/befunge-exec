use crossterm::cursor::{MoveRight, MoveTo, MoveToNextLine};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseEventKind};
use crossterm::style::{ContentStyle, Print, SetStyle};
use crossterm::terminal::{
    BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate, EnterAlternateScreen,
    LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, size,
};
use crossterm::{QueueableCommand, execute};
use std::io::{self, Stdout, Write, stdout};
use text::{Text, t, tw};

mod styles;
mod text;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FocusedTab {
    Console,
    Commands,
    Timeline,
}

pub fn print_tui(tab: FocusedTab) -> io::Result<()> {
    let (width, height) = size()?;
    let mut tui = Tui::new(width, height, tab);
    tui.init()?;

    tui.draw_frame()?;

    loop {

        let event = crossterm::event::read()?;
        match event {
            Event::Resize(width, height) => {
                tui.set_size(width, height);
                tui.draw_frame()?;
            },
            Event::Key(event) => {
                match event {
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    } => break,
                    KeyEvent {
                        code: KeyCode::BackTab,
                        ..
                    } => {
                        tui.focus_previous();
                    }
                    KeyEvent {
                        code: KeyCode::Tab, ..
                    } => {
                        tui.focus_next();
                    }
                    _ => {}
                }
                tui.draw_frame()?;
            },
            Event::Mouse(event) => {
                match event.kind {
                    MouseEventKind::Moved => {},
                    MouseEventKind::Drag(_) => {},
                    _ => eprintln!("Mouse event: {:?}", event)
                }
            }
            _ => {}
        }
    }

    tui.close()?;
    Ok(())
}

#[allow(dead_code)]
struct Tui {
    width: u16,
    height: u16,
    program_width: u16,
    program_height: u16,
    width_bp: WidthBreakPoint,
    height_bp: HeightBreakPoint,

    tab: FocusedTab,
    has_tabbed: bool,
    has_back_tabbed: bool,
    stdout: Stdout,

    console: ConsoleData,
}

struct ConsoleData {
    scroll_height: u16,
}

impl Default for ConsoleData {
    fn default() -> Self {
        Self { scroll_height: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WidthBreakPoint {
    /// Show title, tabs, hint, and sidebar
    /// ║ Befunge Debugger ║ Console ║ Commands │ Timeline │  switch using [shift] tab  ║ <- 81
    /// Range: w > 80
    Wide,
    /// Show title, tabs, and sidebar
    /// ║ Befunge Debugger ║ Console ║ Commands │ Timeline ║ <- 52
    /// Range: 80 >= w > 51
    Normal,
    /// Show tabs
    /// ║ Console ║ Commands │ Timeline ║ <- 33
    /// Range: 51 >= w > 32
    Narrow,
    // Don't show any tab section or headings
    /// ║                   ║ <- 21
    /// Range: 32 > w
    Tiny,
}

impl WidthBreakPoint {
    fn for_width(width: u16) -> Self {
        if width > 80 {
            WidthBreakPoint::Wide
        } else if width > 51 {
            WidthBreakPoint::Normal
        } else if width > 32 {
            WidthBreakPoint::Narrow
        } else {
            WidthBreakPoint::Tiny
        }
    }
}

fn cmp_width_bp(a: WidthBreakPoint, b: WidthBreakPoint) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a, b) {
        (WidthBreakPoint::Wide, WidthBreakPoint::Wide)
        | (WidthBreakPoint::Normal, WidthBreakPoint::Normal)
        | (WidthBreakPoint::Narrow, WidthBreakPoint::Narrow)
        | (WidthBreakPoint::Tiny, WidthBreakPoint::Tiny) => Ordering::Equal,

        (WidthBreakPoint::Wide, WidthBreakPoint::Normal)
        | (WidthBreakPoint::Wide, WidthBreakPoint::Narrow)
        | (WidthBreakPoint::Wide, WidthBreakPoint::Tiny)
        | (WidthBreakPoint::Normal, WidthBreakPoint::Narrow)
        | (WidthBreakPoint::Normal, WidthBreakPoint::Tiny)
        | (WidthBreakPoint::Narrow, WidthBreakPoint::Tiny) => Ordering::Greater,

        (WidthBreakPoint::Normal, WidthBreakPoint::Wide)
        | (WidthBreakPoint::Narrow, WidthBreakPoint::Wide)
        | (WidthBreakPoint::Narrow, WidthBreakPoint::Normal)
        | (WidthBreakPoint::Tiny, WidthBreakPoint::Wide)
        | (WidthBreakPoint::Tiny, WidthBreakPoint::Normal)
        | (WidthBreakPoint::Tiny, WidthBreakPoint::Narrow) => Ordering::Less,
    }
}

impl PartialOrd for WidthBreakPoint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(cmp_width_bp(*self, *other))
    }
}

impl Ord for WidthBreakPoint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        cmp_width_bp(*self, *other)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum HeightBreakPoint {
    /// Show everything
    Tall,
    /// Don't show tab section
    Tiny,
}

impl HeightBreakPoint {
    fn for_height(height: u16) -> Self {
        if height > 32 {
            HeightBreakPoint::Tall
        } else {
            HeightBreakPoint::Tiny
        }
    }
}

impl Tui {
    fn new(width: u16, height: u16, tab: FocusedTab) -> Self {
        let program_width = width - 10;
        let program_height = height - 12;
        let width_bp = WidthBreakPoint::for_width(width);
        let height_bp = HeightBreakPoint::for_height(height);
        Self {
            width,
            height,
            program_width,
            program_height,
            width_bp,
            height_bp,
            tab,
            has_tabbed: false,
            has_back_tabbed: false,
            stdout: stdout(),
            console: Default::default(),
        }
    }

    fn set_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.program_height = width - 10;
        self.program_height = height - 12;
        self.width_bp = WidthBreakPoint::for_width(width);
        self.height_bp = HeightBreakPoint::for_height(height);
    }

    fn show_outer_border(&self) -> bool {
        self.width_bp >= WidthBreakPoint::Narrow
    }

    fn show_sidebar(&self) -> bool {
        self.width_bp >= WidthBreakPoint::Normal
    }

    fn collapse_stack(&self) -> bool {
        // TODO: make true whenever stack depth > available stack UI space
        false
    }

    fn focus_next(&mut self) {
        self.has_tabbed = true;
        self.tab = match self.tab {
            FocusedTab::Console => FocusedTab::Commands,
            FocusedTab::Commands => FocusedTab::Timeline,
            FocusedTab::Timeline => FocusedTab::Console,
        };
    }

    fn focus_previous(&mut self) {
        self.has_back_tabbed = true;
        self.tab = match self.tab {
            FocusedTab::Console => FocusedTab::Timeline,
            FocusedTab::Commands => FocusedTab::Console,
            FocusedTab::Timeline => FocusedTab::Commands,
        };
    }

    fn init(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(self.stdout, EnterAlternateScreen)?;
        execute!(self.stdout, Clear(ClearType::All))?;
        execute!(self.stdout, EnableMouseCapture)?;
        Ok(())
    }

    fn close(&mut self) -> io::Result<()> {
        execute!(self.stdout, Clear(ClearType::All))?;
        execute!(self.stdout, LeaveAlternateScreen)?;
        execute!(self.stdout, DisableMouseCapture)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        execute!(self.stdout, Clear(ClearType::All))
    }

    fn draw_frame(&mut self) -> io::Result<()> {
        self.start_frame()?;
        self.clear()?;
        self.draw_border()?;
        self.draw_headings()?;
        self.draw_scrollbar()?;
        self.end_frame()?;
        Ok(())
    }

    fn start_frame(&mut self) -> io::Result<()> {
        execute!(self.stdout, BeginSynchronizedUpdate)
    }

    fn end_frame(&mut self) -> io::Result<()> {
        self.stdout.queue(EndSynchronizedUpdate)?;
        self.stdout.flush()
    }

    fn draw_border(&mut self) -> io::Result<()> {
        self.stdout.queue(SetStyle(styles::BORDER))?;
        self.stdout.queue(MoveTo(0, 0))?;

        if self.show_sidebar() {
            self.line(tw("╔", 1), text::PIPES, tw("╦═══════╗", 9))?;
        } else if self.show_outer_border() {
            self.line(tw("╔", 1), text::PIPES, tw("╗", 1))?;
        }

        self.draw_border_main()?;
        self.draw_border_tabs()?;
        match self.tab {
            FocusedTab::Console => self.draw_border_console_tab()?,
            FocusedTab::Commands => self.draw_border_commands_tab()?,
            FocusedTab::Timeline => self.draw_border_timeline_tab()?,
        }

        if self.show_sidebar() {
            self.line(tw("╚", 1), text::PIPES, tw("╩═══════╝", 9))?;
        } else if self.show_outer_border() {
            self.line(tw("╚", 1), text::PIPES, tw("╝", 1))?;
        }

        self.stdout.flush()
    }

    fn draw_border_main(&mut self) -> io::Result<()> {
        if self.show_sidebar() {
            let even_parity = self.program_height_parity_even();
            let collapse = self.collapse_stack();
            for i in 0..self.program_height {
                let sidebar = text::sidebar(i, self.program_height, even_parity, collapse);
                self.line(tw("║", 1), text::SPACES, tw(sidebar, 9))?;
            }
        } else {
            for _ in 0..self.program_height {
                self.line(tw("║", 1), text::SPACES, tw("║", 1))?;
            }
        }

        Ok(())
    }

    fn program_height_parity_even(&self) -> bool {
        self.program_height % 2 == 0
    }

    fn draw_border_tabs(&mut self) -> io::Result<()> {
        let tight = self.width == 60;
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
            let even = self.program_height_parity_even() && !self.collapse_stack();
            text::tabs_sidebar(tight, tab, even)
        };

        self.line(heading_lines[0], text::LINES, top)?;
        self.line(heading_lines[1], text::SPACES, mid)?;
        self.line(heading_lines[2], text::PIPES, bot)?;
        Ok(())
    }

    fn draw_border_console_tab(&mut self) -> io::Result<()> {
        self.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("╠═══════╣", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("╠═══════╣", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        Ok(())
    }

    fn draw_border_commands_tab(&mut self) -> io::Result<()> {
        self.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("╠═══════╣", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("╠═══════╣", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        self.line(tw("╠", 1), text::PIPES, tw("╣       ║", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        Ok(())
    }

    fn draw_border_timeline_tab(&mut self) -> io::Result<()> {
        self.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("╠═══════╣", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("╠═══════╣", 9))?;
        self.line(tw("╠", 1), text::PIPES, tw("╣       ║", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        self.line(tw("║", 1), text::SPACES, tw("║       ║", 9))?;
        Ok(())
    }

    fn draw_headings(&mut self) -> io::Result<()> {
        self.draw_stack_heading()?;
        self.draw_tab_headings()?;
        self.draw_logo()?;
        Ok(())
    }

    fn draw_stack_heading(&mut self) -> io::Result<()> {
        self.stdout.queue(MoveTo(self.width - 7, 1))?;
        self.stdout.queue(SetStyle(styles::CYAN_HEADING))?;
        self.stdout.queue(Print("Stack"))?;
        Ok(())
    }

    fn draw_tab_headings(&mut self) -> io::Result<()> {
        self.move_to(2, self.program_height + 2)?;
        self.set_style(styles::CYAN_HEADING)?;
        self.print(text::BEFUNGE_DEBUGGER)?;

        if self.tab == FocusedTab::Console {
            self.set_style(styles::GREEN_HEADING)?;
        } else {
            self.set_style(styles::GREEN_HEADING_UNFOCUSED)?;
        }

        self.move_right(3)?;
        self.print(text::CONSOLE)?;

        if self.tab == FocusedTab::Commands {
            self.set_style(styles::GREEN_HEADING)?;
        } else {
            self.set_style(styles::GREEN_HEADING_UNFOCUSED)?;
        }

        self.move_right(3)?;
        self.print(text::COMMANDS)?;

        if self.tab == FocusedTab::Timeline {
            self.set_style(styles::GREEN_HEADING)?;
        } else {
            self.set_style(styles::GREEN_HEADING_UNFOCUSED)?;
        }

        self.move_right(3)?;
        self.print(text::TIMELINE)?;

        let tabbed_both_ways = self.has_back_tabbed && self.has_tabbed;
        if self.width_bp == WidthBreakPoint::Wide && !tabbed_both_ways {
            self.move_right(4)?;
            self.set_style(styles::GRAY_HEADING)?;
            self.print(text::TAB_SWITCH_HINT)?;
        }

        Ok(())
    }

    fn draw_scrollbar(&mut self) -> io::Result<()> {
        

        self.stdout.queue(SetStyle(styles::CYAN_HEADING))?;
        if self.tab == FocusedTab::Timeline {
            let width = self.width - 10;
            self.move_to(1, self.height - 3)?;
            self.print(text::SOLID.text(10))?;
            self.print(text::EMPTY.text(width - 10))?;
        } else {
            let height = if self.tab == FocusedTab::Console { 7 } else { 5 };
            for i in 0..height {
                self.move_to(self.width - 10, self.program_height + 4 + i)?;
                if self.console.scroll_height == i {
                    self.print(text::SCROllBAR_SOLID)?;
                } else {
                    self.print(text::SCROllBAR_EMPTY)?;
                }
            }
        }
        Ok(())
    }

    fn draw_logo(&mut self) -> io::Result<()> {
        self.move_to(self.width - 7, self.height - 4)?;
        self.set_style(styles::LOGO_OUTLINE)?;
        self.print(t("/\\_/\\"))?;
        self.move_to(self.width - 8, self.height - 3)?;
        self.print(t("(  .  )"))?;
        self.move_to(self.width - 6, self.height - 3)?;
        self.set_style(styles::LOGO_EYES)?;
        self.print(t("o o"))?;
        self.move_to(self.width - 8, self.height - 2)?;
        self.set_style(styles::LOGO_OUTLINE)?;
        self.print(t("befunge"))?;
        Ok(())
    }

    // Whole Line Drawing

    fn line<A, B>(&mut self, pre: Text<A>, mid: impl text::PrintN, end: Text<B>) -> io::Result<()>
    where
        A: AsRef<str>,
        B: AsRef<str>,
    {
        print!("{}", pre.as_ref());
        let used_space = pre.width() + end.width();
        if used_space > self.width {
            eprintln!(
                "Tried to draw '{}' '{}'* '{}' which used {} columns but {} were available",
                pre.as_ref(),
                mid.one(),
                end.as_ref(),
                used_space,
                self.width
            );
            return Ok(());
        }
        let n = self.width - pre.width() - end.width();
        mid.print_n(&mut self.stdout, n)?;
        print!("{}", end.as_ref());
        self.stdout.queue(MoveToNextLine(1))?;
        Ok(())
    }

    // Terminal Operation Wrappers

    fn move_to(&mut self, x: u16, y: u16) -> io::Result<()> {
        self.stdout.queue(MoveTo(x, y))?;
        Ok(())
    }

    fn move_right(&mut self, n: u16) -> io::Result<()> {
        self.stdout.queue(MoveRight(n))?;
        Ok(())
    }

    fn set_style(&mut self, style: ContentStyle) -> io::Result<()> {
        self.stdout.queue(SetStyle(style))?;
        Ok(())
    }

    fn print<A: AsRef<str>>(&mut self, t: Text<A>) -> io::Result<()> {
        self.stdout.queue(Print(t.as_ref()))?;
        Ok(())
    }
}
