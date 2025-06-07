use super::text::{self, Text};

use std::io::{self, Stdout, Write, stdout};

use crossterm::cursor::{MoveRight, MoveTo, MoveToNextLine};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::style::{ContentStyle, SetStyle};
use crossterm::terminal::{
    BeginSynchronizedUpdate, Clear, ClearType, DisableLineWrap, EnableLineWrap,
    EndSynchronizedUpdate, EnterAlternateScreen, LeaveAlternateScreen, SetTitle, disable_raw_mode,
    enable_raw_mode, size,
};
use crossterm::{QueueableCommand, execute};

pub struct Window {
    stdout: Stdout,
    width: u16,
    height: u16,
}

impl Window {
    pub fn new() -> io::Result<Self> {
        let (width, height) = size()?;
        Ok(Self {
            stdout: stdout(),
            width,
            height,
        })
    }

    pub fn set_title(&mut self, title: &str) -> io::Result<()> {
        execute!(self.stdout, SetTitle(title))?;
        Ok(())
    }

    pub fn set_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn init(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(self.stdout, EnterAlternateScreen)?;
        execute!(self.stdout, Clear(ClearType::All))?;
        execute!(self.stdout, DisableLineWrap)?;
        execute!(self.stdout, EnableMouseCapture)?;
        Ok(())
    }

    pub fn close(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(self.stdout, DisableMouseCapture)?;
        execute!(self.stdout, EnableLineWrap)?;
        execute!(self.stdout, Clear(ClearType::All))?;
        execute!(self.stdout, LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn start_frame(&mut self) -> io::Result<()> {
        execute!(self.stdout, BeginSynchronizedUpdate)
    }

    pub fn end_frame(&mut self) -> io::Result<()> {
        self.stdout.queue(EndSynchronizedUpdate)?;
        self.stdout.flush()
    }

    pub fn clear(&mut self) -> io::Result<()> {
        execute!(self.stdout, Clear(ClearType::All))
    }

    pub fn clear_down(&mut self) -> io::Result<()> {
        execute!(self.stdout, Clear(ClearType::FromCursorDown))
    }

    // Terminal Operation Wrappers

    pub fn move_to(&mut self, x: u16, y: u16) -> io::Result<()> {
        self.stdout.queue(MoveTo(x, y))?;
        Ok(())
    }

    pub fn move_right(&mut self, n: u16) -> io::Result<()> {
        if n != 0 {
            self.stdout.queue(MoveRight(n))?;
        }
        Ok(())
    }

    pub fn set_style(&mut self, style: ContentStyle) -> io::Result<()> {
        self.stdout.queue(SetStyle(style))?;
        Ok(())
    }

    pub fn print<A: AsRef<str>>(&mut self, t: Text<A>) -> io::Result<()> {
        write!(self.stdout, "{}", t.as_ref())
    }

    pub fn print_char(&mut self, c: char) -> io::Result<()> {
        write!(self.stdout, "{}", c)
    }

    // Whole Line Drawing

    pub fn line<A, B>(
        &mut self,
        pre: Text<A>,
        mid: impl text::PrintN,
        end: Text<B>,
    ) -> io::Result<()>
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
            self.stdout.queue(MoveToNextLine(1))?;
            return Ok(());
        }
        let n = self.width - pre.width() - end.width();
        mid.print_n(&mut self.stdout, n)?;
        print!("{}", end.as_ref());
        self.stdout.queue(MoveToNextLine(1))?;
        Ok(())
    }
}
