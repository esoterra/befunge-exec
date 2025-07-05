use crate::tui::window::{ConvertToWindowSpace, Window, WindowX, WindowY};

const NON_PROGRAM_WIDTH: u16 = 10;
const NON_PROGRAM_HEIGHT: u16 = 12;

pub fn program_cols(window: &Window) -> u16 {
    window.width() - NON_PROGRAM_WIDTH
}

pub fn program_rows(window: &Window) -> u16 {
    window.height() - NON_PROGRAM_HEIGHT
}

pub fn stack_rows(window: &Window) -> u16 {
    program_rows(window)
}

pub fn stack_rows_parity_even(window: &Window) -> bool {
    program_rows(window) % 2 == 0
}

macro_rules! wrapper_arithmetic {
    ($t:ident) => {
        impl std::ops::Add<u16> for $t {
            type Output = $t;

            fn add(self, rhs: u16) -> Self::Output {
                $t(self.0 + rhs)
            }
        }

        impl std::ops::Sub<u16> for $t {
            type Output = $t;

            fn sub(self, rhs: u16) -> Self::Output {
                $t(self.0 - rhs)
            }
        }
    };
}

macro_rules! convert_add {
    ($from:ident, $to:ident, $amount:expr) => {
        impl ConvertToWindowSpace<$to> for $from {
            fn convert(self, _window: &Window) -> $to {
                $to($amount + self.0)
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgramX(pub u16);
wrapper_arithmetic!(ProgramX);
convert_add!(ProgramX, WindowX, 1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgramY(pub u16);
wrapper_arithmetic!(ProgramY);
convert_add!(ProgramY, WindowY, 1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SidebarX(pub u16);
wrapper_arithmetic!(SidebarX);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SidebarY(pub u16);
wrapper_arithmetic!(SidebarY);

impl SidebarY {
    pub fn max(window: &Window) -> SidebarY {
        SidebarY(program_rows(window) - 1)
    }
}

impl ConvertToWindowSpace<WindowX> for SidebarX {
    fn convert(self, window: &Window) -> WindowX {
        WindowX(program_cols(window) + 1 + self.0)
    }
}
impl ConvertToWindowSpace<WindowY> for SidebarY {
    fn convert(self, _window: &Window) -> WindowY {
        WindowY(1 + self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabY(pub u16);
wrapper_arithmetic!(TabY);

impl ConvertToWindowSpace<WindowY> for TabY {
    fn convert(self, window: &Window) -> WindowY {
        WindowY(program_rows(window) + 4 + self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabHeadingY(pub u16);
wrapper_arithmetic!(TabHeadingY);

impl ConvertToWindowSpace<WindowY> for TabHeadingY {
    fn convert(self, window: &Window) -> WindowY {
        WindowY(program_rows(window) + 1 + self.0)
    }
}
