#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Status {
    Completed, Waiting, Terminated
}

use std::fmt::Debug;

trait Funge<Cell, Vector>
    where
        Vector: Debug {

    fn tick(&mut self) -> Status;

    fn get_pos(&self) -> &Vector;
    fn get_delta(&self) -> &Vector;
    fn get_stack(&self) -> &[&[Cell]];

    fn read_cell(&self, pos: &Vector) -> Cell;

    fn read_output(&mut self) -> Vec<Cell>;
    fn write_input(&mut self, input: &[Cell]);
}

trait FungeCell: Copy + Default + Debug;

trait FungeVector: Copy + Debug {
    fn north() -> Self;
    fn south() -> Self;
    fn east() -> Self;
    fn west() -> Self;
    fn high() -> Self;
    fn low() -> Self;

    fn reverse(self) -> Self;
    fn add(self, other: Self) -> Self;
    fn turn_left(self) -> Self;
    fn turn_right(self) -> Self;
}

trait FungeStack {

    fn push(&mut self, cell: Cell);
    fn pushm(&mut self, cells: Vec<Cell>);
    fn pop(&mut self) -> Cell;
    fn popm(&mut self, num: usize) -> Vec<Cell>;
}

type Test = FungeStack<Vec<Cell>>;

trait FungeState<Cell, Vector, Stack>
    where
        Cell: FungeCell,
        Vector: FungeVector,
        Stack: FungeStack {

    fn cell_get(&self, pos: &Vector) -> Cell;

    fn ips(&self) -> usize;
    fn ip_exists(&self, ip_index: usize) -> bool;

    fn ip_delta_set(&mut self, ip_index: usize, delta: Vector);
    fn ip_delta_get(&mut, ip_index: usize, delta: Vector);
    
    fn ip_pos_set(&mut self, ip_index: usize, pos: Vector);
    fn ip_pos_get(&self, ip_index: usize) -> &Vector;

    fn ip_offset_set(&mut self, ip_index: usize, offset: Vector);
    fn ip_offset_get(&self, ip_index: usize);
    
    /// The STACK is the stack of stacks
    fn stack_push(&mut self, cells: Vec<Cell>);
    fn stack_pop(&mut self) -> Vec<Cell>;
    fn stack_clear(&mut self);

    /// TOSS refers to the Top of Stack Stack it is the last value pushed onto the STACK
    fn get_toss(&mut self) -> Stack;
    /// SOSS refers to the Second on Stack Stack, which is directly below the TOSS
    fn get_soss(&mut self) -> Stack;
}