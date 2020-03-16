use super::core::{ Direction, Position };
use super::program::{ VecProgram };
use super::interpreter::{ Interpreter, Status };

const EMPTY_STACK: &[u8] = &[];

fn one_liner(line: &[u8]) -> Interpreter<VecProgram> {
    let input = vec![Vec::from(line)];
    let program = VecProgram::from_vec(input);
    Interpreter::from(program)
}

#[test]
fn test_push_num() {
    let cases: [(u8, u8); 10] = [ 
        (b'0', 0), (b'1', 1), (b'2', 2), (b'3', 3), (b'4', 4), (b'5', 5), (b'6', 6), (b'7', 7), (b'8', 8), (b'9', 9)
    ];
    for (opcode, number) in cases.into_iter() {
        test_push_num_recipe(*opcode, *number);
    }
}

fn test_push_num_recipe(opcode: u8, number: u8) {
    let mut interpreter = one_liner(&[opcode, b' ']);

    assert_eq!(Direction::Right, interpreter.get_current_dir());
    assert_eq!(Position { x: 0, y: 0 }, interpreter.get_current_pos());
    assert_eq!(EMPTY_STACK, interpreter.get_stack());

    let status = interpreter.step();
    assert_eq!(Status::Completed, status);

    assert_eq!(Direction::Right, interpreter.get_current_dir());
    assert_eq!(Position { x: 1, y: 0 }, interpreter.get_current_pos());
    assert_eq!(&[number], interpreter.get_stack());
}

#[test]
fn test_left_arrow() {
    let mut interpreter = one_liner(&[b' ', b'<']);

    assert_eq!(Direction::Right, interpreter.get_current_dir());
    assert_eq!(Position { x: 0, y: 0 }, interpreter.get_current_pos());
    assert_eq!(EMPTY_STACK, interpreter.get_stack());

    let status = interpreter.step();
    assert_eq!(Status::Completed, status);

    assert_eq!(Direction::Right, interpreter.get_current_dir());
    assert_eq!(Position { x: 1, y: 0 }, interpreter.get_current_pos());
    assert_eq!(EMPTY_STACK, interpreter.get_stack());

    let status = interpreter.step();
    assert_eq!(Status::Completed, status);

    assert_eq!(Direction::Left, interpreter.get_current_dir());
    assert_eq!(Position { x: 0, y: 0 }, interpreter.get_current_pos());
    assert_eq!(EMPTY_STACK, interpreter.get_stack());
}

#[test]
fn test_arrow_loop() {
    let mut interpreter = Interpreter::from(VecProgram::from_vec(
        vec![
            vec![b'v', b'<'],
            vec![b'>', b'^']
        ]
    ));

    assert_eq!(Direction::Right, interpreter.get_current_dir());
    assert_eq!(Position { x: 0, y: 0 }, interpreter.get_current_pos());
    assert_eq!(EMPTY_STACK, interpreter.get_stack());

    let status = interpreter.step();
    assert_eq!(Status::Completed, status);

    assert_eq!(Direction::Down, interpreter.get_current_dir());
    assert_eq!(Position { x: 0, y: 1 }, interpreter.get_current_pos());

    let status = interpreter.step();
    assert_eq!(Status::Completed, status);

    assert_eq!(Direction::Right, interpreter.get_current_dir());
    assert_eq!(Position { x: 1, y: 1 }, interpreter.get_current_pos());
    
    let status = interpreter.step();
    assert_eq!(Status::Completed, status);

    assert_eq!(Direction::Up, interpreter.get_current_dir());
    assert_eq!(Position { x: 1, y: 0 }, interpreter.get_current_pos());
    
    let status = interpreter.step();
    assert_eq!(Status::Completed, status);

    assert_eq!(Direction::Left, interpreter.get_current_dir());
    assert_eq!(Position { x: 0, y: 0 }, interpreter.get_current_pos());
    assert_eq!(EMPTY_STACK, interpreter.get_stack());
}

#[test]
fn test_put() {
    let mut interpreter = one_liner(&[b'7', b'9', b'0', b'p']);
    
    assert_eq!(Direction::Right, interpreter.get_current_dir());
    assert_eq!(Position { x: 0, y: 0 }, interpreter.get_current_pos());
    assert_eq!(EMPTY_STACK, interpreter.get_stack());

    let status = interpreter.step();
    assert_eq!(Status::Completed, status);
    let status = interpreter.step();
    assert_eq!(Status::Completed, status);
    let status = interpreter.step();
    assert_eq!(Status::Completed, status);

    assert_eq!(&[7, 9, 0], interpreter.get_stack());

    let status = interpreter.step();
    assert_eq!(Status::Completed, status);
    
    assert_eq!(Direction::Right, interpreter.get_current_dir());
    assert_eq!(Position { x: 4, y: 0 }, interpreter.get_current_pos());
    assert_eq!(EMPTY_STACK, interpreter.get_stack());

    assert_eq!(7, interpreter.get_opcode(Position { x: 9, y: 0 }));
}

#[test]
fn test_get() {
    let mut interpreter = one_liner(&[b'7', b'0', b'g', b' ', b' ', b' ', b' ', b'4']);
    
    assert_eq!(Direction::Right, interpreter.get_current_dir());
    assert_eq!(Position { x: 0, y: 0 }, interpreter.get_current_pos());
    assert_eq!(EMPTY_STACK, interpreter.get_stack());

    let status = interpreter.step();
    assert_eq!(Status::Completed, status);
    let status = interpreter.step();
    assert_eq!(Status::Completed, status);

    assert_eq!(&[7, 0], interpreter.get_stack());

    let status = interpreter.step();
    assert_eq!(Status::Completed, status);

    assert_eq!(Direction::Right, interpreter.get_current_dir());
    assert_eq!(Position { x: 3, y: 0 }, interpreter.get_current_pos());
    assert_eq!(&[b'4'], interpreter.get_stack());
}