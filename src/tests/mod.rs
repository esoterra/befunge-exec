#[cfg(test)]
mod tests {
    use crate::execution::{ Program, Runtime, Status, Direction, Position };

    const EMPTY_STACK: &[u8] = &[];

    fn one_liner(line: &[u8]) -> Runtime {
        let input = vec![Vec::from(line)];
        let program: Program = input.into();
        Runtime::from(program)
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
        let mut runtime = one_liner(&[opcode, b' ']);

        assert_eq!(Direction::Right, *runtime.get_current_dir());
        assert_eq!(Position { x: 0, y: 0 }, *runtime.get_current_pos());
        assert_eq!(EMPTY_STACK, runtime.get_stack());

        let status = runtime.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Right, *runtime.get_current_dir());
        assert_eq!(Position { x: 1, y: 0 }, *runtime.get_current_pos());
        assert_eq!(&[number], runtime.get_stack());
    }

    #[test]
    fn test_left_arrow() {
        let mut runtime = one_liner(&[b' ', b'<']);

        assert_eq!(Direction::Right, *runtime.get_current_dir());
        assert_eq!(Position { x: 0, y: 0 }, *runtime.get_current_pos());
        assert_eq!(EMPTY_STACK, runtime.get_stack());

        let status = runtime.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Right, *runtime.get_current_dir());
        assert_eq!(Position { x: 1, y: 0 }, *runtime.get_current_pos());
        assert_eq!(EMPTY_STACK, runtime.get_stack());

        let status = runtime.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Left, *runtime.get_current_dir());
        assert_eq!(Position { x: 0, y: 0 }, *runtime.get_current_pos());
        assert_eq!(EMPTY_STACK, runtime.get_stack());
    }
    
    #[test]
    fn test_arrow_loop() {
        let mut runtime = Runtime::from(Program::from(
            vec![
                vec![b'v', b'<'],
                vec![b'>', b'^']
            ]
        ));

        assert_eq!(Direction::Right, *runtime.get_current_dir());
        assert_eq!(Position { x: 0, y: 0 }, *runtime.get_current_pos());
        assert_eq!(EMPTY_STACK, runtime.get_stack());

        let status = runtime.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Down, *runtime.get_current_dir());
        assert_eq!(Position { x: 0, y: 1 }, *runtime.get_current_pos());

        let status = runtime.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Right, *runtime.get_current_dir());
        assert_eq!(Position { x: 1, y: 1 }, *runtime.get_current_pos());
        
        let status = runtime.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Up, *runtime.get_current_dir());
        assert_eq!(Position { x: 1, y: 0 }, *runtime.get_current_pos());
        
        let status = runtime.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Left, *runtime.get_current_dir());
        assert_eq!(Position { x: 0, y: 0 }, *runtime.get_current_pos());
        assert_eq!(EMPTY_STACK, runtime.get_stack());
    }

    #[test]
    fn test_put() {
        let mut runtime = one_liner(&[b'7', b'9', b'0', b'p']);
        
        assert_eq!(Direction::Right, *runtime.get_current_dir());
        assert_eq!(Position { x: 0, y: 0 }, *runtime.get_current_pos());
        assert_eq!(EMPTY_STACK, runtime.get_stack());

        let status = runtime.step();
        assert_eq!(Status::Completed, status);
        let status = runtime.step();
        assert_eq!(Status::Completed, status);
        let status = runtime.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(&[7, 9, 0], runtime.get_stack());

        let status = runtime.step();
        assert_eq!(Status::Completed, status);
        
        assert_eq!(Direction::Right, *runtime.get_current_dir());
        assert_eq!(Position { x: 4, y: 0 }, *runtime.get_current_pos());
        assert_eq!(EMPTY_STACK, runtime.get_stack());

        assert_eq!(7, runtime.get_opcode(&Position { x: 9, y: 0 }));
    }
    
    #[test]
    fn test_get() {
        let mut runtime = one_liner(&[b'7', b'0', b'g', b' ', b' ', b' ', b' ', b'4']);
        
        assert_eq!(Direction::Right, *runtime.get_current_dir());
        assert_eq!(Position { x: 0, y: 0 }, *runtime.get_current_pos());
        assert_eq!(EMPTY_STACK, runtime.get_stack());

        let status = runtime.step();
        assert_eq!(Status::Completed, status);
        let status = runtime.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(&[7, 0], runtime.get_stack());

        let status = runtime.step();
        assert_eq!(Status::Completed, status);

        assert_eq!(Direction::Right, *runtime.get_current_dir());
        assert_eq!(Position { x: 3, y: 0 }, *runtime.get_current_pos());
        assert_eq!(&[b'4'], runtime.get_stack());
    }
}