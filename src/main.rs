mod befunge93;

use std::convert::TryFrom;
use std::str::from_utf8;
use std::io::{ stdin, stdout, Write };
use std::fs::File;
use std::io::Result;
use befunge93::{ Program, Runtime, Status, Position };
use std::collections::HashSet;

fn main() {
    match run() {
        Ok(_) => println!("Shell exited correctly"),
        Err(message) => {
            println!("Shell exited with error");
            println!("{}", message);
        }
    }
}

fn run() -> Result<()> {
    let input = stdin();

    print!("Enter a file name: ");
    stdout().flush()?;

    let mut file_name = String::new();
    input.read_line(&mut file_name)?;
    let file_name = format!("./programs/{}.b93", file_name.trim());
    println!("Loading file {}", file_name);

    let file = File::open(file_name)?;
    let program = Program::try_from(file)?;
    let mut runtime = Runtime::from(program);
    let mut breakpoints = HashSet::new();

    loop {
        print!("> ");
        stdout().flush()?;

        let mut buffer = String::new();
        input.read_line(&mut buffer)?;
        let bytes = buffer.trim().as_bytes();

        match bytes.get(0) {
            Some(b's') => {
                step(&mut runtime);
            },
            Some(b'r') => {
                step_loop(&mut runtime, &breakpoints)?;
            },
            Some(b'i') => {
                runtime.write_input(&bytes[2..]);
            },
            Some(b'p') => {
                println!("{:?}", runtime.get_current_pos());
            },
            Some(b'd') => {
                println!("{:?}", runtime);
            },
            Some(b'b') => {
                if let Some(pos) = parse_breakpoint(bytes) {
                    breakpoints.insert(pos);
                } else {
                    println!("Breakpoint (b) takes 2 arguments");
                }

            },
            Some(b'l') => {
                let line = runtime.get_line().unwrap_or(&[]);
                let line_string = from_utf8(line);
                println!("{:?}", line_string.unwrap());
            },
            Some(b'q') => {
                break;
            },
            _ => {}
        }
    }

    Ok(())
}

fn step(runtime: &mut Runtime) -> Status {
    let status = runtime.step();

    let output = runtime.read_output();
    let output_string = from_utf8(&output).unwrap();
    if output.len() != 0 {
        println!("{}", output_string);
    }

    match status {
        Status::Terminated  => println!("Program terminated"),
        Status::Waiting     => println!("Waiting for input"),
        Status::Completed   => {}
    }

    status
}

fn step_loop(runtime: &mut Runtime, breakpoints: &HashSet<Position>) -> Result<()> {
    loop {
        let status = runtime.step();

        match status {
            Status::Completed => {},
            Status::Terminated  => {
                println!();
                println!("Program terminated");
                break;
            }
            Status::Waiting => {
                println!();
                println!("Waiting for input");
                break;
            }
        }

        let output = runtime.read_output();
        let output_string = from_utf8(&output).unwrap();
        if output.len() != 0 {
            print!("{}", output_string);
        }

        stdout().flush()?;

        if breakpoints.contains(runtime.get_current_pos()) {
            println!("Breakpoint reached");
            break;
        }
    }

    Ok(())
}

fn parse_breakpoint(command: &[u8]) -> Option<Position> {
    let sections: Vec<&[u8]> = command.split(|c| *c == b' ').collect();
    if sections.len() == 3 {
        let arg0 = from_utf8(sections[1]).ok()?;
        let arg1 = from_utf8(sections[2]).ok()?;
        let x: usize = String::from(arg0).parse().ok()?;
        let y: usize = String::from(arg1).parse().ok()?;

        Some(Position { x, y })
    } else {
        None
    }
}