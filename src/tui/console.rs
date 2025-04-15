
pub struct ConsoleView {
    pub scroll_height: u16,
}

impl Default for ConsoleView {
    fn default() -> Self {
        Self { scroll_height: 0 }
    }
}