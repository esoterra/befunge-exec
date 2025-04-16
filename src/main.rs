mod analyze;
mod core;
mod debug;
mod interpreter;
mod io;
mod space;
mod timeline;
mod tui;

use std::io::Result;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use std::{cmp::min, fs};

use clap::{Parser, Subcommand};
use io::StdIO;
use space::Space;

use crate::interpreter::{Interpreter, Status};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Run { path: PathBuf },

    Debug { path: PathBuf },

    Tui { path: PathBuf },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Run { path } => run(path),
        Command::Debug { path } => debug::debug(path),
        Command::Tui { path } => {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let program = fs::read(path).unwrap();
            tui::run_tui(name, program).unwrap();
            std::process::exit(0);
        }
    };
    if let Err(error) = result {
        eprintln!("{:?}", error);
        std::process::exit(1);
    }
}

fn run(path: PathBuf) -> Result<()> {
    let program = fs::read(path)?;
    let space = Space::new(&program);
    let io = StdIO::default();
    let mut interpreter = Interpreter::new(space, io);

    let mut wait_count = 0;
    loop {
        let status = interpreter.step(&mut ());
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
    use super::core::{Direction, Position, Cell};
    use super::interpreter::{Interpreter, Status};
    use crate::space::Space;
    use crate::io::VecIO;

    type DebugInterpreter<'src> = Interpreter<VecIO>;

    const EMPTY_STACK: &[Cell] = &[];

    fn one_liner(line: &[u8]) -> DebugInterpreter {
        let program = Vec::from(line);
        let space = Space::new(&program);
        let io = VecIO::default();
        Interpreter::new(space, io)
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
        let mut interpreter = one_liner(&[opcode, b' ']);

        let status = interpreter.step(&mut ());
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Right, interpreter.current_direction());
        assert_eq!(Position { x: 1, y: 0 }, interpreter.current_position());
        assert_eq!(&[Cell(number)], interpreter.stack());
    }

    #[test]
    fn test_left_arrow() {
        let mut interpreter = one_liner(&[b' ', b'<']);

        let status = interpreter.step(&mut ());
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Right, interpreter.current_direction());
        assert_eq!(Position { x: 1, y: 0 }, interpreter.current_position());
        assert_eq!(EMPTY_STACK, interpreter.stack());

        let status = interpreter.step(&mut ());
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Left, interpreter.current_direction());
        assert_eq!(Position::ORIGIN, interpreter.current_position());
        assert_eq!(EMPTY_STACK, interpreter.stack());
    }

    #[test]
    fn test_arrow_loop() {
        let program = vec![b'v', b'<', b'\n', b'>', b'^'];
        let io = VecIO::default();
        let space = Space::new(&program);
        let mut interpreter = Interpreter::new(space, io);

        let sequence = [
            (0, 1, Direction::Down),
            (1, 1, Direction::Right),
            (1, 0, Direction::Up),
            (0, 0, Direction::Left),
        ];

        for _ in 0..255 {
            for (x, y, direction) in sequence.iter().copied() {
                let status = interpreter.step(&mut ());
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
            assert_eq!(Status::Completed, interpreter.step(&mut ()));
        }
        // Verify that the numbers are on the stack
        assert_eq!(Position { x: 6, y: 0 }, interpreter.current_position());
        assert_eq!(&[Cell(2), Cell(1), Cell(0)], interpreter.stack());
        // Step over the p command
        assert_eq!(Status::Completed, interpreter.step(&mut ()));
        // Verify that the position and direction are correct
        assert_eq!(Position { x: 7, y: 0 }, interpreter.current_position());
        assert_eq!(Direction::Right, interpreter.current_direction());
        // Verify that the stack is now empty
        assert_eq!(EMPTY_STACK, interpreter.stack());
        // Verify that the value 2 was placed into the specified position
        assert_eq!(Cell(2), interpreter.space().get_cell(Position { x: 1, y: 0 }));
    }

    #[test]
    fn test_get() {
        let mut interpreter = one_liner(&[b'7', b'0', b'g', b' ', b' ', b' ', b' ', b'4']);

        assert_eq!(Direction::Right, interpreter.current_direction());
        assert_eq!(Position::ORIGIN, interpreter.current_position());
        assert_eq!(EMPTY_STACK, interpreter.stack());

        let status = interpreter.step(&mut ());
        assert_eq!(Status::Completed, status);
        let status = interpreter.step(&mut ());
        assert_eq!(Status::Completed, status);

        assert_eq!(&[Cell(7), Cell(0)], interpreter.stack());

        let status = interpreter.step(&mut ());
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Right, interpreter.current_direction());
        assert_eq!(Position { x: 3, y: 0 }, interpreter.current_position());
        assert_eq!(&[Cell(b'4')], interpreter.stack());
    }
}
