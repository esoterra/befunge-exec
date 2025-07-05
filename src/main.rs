#![allow(clippy::collapsible_else_if)]
mod analyze;
mod core;
mod debugger;
mod interpreter;
mod io;
mod record;
mod space;
mod terminal;
mod tui;

use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use std::{cmp::min, fs};

use clap::{Parser, Subcommand};
use ftail::Ftail;
use log::LevelFilter;
use space::Space;
use thiserror::Error;

use crate::interpreter::{Interpreter, InterpreterError, Status};

/// Befunge runtime and development tools.
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run a Befunge program.
    Run { path: PathBuf },
    /// Run the specified program in an interactive debugger.
    Debug {
        /// Path of program to run.
        path: PathBuf,
        /// Log level
        #[arg(long)]
        log_level: Option<LevelFilter>
    },
}

#[derive(Debug, Error)]
enum Error {
    #[error("I/O error")]
    IO(#[from] std::io::Error),
    #[error("Interpreter Error")]
    Interpreter(#[from] InterpreterError),
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Run { path } => run(path),
        Command::Debug { path, log_level } => {
            init_logging(log_level);
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let program = fs::read(path).unwrap();
            tui::run_tui(name, program)
        }
    };
    if let Err(error) = result {
        log::error!("{:?}", error);
        std::process::exit(1);
    }
    std::process::exit(0)
}

fn init_logging(log_level: Option<LevelFilter>) {
    // Don't log at all if log level is off
    if matches!(log_level, Some(LevelFilter::Off)) {
        return;
    }
    // Default to DEBUG
    let level = log_level.unwrap_or(log::LevelFilter::Debug);

    let mut path = PathBuf::from(std::env::var("HOME").unwrap());
    path.push(".bft/logs");
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }

    // Determine the name of the file
    let t = chrono::Utc::now().format("%y-%m-%d-%H-%S");
    let file_name = format!("bft_log_{}.txt", t);
    path.push(file_name);
    let mut counter = 0;
    while path.exists() {
        path.set_file_name(format!("bft_log_{}-({}).txt", t, counter));
        counter += 1;
    }

    // Create the logger and print if there's an error.
    let logger = Ftail::new().single_file(&path, false, level).init();
    if let Err(error) = logger {
        log::info!("{:?}", error);
        std::process::exit(1);
    }
}

fn run(path: PathBuf) -> Result<(), Error> {
    let program = fs::read(path)?;
    let space = Space::new(&program);
    let mut interpreter = Interpreter::new_std(space);

    let mut wait_count = 0;
    loop {
        let status = interpreter.step();
        match status {
            Status::Completed => {
                wait_count = 0;
            }
            Status::Waiting => {
                wait_count += 1;
                let wait = min(wait_count, 500);
                let wait = Duration::from_millis(wait);
                sleep(wait);
            }
            Status::Terminated => {
                return Ok(());
            }
            Status::Error(error) => {
                return Err(error.into());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::core::{Direction, Position, StackCell};
    use super::interpreter::{Interpreter, Status};
    use crate::core::GridCell;
    use crate::io::VecIO;
    use crate::record::StdOutEventLog;
    use crate::space::Space;

    type DebugInterpreter<'src> = Interpreter<VecIO, StdOutEventLog>;

    const EMPTY_STACK: &[StackCell] = &[];

    fn one_liner(line: &[u8]) -> DebugInterpreter {
        let program = Vec::from(line);
        let space = Space::new(&program);
        let io = VecIO::default();
        Interpreter::new(space, io, StdOutEventLog)
    }

    #[test]
    fn test_initial_settings() {
        let interpreter = one_liner(&[]);
        assert_eq!(Direction::Right, interpreter.current_direction());
        assert_eq!(Position::ORIGIN, interpreter.current_position());
        assert_eq!(EMPTY_STACK, interpreter.stack());
    }

    #[test]
    fn test_push_num() {
        let cases: [(u8, u8); 10] = [
            (b'0', 0),
            (b'1', 1),
            (b'2', 2),
            (b'3', 3),
            (b'4', 4),
            (b'5', 5),
            (b'6', 6),
            (b'7', 7),
            (b'8', 8),
            (b'9', 9),
        ];
        for (opcode, number) in cases.iter() {
            test_push_num_recipe(*opcode, *number);
        }
    }

    fn test_push_num_recipe(opcode: u8, number: u8) {
        let mut interpreter = one_liner(&[opcode]);

        let status = interpreter.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Right, interpreter.current_direction());
        assert_eq!(Position { x: 0, y: 0 }, interpreter.current_position());
        assert_eq!(&[StackCell(number as i32)], interpreter.stack());
    }

    #[test]
    fn test_left_arrow() {
        let mut interpreter = one_liner(b"1<");

        let status = interpreter.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Right, interpreter.current_direction());
        assert_eq!(Position { x: 1, y: 0 }, interpreter.current_position());
        assert_eq!(&[StackCell(1)], interpreter.stack());

        let status = interpreter.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Left, interpreter.current_direction());
        assert_eq!(Position::ORIGIN, interpreter.current_position());
        assert_eq!(&[StackCell(1)], interpreter.stack());
    }

    #[test]
    fn test_arrow_loop() {
        let program = vec![b'v', b'<', b'\n', b'>', b'^'];
        let io = VecIO::default();
        let space = Space::new(&program);
        let mut interpreter = Interpreter::new(space, io, StdOutEventLog);

        let sequence = [
            (0, 1, Direction::Down),
            (1, 1, Direction::Right),
            (1, 0, Direction::Up),
            (0, 0, Direction::Left),
        ];

        for _ in 0..255 {
            for (x, y, direction) in sequence.iter().copied() {
                let status = interpreter.step();
                assert_eq!(Status::Completed, status);

                assert_eq!(direction, interpreter.current_direction());
                assert_eq!(Position { x, y }, interpreter.current_position());
                assert_eq!(EMPTY_STACK, interpreter.stack());
            }
        }
    }

    #[test]
    fn test_put() {
        let mut interpreter = one_liner(b"211p3");

        // Push the three numbers
        assert_eq!(Status::Completed, interpreter.step());
        assert_eq!(Status::Completed, interpreter.step());
        assert_eq!(Status::Completed, interpreter.step());
        // Verify that the numbers are on the stack
        assert_eq!(Position { x: 3, y: 0 }, interpreter.current_position());
        assert_eq!(
            &[StackCell(2), StackCell(1), StackCell(1)],
            interpreter.stack()
        );
        // Step over the p command
        assert_eq!(Status::Completed, interpreter.step());
        // Verify that the position and direction are correct
        assert_eq!(Position { x: 4, y: 0 }, interpreter.current_position());
        // Verify that the stack is now empty
        assert_eq!(EMPTY_STACK, interpreter.stack());
        // Verify that the value 2 was placed into the specified position
        assert_eq!(
            GridCell(2),
            interpreter.space().get_cell(Position { x: 1, y: 1 })
        );
    }

    #[test]
    fn test_get() {
        let mut interpreter = one_liner(b"70g    4");

        assert_eq!(Direction::Right, interpreter.current_direction());
        assert_eq!(Position::ORIGIN, interpreter.current_position());
        assert_eq!(EMPTY_STACK, interpreter.stack());

        let status = interpreter.step();
        assert_eq!(Status::Completed, status);
        let status = interpreter.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(&[StackCell(7), StackCell(0)], interpreter.stack());

        let status = interpreter.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Right, interpreter.current_direction());
        assert_eq!(Position { x: 7, y: 0 }, interpreter.current_position());
        assert_eq!(&[StackCell(b'4' as i32)], interpreter.stack());
    }
}
