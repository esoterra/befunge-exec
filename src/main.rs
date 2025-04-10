mod core;
mod debug;
mod interpreter;
mod io;
mod program;
mod timeline;
mod tui;

use std::cmp::min;
use std::fs::File;
use std::io::Result;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use clap::{Parser, Subcommand};
use io::StdIO;

use crate::interpreter::{Interpreter, Status};
use crate::program::VecProgram;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Run { path: PathBuf },

    Debug { path: PathBuf },

    Tui,
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Run { path } => run(path),
        Command::Debug { path } => debug::debug(path),
        Command::Tui => {
            tui::print_tui(tui::FocusedTab::Console).unwrap();
            std::process::exit(0);
        }
    };
    if let Err(error) = result {
        eprintln!("{:?}", error);
        std::process::exit(1);
    }
}

fn run(path: PathBuf) -> Result<()> {
    let file = File::open(&path)?;
    let program = VecProgram::try_from(file)?;
    let io = StdIO::default();
    let mut interpreter = Interpreter::new(program, io);

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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::core::{Direction, Position};
    use super::interpreter::{Interpreter, Status};
    use super::program::VecProgram;
    use crate::io::VecIO;

    type DebugInterpreter = Interpreter<VecProgram, VecIO>;

    const EMPTY_STACK: &[u8] = &[];

    fn one_liner(line: &[u8]) -> DebugInterpreter {
        let data = Vec::from(line);
        let program = VecProgram::from(data);
        let io = VecIO::default();
        Interpreter::new(program, io)
    }

    #[test]
    fn test_initial_settings() {
        let interpreter = one_liner(&[]);
        assert_eq!(Direction::Right, interpreter.current_direction());
        assert_eq!(Position { x: 0, y: 0 }, interpreter.current_position());
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
        let mut interpreter = one_liner(&[opcode, b' ']);

        let status = interpreter.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Right, interpreter.current_direction());
        assert_eq!(Position { x: 1, y: 0 }, interpreter.current_position());
        assert_eq!(&[number], interpreter.stack());
    }

    #[test]
    fn test_left_arrow() {
        let mut interpreter = one_liner(&[b' ', b'<']);

        let status = interpreter.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Right, interpreter.current_direction());
        assert_eq!(Position { x: 1, y: 0 }, interpreter.current_position());
        assert_eq!(EMPTY_STACK, interpreter.stack());

        let status = interpreter.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Left, interpreter.current_direction());
        assert_eq!(Position { x: 0, y: 0 }, interpreter.current_position());
        assert_eq!(EMPTY_STACK, interpreter.stack());
    }

    #[test]
    fn test_arrow_loop() {
        let program = VecProgram::from(vec![b'v', b'<', b'\n', b'>', b'^']);
        let io = VecIO::default();
        let mut interpreter = Interpreter::new(program, io);

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
        let mut interpreter = one_liner(&[b' ', b' ', b' ', b'2', b'1', b'0', b'p', b' ']);

        // Step over the 3 spaces and 3 number pushes
        for _i in 0..6 {
            assert_eq!(Status::Completed, interpreter.step());
        }
        // Verify that the numbers are on the stack
        assert_eq!(Position { x: 6, y: 0 }, interpreter.current_position());
        assert_eq!(&[2, 1, 0], interpreter.stack());
        // Step over the p command
        assert_eq!(Status::Completed, interpreter.step());
        // Verify that the position and direction are correct
        assert_eq!(Position { x: 7, y: 0 }, interpreter.current_position());
        assert_eq!(Direction::Right, interpreter.current_direction());
        // Verify that the stack is now empty
        assert_eq!(EMPTY_STACK, interpreter.stack());
        // Verify that the value 2 was placed into the specified position
        assert_eq!(2, interpreter.get_opcode(Position { x: 1, y: 0 }));
    }

    #[test]
    fn test_get() {
        let mut interpreter = one_liner(&[b'7', b'0', b'g', b' ', b' ', b' ', b' ', b'4']);

        assert_eq!(Direction::Right, interpreter.current_direction());
        assert_eq!(Position { x: 0, y: 0 }, interpreter.current_position());
        assert_eq!(EMPTY_STACK, interpreter.stack());

        let status = interpreter.step();
        assert_eq!(Status::Completed, status);
        let status = interpreter.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(&[7, 0], interpreter.stack());

        let status = interpreter.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Right, interpreter.current_direction());
        assert_eq!(Position { x: 3, y: 0 }, interpreter.current_position());
        assert_eq!(&[b'4'], interpreter.stack());
    }
}
