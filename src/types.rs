#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

impl Cursor {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Viewport {
    pub row_offset: usize,
    pub col_offset: usize,
    pub height: usize,
    pub width: usize,
}

impl Viewport {
    pub fn new(height: usize, width: usize) -> Self {
        Self {
            row_offset: 0,
            col_offset: 0,
            height,
            width,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Insert(char),
    DeleteBackward,
    DeleteForward,
    Newline,
}
