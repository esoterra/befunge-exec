use std::io;
use std::time::{Duration, Instant};

mod draw;
pub mod styles;
pub mod tabs;
pub mod text;
mod window;

pub use draw::{Draw, DrawBorder};
pub use tabs::{CommandsView, ConsoleView, FocusedTab, Tabs, TimelineView};
pub use text::{t, tw};
pub use window::Window;

use crate::analyze::{self, PathAnalysis};
use crate::interpreter::Interpreter;
use crate::io::VecIO;
use crate::{core::Position, space::Space};

use crossterm::event::{Event, KeyCode, KeyEvent, MouseEventKind};

const TICKS_PER_SECOND: u64 = 40;
const MILLIS_PER_TICK: u64 = 1000 / TICKS_PER_SECOND;

pub fn run_tui(name: String, program: Vec<u8>) -> io::Result<()> {
    let title = format!("befunge-exec: {}", name);
    let mut window = Window::new()?;
    let mut tui = Tui::new(title, program)?;

    tui.init(&mut window)?;

    let mut resized = false;
    let mut next_tick = Instant::now() + Duration::from_millis(MILLIS_PER_TICK);
    'tick: loop {
        'poll: loop {
            // Check if it's time for the next tick
            let now = Instant::now();
            if now >= next_tick {
                break 'poll;
            }

            // Try to read an event before the next tick
            let has_event = crossterm::event::poll(next_tick - now)?;
            if !has_event {
                continue;
            }

            // Process the event
            let event = crossterm::event::read()?;
            match event {
                Event::Resize(width, height) => {
                    eprintln!("Resized to ({}, {})", width, height);
                    window.set_size(width, height);
                    resized = true;
                }
                Event::Key(event) => {
                    if event.code == KeyCode::Esc {
                        break 'tick;
                    }
                    tui.on_key_event(event);
                }
                Event::Mouse(event) => match event.kind {
                    MouseEventKind::Moved => {}
                    MouseEventKind::Drag(_) => {}
                    _ => eprintln!("Mouse event: {:?}", event),
                },
                _ => {}
            }
        }

        tui.tick(&mut window, resized)?;
        resized = false;

        next_tick += Duration::from_millis(MILLIS_PER_TICK);
    }

    tui.close(&mut window)?;
    Ok(())
}

pub const WIDE_WIDTH: u16 = 80;

#[allow(dead_code)]
struct Tui {
    title: String,
    program: Vec<u8>,
    analysis: PathAnalysis,
    interpreter: Interpreter<VecIO>,

    program_view: ProgramView,
    tabs: Tabs,
}

const NON_PROGRAM_WIDTH: u16 = 10;
const NON_PROGRAM_HEIGHT: u16 = 12;

#[derive(Default)]
struct ProgramView {}

struct Dimensions {
    cols: u16,
    rows: u16,
}

impl ProgramView {
    fn dimensions(window: &Window) -> Dimensions {
        Self::dimensions_for_size(window.width(), window.height())
    }

    fn dimensions_for_size(width: u16, height: u16) -> Dimensions {
        let cols = width - NON_PROGRAM_WIDTH;
        let rows = height - NON_PROGRAM_HEIGHT;
        Dimensions { cols, rows }
    }

    fn height_parity_even(window: &Window) -> bool {
        let height = window.height() - NON_PROGRAM_HEIGHT;
        height % 2 == 0
    }
}

struct CursorDisplay {
    pos: Position,
}

struct TabHeadings {
    tab: FocusedTab,
    tabbed_both_ways: bool,
}

struct StackHeading;
struct CatLogo;

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
impl Tui {
    fn new(title: String, program: Vec<u8>) -> io::Result<Self> {
        let space = Space::new(&program);
        let analysis = analyze::analyze_path(&space);
        let interpreter = Interpreter::new(space, VecIO::default());
        Ok(Self {
            title,
            program,
            analysis,
            interpreter,
            program_view: Default::default(),
            tabs: Default::default(),
        })
    }

    fn show_outer_border(&self, _window: &Window) -> bool {
        true
    }

    fn show_sidebar(&self, window: &Window) -> bool {
        window.width() >= 52
    }

    fn init(&self, window: &mut Window) -> io::Result<()> {
        window.init()?;
        window.set_title(&self.title)?;
        // Draw first frame
        window.start_frame()?;
        window.clear()?;
        self.draw_border(window)?;
        self.draw(window)?;
        window.end_frame()
    }

    fn close(&self, window: &mut Window) -> io::Result<()> {
        window.close()
    }

    fn tick(&mut self, window: &mut Window, resized: bool) -> io::Result<()> {
        window.start_frame()?;
        if resized {
            // redraw everything on resize
            eprintln!("Draw everything");
            window.clear()?;
            self.draw_border(window)?;
            self.draw(window)?;
        } else {
            if self.tabs.dirty {
                eprintln!("Draw tabs");
                self.tabs.dirty = false;
                let Dimensions { cols: _, rows } = ProgramView::dimensions(window);
                window.move_to(0, rows + 1)?;
                window.clear_down()?;
                window.set_style(styles::BORDER)?;
                self.tabs.draw_border(window)?;
                self.draw_border_last(window)?;
                self.tabs.draw(window)?;
            }
        }
        window.end_frame()
    }

    fn on_key_event(&mut self, event: KeyEvent) {
        self.tabs.on_key_event(event);
    }
}
