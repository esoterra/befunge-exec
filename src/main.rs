mod execution;

use std::convert::TryFrom;
use std::str::from_utf8;
use std::io::stdin;
use std::fs::File;
use std::io::Result;
use execution::{ Program, Runtime, Status };

fn main() {
    let _ = run();
}

fn run() -> Result<()> {
    let input = stdin();

    // print!("Enter a file name: ");

    // let mut file_name = String::new();
    // input.read_line(&mut file_name)?;

    let file_name = String::from("./programs/helloworld.b98");

    let file = File::open(file_name)?;
    let program = Program::try_from(file)?;
    let mut runtime = Runtime::from(&program);

    loop {
        let mut buffer = String::new();
        input.read_line(&mut buffer)?;
        let bytes = buffer.as_bytes();

        match bytes.get(0) {
            Some(b's') => {
                step(&mut runtime);
            },
            Some(b'i') => {
                runtime.write_input(&bytes[2..]);
            },
            Some(b'p') => {
                println!("{:?}", runtime.get_current_pos());
            },
            Some(b'l') => {
                let y = runtime.get_current_pos().y;
                let line = program.get_line(y).unwrap_or(&[]);
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

fn step(runtime: &mut Runtime) {
    let status = runtime.step();

    let output = runtime.read_output();
    let output_string = from_utf8(&output).unwrap();
    if output.len() != 0 {
        println!("{}", output_string);
    }
    
    match status {
        Status::Exception   => println!("Encountered an Exception"),
        Status::Terminated  => println!("Encountered an Exception"),
        Status::Waiting     => println!("Waiting for input"),
        Status::Completed   => {}
    }
}