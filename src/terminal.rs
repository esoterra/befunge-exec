use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{
    io::{IO, try_read_number},
    tui::ListenForKey,
};

/// Simulates the behavior of a terminal prompt.
/// Allows input to be edited until committed (using newline)
pub struct VirtualTerminal {
    /// Append-only display data
    /// Contains program output and committed user input
    display: Vec<u8>,
    /// The index of newline characters
    /// Used to determine where lines start and end.
    newline_indices: Vec<usize>,
    /// A deque of input that has been committed and can be read
    available_input: VecDeque<u8>,
    /// Uncommitted user input that can still be modified
    /// Treated as "floating" on top of / after the display data
    uncommitted: Vec<u8>,
    /// Offset of the cursor in the uncommitted buffer
    cursor: usize,
    /// Whether changes have been observed.
    dirty: bool,
}

impl Default for VirtualTerminal {
    fn default() -> Self {
        // Capacities chosen by vibes so that most typical program evaluations
        // shouldn't ever have to resize them.
        Self {
            display: Vec::with_capacity(512),
            newline_indices: Vec::with_capacity(32),
            available_input: VecDeque::with_capacity(512),
            uncommitted: Vec::with_capacity(64),
            cursor: 0,
            dirty: false,
        }
    }
}

impl ListenForKey for VirtualTerminal {
    type Output = ();

    fn on_key_event(&mut self, event: KeyEvent) -> Self::Output {
        if matches!(event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            match event.code {
                KeyCode::Backspace => self.backspace(),
                KeyCode::Enter => self.commit(),
                KeyCode::Left => self.left(),
                KeyCode::Right => self.right(),
                KeyCode::Delete => self.delete(),
                KeyCode::Char(c) => self.input_key(c, event.modifiers),
                _ => {}
            }
        }
    }
}

impl VirtualTerminal {
    fn left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        self.dirty = true;
    }

    fn right(&mut self) {
        if self.cursor == self.uncommitted.len() {
            return;
        }
        self.cursor += 1;
        self.dirty = true;
    }

    fn input_key(&mut self, c: char, modifiers: KeyModifiers) {
        // Ignore non-ascii characters
        if !c.is_ascii() {
            return;
        }
        // Apply shift modifier if necessary
        let c = if modifiers.contains(KeyModifiers::SHIFT) {
            c.to_ascii_uppercase()
        } else {
            c
        };
        // Insert character and shift cursor
        self.uncommitted.insert(self.cursor, c as u8);
        self.cursor += 1;
        self.dirty = true;
    }

    fn backspace(&mut self) {
        // Backspace in an empty prompt does nothing
        if self.uncommitted.is_empty() {
            return;
        }
        // Delete at the front of the prompt does nothing
        if self.cursor == 0 {
            return;
        }
        self.uncommitted.remove(self.cursor - 1);
        self.cursor -= 1;
        self.dirty = true;
    }

    fn delete(&mut self) {
        // Delete at the end of the prompt does nothing
        if self.cursor == self.uncommitted.len() {
            return;
        }
        self.uncommitted.remove(self.cursor);
        self.cursor -= 1;
        self.dirty = true;
    }

    fn commit(&mut self) {
        // Append and record a newline
        let i = self.display.len() + self.uncommitted.len();
        self.newline_indices.push(i);
        self.uncommitted.push(b'\n');
        // Append the uncommitted buffer to the input and display
        self.available_input.extend(&self.uncommitted);
        self.display.extend(&self.uncommitted);
        // Clear the uncommitted buffer
        self.uncommitted.clear();
        // Reset the cursor to zero
        self.cursor = 0;
        self.dirty = true;
    }

    // get a line of committed terminal output
    pub fn get_line(&self, line: usize) -> Option<&[u8]> {
        let newlines = self.newline_indices.len();

        if line > newlines {
            return None;
        }

        let start = {
            if line == 0 {
                0
            } else {
                self.newline_indices[line - 1] + 1
            }
        };

        let end = {
            if line == newlines {
                self.display.len()
            } else {
                self.newline_indices[line]
            }
        };

        Some(&self.display[start..end])
    }

    pub fn num_lines(&self) -> usize {
        self.newline_indices.len() + 1
    }

    pub fn uncommitted(&self) -> &[u8] {
        &self.uncommitted
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn dirty(&mut self) -> bool {
        let dirty = self.dirty;
        self.dirty = false;
        dirty
    }
}

impl IO for VirtualTerminal {
    fn read_byte(&mut self) -> Option<u8> {
        self.available_input.pop_front()
    }

    fn read_number(&mut self) -> Option<u8> {
        let iter = self.available_input.iter().copied();
        let (skip, output) = match try_read_number(iter) {
            Ok((bytes, n)) => (bytes, Some(n)),
            Err(skip) => (skip, None),
        };
        for _ in 0..skip {
            self.available_input.pop_front();
        }
        output
    }

    fn write(&mut self, buf: &[u8]) {
        let len = self.display.len();
        for (i, b) in buf.iter().copied().enumerate() {
            if b == b'\n' {
                self.newline_indices.push(len + i);
            }
        }
        self.display.extend_from_slice(buf);
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: KeyModifiers = KeyModifiers::empty();

    #[test]
    fn test_typing() {
        let mut t = VirtualTerminal::default();
        t.input_key('a', NONE);
        t.input_key('b', NONE);
        t.input_key('c', NONE);
        t.input_key('d', NONE);
        assert_eq!(t.uncommitted, b"abcd");
        assert_eq!(t.cursor, 4);
    }

    #[test]
    fn test_lines() {
        let mut t = VirtualTerminal::default();
        t.write(b"aaaaaaaa");
        assert_eq!(t.num_lines(), 1);
        assert_eq!(t.get_line(0), Some(b"aaaaaaaa".as_slice()));
        t.write(b"\n");
        assert_eq!(t.num_lines(), 2);
        assert_eq!(t.get_line(0), Some(b"aaaaaaaa".as_slice()));
        assert_eq!(t.get_line(1), Some(b"".as_slice()));
        t.write(b"asdf\nasdf\nasdf\na");
        assert_eq!(t.num_lines(), 5);
        assert_eq!(t.get_line(0), Some(b"aaaaaaaa".as_slice()));
        assert_eq!(t.get_line(1), Some(b"asdf".as_slice()));
        assert_eq!(t.get_line(2), Some(b"asdf".as_slice()));
        assert_eq!(t.get_line(3), Some(b"asdf".as_slice()));
        assert_eq!(t.get_line(4), Some(b"a".as_slice()));
    }

    #[test]
    fn test_prompt() {
        let mut t = VirtualTerminal::default();
        // Write prompt
        t.write(b"Input number!");
        assert_eq!(t.display, b"Input number!");
        assert_eq!(t.num_lines(), 1);
        assert_eq!(t.get_line(0), Some(b"Input number!".as_slice()));
        // Input response
        t.input_key('1', NONE);
        t.input_key('2', NONE);
        t.commit();
        assert_eq!(t.display, b"Input number!12\n");
        assert_eq!(t.num_lines(), 2);
        assert_eq!(t.get_line(0), Some(b"Input number!12".as_slice()));
        assert_eq!(t.get_line(1), Some(b"".as_slice()));
        // Check input is available
        let input: Vec<_> = t.available_input.iter().copied().collect();
        assert_eq!(input, vec![b'1', b'2', b'\n']);
        // Read number from input
        let n = t.read_number();
        assert_eq!(n, Some(12));
    }
}
