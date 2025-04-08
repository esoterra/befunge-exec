use std::{
    collections::VecDeque,
    io::{Read, Stdin, Stdout, Write, stdin, stdout},
};

pub trait IO {
    fn read_byte(&mut self) -> Option<u8>;
    fn read_number(&mut self) -> Option<u8>;
    fn write(&mut self, buf: &[u8]);
}

pub struct StdIO {
    input: InputBuffer,
    stdout: Stdout,
}

impl Default for StdIO {
    fn default() -> Self {
        Self { input: Default::default(), stdout: stdout() }
    }
}

pub struct InputBuffer {
    stdin: Stdin,
    buffer: [u8; 32],
    offset: usize,
    length: usize,
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self {
            stdin: stdin(),
            buffer: [0; 32],
            offset: 0,
            length: 0,
        }
    }
}

impl std::fmt::Debug for InputBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputBuffer")
            .field("stdin", &self.stdin)
            .field("buffer", &format!("{:x?}", &self.buffer))
            .field("offset", &self.offset)
            .field("length", &self.length)
            .finish()
    }
}

impl InputBuffer {
    fn read_byte(&mut self) -> Option<u8> {
        if self.is_empty() {
            let n = self.stdin.read(&mut self.buffer).unwrap();
            if n == 0 {
                return None;
            }
            self.offset = 0;
            self.length = n;
        }

        let value = self.buffer[self.offset];
        self.offset += 1;
        self.length -= 1;
        return Some(value);
    }

    fn is_empty(&self) -> bool {
        self.length == 0
    }

    fn skip_and_shift(&mut self, skip: usize) {
        assert!(skip <= self.length, "Tried to skip ({}) more than was available ({})", skip, self.length);
        let start = self.offset+skip;
        let end = self.offset+self.length;
        if start != end {
            let src = start..end;
            self.buffer.copy_within(src, 0);
        }
        self.offset = 0;
        self.length -= skip;
    }

    /// Returns the amount more that was able to be read
    fn read_more(&mut self) -> usize {
        let buf = match self.length {
            0 => &mut self.buffer,
            _ => {
                let start = self.offset;
                let end = self.offset + self.length;
                &mut self.buffer[start..end]
            }
        };
        let n = self.stdin.read(buf).unwrap();
        self.length += n;
        n
    }

    fn bytes(&self) -> impl Iterator<Item = u8> {
        let start = self.offset;
        let end = self.offset + self.length;
        let buf = &self.buffer[start..end];
        buf.iter().copied()
    }
}

impl IO for StdIO {
    fn read_byte(&mut self) -> Option<u8> {
        self.input.read_byte()
    }
    
    fn read_number(&mut self) -> Option<u8> {
        if self.input.is_empty() {
            let n = self.input.read_more();
            if n == 0 {
                return None;
            }
        }
        loop {
            let iter = self.input.bytes();
            match try_read_number(iter) {
                Ok((offset, num)) => {
                    self.input.skip_and_shift(offset);
                    return Some(num);
                },
                Err(skippable) => {
                    self.input.skip_and_shift(skippable);
                    if self.input.read_more() == 0 {
                        return None;
                    }
                },
            }
        }
    }

    fn write(&mut self, buf: &[u8]) {
        self.stdout.write(buf).unwrap();
    }
}

#[derive(Default, Debug)]
pub struct VecIO {
    input_buffer: VecDeque<u8>,
    output_buffer: Vec<u8>,
}

impl VecIO {
    /// Appends data to the input buffer
    pub fn write_input(&mut self, input: &[u8]) {
        for byte in input {
            self.input_buffer.push_back(*byte);
        }
    }

    pub fn println_output(&mut self) {
        let mut out = stdout();
        if !self.output_buffer.is_empty() {
            out.write(&self.output_buffer).unwrap();
            write!(out, "\n").unwrap();
            out.flush().unwrap();
            self.output_buffer.clear();
        }
    }
}

impl IO for VecIO {
    fn read_byte(&mut self) -> Option<u8> {
        self.input_buffer.pop_front()
    }
    
