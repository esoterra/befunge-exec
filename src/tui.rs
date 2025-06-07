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
use crate::debugger::Debugger;
use crate::interpreter::Interpreter;
use crate::io::VecIO;
use crate::tui::draw::{Dimensions, ProgramView};
use crate::tui::tabs::CommandEvent;
use crate::{core::Position};

use crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent, MouseEventKind};

const TICKS_PER_SECOND: u64 = 40;
const MILLIS_PER_TICK: u64 = 1000 / TICKS_PER_SECOND;

pub fn run_tui(name: String, program: Vec<u8>) -> io::Result<()> {
    let title = format!("befunge-exec: {}", name);
    let mut window = Window::new()?;
    let mut tui = Tui::new(title, program);

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
                Event::Mouse(event) => tui.on_mouse_event(event, &window),
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

pub trait ListenForKey {
    type Output;

    fn on_key_event(&mut self, event: KeyEvent) -> Self::Output;
}

pub trait ListenForMouse {
    type Output;

    fn on_mouse_event(&mut self, event: MouseEvent, window: &Window) -> Self::Output;
}

#[allow(dead_code)]
struct Tui {
    title: String,
    debugger: Debugger,
    tabs: Tabs,
}

const NON_PROGRAM_WIDTH: u16 = 10;
const NON_PROGRAM_HEIGHT: u16 = 12;

impl Tui {
    fn new(title: String, program: Vec<u8>) -> Self {
        Self {
            title,
            debugger: Debugger::new(program),
            tabs: Default::default(),
        }
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

        self.debugger.tick();

        self.tabs.position = self.debugger.current_position();

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
}

impl ListenForKey for Tui {
    type Output = Option<CommandEvent>;

    fn on_key_event(&mut self, event: KeyEvent) -> Self::Output {
        self.tabs.on_key_event(event)
    }
}

impl ListenForMouse for Tui {
    type Output = ();

    fn on_mouse_event(&mut self, event: MouseEvent, window: &Window) -> Self::Output {
        match event.kind {
            MouseEventKind::Moved => {}
            MouseEventKind::Drag(_) => {}
            _ => eprintln!("Mouse event: {:?}", event),
        }
        self.tabs.on_mouse_event(event, window);
    }
}
