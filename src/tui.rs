use std::io;

mod commands;
mod console;
mod draw;
pub mod styles;
pub mod text;
mod timeline;
mod window;

pub use commands::CommandsView;
pub use console::ConsoleView;
pub use draw::{Draw, DrawBorder};
pub use text::{t, tw};
pub use timeline::TimelineView;
pub use window::Window;

use crate::core::Position;
use crate::interpreter::Interpreter;
use crate::io::VecIO;

use crossterm::event::{
    Event, KeyCode, KeyEvent, MouseEventKind,
};

pub const NON_PROGRAM_WIDTH: u16 = 10;
pub const NON_PROGRAM_HEIGHT: u16 = 12;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FocusedTab {
    Console,
    Commands,
    Timeline,
}

pub fn run_tui(name: String, program: Vec<u8>, tab: FocusedTab) -> io::Result<()> {
    let title = format!("befunge-exec: {}", name);
    let mut window = Window::new()?;
    let mut tui = Tui::new(title, program, tab, &window)?;
    tui.init(&mut window)?;

    tui.draw_frame(&mut window)?;

    loop {
        let event = crossterm::event::read()?;
        match event {
            Event::Resize(width, height) => {
                window.set_size(width, height);
                tui.set_size(width, height);
                tui.draw_frame(&mut window)?;
            }
            Event::Key(event) => {
                if event.code == KeyCode::Esc {
                    break;
                }
                tui.on_key_event(event);
                tui.draw_frame(&mut window)?;
            }
            Event::Mouse(event) => match event.kind {
                MouseEventKind::Moved => {}
                MouseEventKind::Drag(_) => {}
                _ => eprintln!("Mouse event: {:?}", event),
            },
            _ => {}
        }
    }

    tui.close(&mut window)?;
    Ok(())
}

#[allow(dead_code)]
struct Tui {
    title: String,
    program: Vec<u8>,
    interpreter: Interpreter<VecIO>,
    width_bp: WidthBreakPoint,
    height_bp: HeightBreakPoint,

    tab: FocusedTab,
    has_tabbed: bool,
    has_back_tabbed: bool,

    program_view: ProgramView,
    console_view: ConsoleView,
    commands_view: CommandsView,
    timeline_view: TimelineView,
}

#[derive(Default)]
struct ProgramView {
}

impl ProgramView {
    fn dimensions(window: &Window) -> (u16, u16) {
        let width = window.width() - NON_PROGRAM_WIDTH;
        let height = window.height() - NON_PROGRAM_HEIGHT;
        (width, height)
    }

    fn height_parity_even(window: &Window) -> bool {
        let height = window.height() - NON_PROGRAM_HEIGHT;
        height % 2 == 0
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
    fn new(title: String, program: Vec<u8>, tab: FocusedTab, window: &Window) -> io::Result<Self> {
        let interpreter = Interpreter::new(&program, VecIO::default());
        let program_view = ProgramView::default();
        let width_bp = WidthBreakPoint::for_width(window.width());
        let height_bp = HeightBreakPoint::for_height(window.height());
        Ok(Self {
            title,
            program,
            interpreter,
            width_bp,
            height_bp,
            tab,
            has_tabbed: false,
            has_back_tabbed: false,
            program_view,
            console_view: Default::default(),
            commands_view: Default::default(),
            timeline_view: TimelineView,
        })
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

    fn init(&self, window: &mut Window) -> io::Result<()> {
        window.init()?;
        window.set_title(&self.title)?;
        Ok(())
    }

    fn close(&self, window: &mut Window) -> io::Result<()> {
        window.close()
    }

    fn set_size(&mut self, width: u16, height: u16) {
        self.width_bp = WidthBreakPoint::for_width(width);
        self.height_bp = HeightBreakPoint::for_height(height);
    }

    fn on_key_event(&mut self, event: KeyEvent) {
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
            _ => match self.tab {
                FocusedTab::Console => {}
                FocusedTab::Commands => self.commands_view.on_key_event(event),
                FocusedTab::Timeline => {}
            },
        }
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

    fn draw_frame(&mut self, window: &mut Window) -> io::Result<()> {
        window.start_frame()?;
        window.clear()?;
        self.draw_border(window)?;
        self.draw(window)?;
        window.end_frame()?;
        Ok(())
    }
}

struct ProgramDisplay<'a> {
    interpreter: &'a Interpreter<VecIO>
}

struct CursorDisplay {
    pos: Position
}

struct TabHeadings {
    tab: FocusedTab,
    width_bp: WidthBreakPoint,
    tabbed_both_ways: bool,
}

struct StackHeading;
struct CatLogo;