    fn read_number(&mut self) -> Option<u8> {
        let iter = self.input_buffer.iter().copied();
        let (offset, byte) = try_read_number(iter).ok()?;
        for _ in 0..offset {
            self.input_buffer.pop_front();
        }
        Some(byte)
    }

    fn write(&mut self, buf: &[u8]) {
        self.output_buffer.extend_from_slice(buf);
    }
}

// Either reads a number from the iterator successfully 
// returning the number of bytes read and the value of the number
// or returns that a number could not be read and how many bytes can be skipped.
fn try_read_number(iter: impl Iterator<Item = u8>) -> Result<(usize, u8), usize> {
    let mut iter = iter.enumerate();
    let (mut offset, mut num) = base_number(&mut iter)?;
    let skippable = offset - 1;
    while let Some((i, byte)) = iter.next() {
        if matches!(byte, b'0'..=b'9') {
            let value = byte - b'0';
            if let Some(new_num) = try_combine(num, value) {
                offset = i+1;
                num = new_num;
            } else {
                return Ok((offset, num));
            }
        } else {
            return Ok((offset, num));
        }
    }
    Err(skippable)
}

fn base_number(iter: &mut impl Iterator<Item = (usize, u8)>) -> Result<(usize, u8), usize> {
    let mut skippable = 0;
    while let Some((i, byte)) = iter.next() {
        let n = i+1;
        if matches!(byte, b'0'..=b'9') {
            let value = byte - b'0';
            return Ok((n, value));
        } else {
            skippable = n;
        }
    };
    Err(skippable)
}

// If a number is greater than SHIFT_MAX, 10*number is greater than 255.
const SHIFT_MAX: u8 = 25;

fn try_combine(old: u8, new: u8) -> Option<u8> {
    if old > SHIFT_MAX {
        return None;
    }
    let value = old * 10;
    let remaining = 255 - value;
    if new > remaining {
        return None;
    }
    Some(value + new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_read_number() {
        let cases = [
            // Large enough numbers
            ("alice bob charlie23562347", Ok((20, 235))),
            // Num then non-number
            ("@#$@%^$#%^%^3a", Ok((13, 3))),
            ("a66$", Ok((3, 66))),
            ("1 ", Ok((1, 1))),
            ("24\n", Ok((2, 24))),
            // Incomplete numbers
            ("@#$@%^$#%^%^3", Err(12)),
            ("a66", Err(1)),
            ("1", Err(0)),
            ("11", Err(0)),
            // No numbers at all
            ("abcdefg", Err(7)),
            ("@#$%@#$*&%^", Err(11)),
            ("a b c", Err(5)),
        ];
        for (input, expected) in cases.into_iter() {
            let iter = input.as_bytes().iter().copied();
            let actual = try_read_number(iter);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_base_num() {
        let cases = [
            // Skips non-numbers
            ("alice bob charlie23562347", Ok((18, 2))),
            ("@#$@%^$#%^%^3", Ok((13, 3))),
            ("a66", Ok((2, 6))),
            // Gives None if there isn't a number
            ("abcdefg", Err(7)),
        ];
        for (input, expected) in cases.into_iter() {
            let mut iter = input.as_bytes().iter().copied().enumerate();
            let actual = base_number(&mut iter);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_try_combine() {
        assert_eq!(Some(11), try_combine(1, 1));

        for i in 0..24 {
            for j in 0..9 {
                assert_eq!(Some(10*i + j), try_combine(i, j));
            }
        }

        assert_eq!(Some(250), try_combine(25, 0));
        assert_eq!(Some(251), try_combine(25, 1));
        assert_eq!(Some(252), try_combine(25, 2));
        assert_eq!(Some(253), try_combine(25, 3));
        assert_eq!(Some(254), try_combine(25, 4));
        assert_eq!(Some(255), try_combine(25, 5));
        assert_eq!(None, try_combine(25, 6));
        assert_eq!(None, try_combine(25, 7));
        assert_eq!(None, try_combine(25, 8));
        assert_eq!(None, try_combine(25, 9));

        for i in 26..=255 {
            for j in 0..9 {
                assert_eq!(None, try_combine(i, j));
            }
        }
    }
}