use std::io;
use std::time::{Duration, Instant};

mod draw;
pub mod layout;
pub mod styles;
pub mod tabs;
pub mod text;
mod window;

pub use draw::{Draw, DrawBorder};
pub use tabs::{FocusedTab, Tabs};
pub use window::Window;

use crate::core::Position;
use crate::debugger::Debugger;
use crate::tui::draw::{CursorDisplay, ProgramCellCursor, ProgramCellReset, Sidebar};
use crate::tui::layout::TabHeadingY;
use crate::tui::tabs::CommandEvent;
use crate::tui::window::WindowX;

use crossterm::event::{Event, KeyCode, KeyEvent, MouseEvent};

const TICKS_PER_SECOND: u64 = 40;
const MILLIS_PER_TICK: u64 = 1000 / TICKS_PER_SECOND;

pub fn run_tui(name: String, program: Vec<u8>) -> Result<(), crate::Error> {
    let title = format!("Befunge Tools: {}", name);
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
                    log::info!("Resized to ({}, {})", width, height);
                    window.set_size(width, height);
                    resized = true;
                }
                Event::Key(event) => {
                    if event.code == KeyCode::Esc {
                        break 'tick;
                    }
                    let event = tui.on_key_event(event);
                    if event.is_some() {
                        break 'tick;
                    }
                }
                Event::Mouse(event) => tui.on_mouse_event(event, &window),
                _ => {}
            }
        }

        tui.tick(&mut window, resized)?;
        resized = false;

        next_tick += Duration::from_millis(MILLIS_PER_TICK);

        let now = Instant::now();
        if now >= next_tick {
            log::debug!("Slow frame!!");
        }
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
    counter: u64,
}

impl Tui {
    fn new(title: String, program: Vec<u8>) -> Self {
        Self {
            title,
            debugger: Debugger::new(program),
            tabs: Default::default(),
            counter: 0,
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
        // Update the "ticks within second" counter
        self.counter += 1;
        self.counter %= TICKS_PER_SECOND;

        // Tick the debugger
        let old_pos = self.debugger.current_position();
        let debugger_updated = self.debugger.tick();
        let new_pos = self.debugger.current_position();
        self.tabs.position = new_pos;

        // Check if tabs or terminal are dirty
        let tabs_dirty = self.tabs.dirty;
        let terminal_dirty = self.debugger.io_mut().dirty();

        // Return early if nothing has changed
        let nothing_changed = !resized && !debugger_updated && !tabs_dirty && !terminal_dirty;
        if nothing_changed {
            return Ok(());
        }

        window.start_frame()?;

        let redraw_all = resized;
        let redraw_top = resized;
        let redraw_bot = resized || tabs_dirty || terminal_dirty;

        if redraw_all {
            // redraw everything on resize
            log::info!("Draw everything");
            window.clear()?;
            self.draw_border(window)?;
            self.draw(window)?;
        } else if redraw_bot {
            log::info!("Draw tabs");
            self.tabs.dirty = false;
            window.move_to(WindowX(0), TabHeadingY(0))?;
            window.clear_down()?;
            window.set_style(styles::BORDER)?;
            self.tabs.draw_border(window)?;
            self.draw_border_last(window)?;
            (self.debugger.io(), &self.tabs).draw(window)?;
        }

        self.update_program_cursor(old_pos, new_pos, window)?;

        // If top wasn't redrawn and the debugger has updated, redraw the sidebar
        if !redraw_top && debugger_updated {
            log::info!("Draw sidebar");
            let sidebar = Sidebar {
                debugger: &self.debugger,
            };
            sidebar.draw_border(window)?;
            sidebar.draw(window)?;
        }
        // If bottom wasn't redrawn and the position has changed, redraw the position
        if !redraw_bot && old_pos != new_pos {
            log::info!("Draw cursor position");
            CursorDisplay { pos: new_pos }.draw(window)?;
        }
        // Move the terminal cursor to the focused tab
        self.tabs.move_to_cursor(self.debugger.io(), window)?;
        window.end_frame()
    }

    fn update_program_cursor(
        &mut self,
        old_pos: Position,
        new_pos: Position,
        window: &mut Window,
    ) -> io::Result<()> {
        if new_pos != old_pos {
            self.counter = 0;
            ProgramCellReset {
                debugger: &self.debugger,
                pos: old_pos,
            }
            .draw(window)?;
            ProgramCellCursor {
                debugger: &self.debugger,
                pos: new_pos,
                background_on: true,
            }
            .draw(window)?;
        } else {
            let background_on = self.counter < 20;
            ProgramCellCursor {
                debugger: &self.debugger,
                pos: new_pos,
                background_on,
            }
            .draw(window)?;
        }
        Ok(())
    }
}

struct QuitEvent;

impl ListenForKey for Tui {
    type Output = Option<QuitEvent>;

    fn on_key_event(&mut self, event: KeyEvent) -> Self::Output {
        let command_event = self.tabs.on_key_event(event);
        if let Some(command_event) = command_event {
            match command_event {
                CommandEvent::Load { path } => todo!("Load program in '{}'", path),
                CommandEvent::Step { n } => self.debugger.add_steps(n),
                CommandEvent::Run => self.debugger.start_running(),
                CommandEvent::Pause => self.debugger.pause(),
                CommandEvent::Breakpoint { pos } => self.debugger.toggle_breakpoint(pos),
                CommandEvent::Quit => return Some(QuitEvent),
                CommandEvent::PassToTerminal => {
                    self.debugger.io_mut().on_key_event(event);
                }
            }
        }
        None
    }
}

impl ListenForMouse for Tui {
    type Output = ();

    fn on_mouse_event(&mut self, event: MouseEvent, window: &Window) -> Self::Output {
        self.tabs.on_mouse_event(event, window);
    }
}
