use std::collections::HashSet;
use std::fs::File;
use std::io::Result;
use std::io::{Write, stdin, stdout};
use std::path::PathBuf;
use std::str::from_utf8;

use crate::core::Position;
use crate::interpreter::{Interpreter, Status as InterpreterStatus};
use crate::io::VecIO;
use crate::program::VecProgram;

type DebugInterpreter = Interpreter<VecProgram, VecIO>;

pub fn debug(path: PathBuf) -> Result<()> {
    let input = stdin();

    let file = File::open(&path)?;
    let program = VecProgram::try_from(file)?;
    let io = VecIO::default();
    let mut interpreter = Interpreter::new(program, io);
    let mut breakpoints = HashSet::new();

    loop {
        print!("> ");
        stdout().flush()?;

        let mut buffer = String::new();
        input.read_line(&mut buffer)?;
        let bytes = buffer.trim().as_bytes();

        match bytes.get(0) {
            Some(b's') => {
                // Step 's' command
                step(&mut interpreter);
                interpreter.io().println_output();
            }
            Some(b'r') => {
                // Run 'r' command
                debug_run(&mut interpreter, &breakpoints);
                interpreter.io().println_output();
            }
            Some(b'i') => {
                // Input 'i <input>' command
                interpreter.io().write_input(&bytes[2..]);
            }
            Some(b'p') => {
                // Position 'p' command
                println!("{:?}", interpreter.current_position());
            }
            Some(b'd') => {
                // Debug 'd' print command
                println!("{:?}", interpreter);
            }
            Some(b'b') => {
                // Breakpoint 'b <x> <y>' command
                if let Some(pos) = parse_breakpoint(bytes) {
                    breakpoints.insert(pos);
                } else {
                    println!("Breakpoint (b) takes 2 arguments");
                }
            }
            Some(b'l') => {
                // Line 'l' print command
                let line = interpreter.get_line().unwrap_or(&[]);
                let line_string = from_utf8(line);
                println!("{:?}", line_string.unwrap());
            }
            Some(b'q') => {
                // Quit 'q' command
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

fn parse_breakpoint(command: &[u8]) -> Option<Position> {
    let sections: Vec<&[u8]> = command.split(|c| *c == b' ').collect();
    if sections.len() == 3 {
        let arg0 = from_utf8(sections[1]).ok()?;
        let arg1 = from_utf8(sections[2]).ok()?;
        let x = String::from(arg0).parse().ok()?;
        let y = String::from(arg1).parse().ok()?;

        Some(Position { x, y })
    } else {
        None
    }
}

fn step(interpreter: &mut DebugInterpreter) {
    interpreter.step();
    interpreter.io().println_output();
}

fn debug_run(interpreter: &mut DebugInterpreter, breakpoints: &HashSet<Position>) {
    loop {
        let status = interpreter.step();

        if matches!(
            status,
            InterpreterStatus::Terminated | InterpreterStatus::Waiting
        ) {
            return;
        }

        if breakpoints.contains(&interpreter.current_position()) {
            return;
        }
    }
}
