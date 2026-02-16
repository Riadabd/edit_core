use crate::buffer::Buffer;
use crate::text::{char_to_byte_index, slice_line};
use crate::types::{Action, Cursor, Viewport};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Editor {
    buffer: Buffer,
    cursor: Cursor,
    viewport: Viewport,
    dirty: bool,
}

impl Editor {
    pub fn new(buffer: Buffer, viewport: Viewport) -> Self {
        let mut editor = Self {
            buffer,
            cursor: Cursor::new(0, 0),
            viewport,
            dirty: false,
        };

        editor.clamp_cursor();
        editor.ensure_cursor_visible();
        editor
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    pub fn viewport(&self) -> Viewport {
        self.viewport
    }

    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.ensure_cursor_visible();
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn reset_dirty(&mut self) {
        self.dirty = false;
    }

    pub fn apply(&mut self, action: Action) {
        self.clamp_cursor();

        let mut mutated = false;
        match action {
            Action::MoveLeft => self.move_left(),
            Action::MoveRight => self.move_right(),
            Action::MoveWordLeft => self.move_word_left(),
            Action::MoveWordRight => self.move_word_right(),
            Action::MoveUp => self.move_up(),
            Action::MoveDown => self.move_down(),
            Action::Insert(ch) => {
                if ch == '\n' {
                    mutated = self.insert_newline();
                } else {
                    mutated = self.insert_char(ch);
                }
            }
            Action::DeleteBackward => mutated = self.delete_backward(),
            Action::DeleteForward => mutated = self.delete_forward(),
            Action::Newline => mutated = self.insert_newline(),
        }

        if mutated {
            self.dirty = true;
        }

        self.clamp_cursor();
        self.ensure_cursor_visible();
    }

    pub fn visible_lines(&self) -> Vec<String> {
        if self.viewport.height == 0 {
            return Vec::new();
        }

        let start = self.viewport.row_offset;
        if start >= self.buffer.line_count() {
            return Vec::new();
        }

        let end = (start + self.viewport.height).min(self.buffer.line_count());
        let mut lines = Vec::with_capacity(end - start);
        for row in start..end {
            let line = self.buffer.line(row).unwrap_or_default();
            if self.viewport.width == 0 {
                lines.push(String::new());
            } else {
                lines.push(slice_line(
                    line,
                    self.viewport.col_offset,
                    self.viewport.width,
                ));
            }
        }
        lines
    }

    pub fn cursor_screen_pos(&self) -> (usize, usize) {
        (
            self.cursor.row.saturating_sub(self.viewport.row_offset),
            self.cursor.col.saturating_sub(self.viewport.col_offset),
        )
    }

    fn clamp_cursor(&mut self) {
        if self.buffer.lines.is_empty() {
            self.buffer.lines.push(String::new());
        }
        let line_count = self.buffer.line_count();
        if self.cursor.row >= line_count {
            self.cursor.row = line_count.saturating_sub(1);
        }
        let max_col = self.buffer.line_len_chars(self.cursor.row);
        if self.cursor.col > max_col {
            self.cursor.col = max_col;
        }
    }

    fn ensure_cursor_visible(&mut self) {
        if self.viewport.height == 0 || self.cursor.row < self.viewport.row_offset {
            self.viewport.row_offset = self.cursor.row;
        } else if self.cursor.row >= self.viewport.row_offset + self.viewport.height {
            self.viewport.row_offset = self.cursor.row + 1 - self.viewport.height;
        }

        if self.viewport.width == 0 || self.cursor.col < self.viewport.col_offset {
            self.viewport.col_offset = self.cursor.col;
        } else if self.cursor.col >= self.viewport.col_offset + self.viewport.width {
            self.viewport.col_offset = self.cursor.col + 1 - self.viewport.width;
        }
    }

    fn move_left(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
            return;
        }

        if self.cursor.row > 0 {
            self.cursor.row -= 1;
            self.cursor.col = self.buffer.line_len_chars(self.cursor.row);
        }
    }

    fn move_word_left(&mut self) {
        if self.cursor.row == 0 && self.cursor.col == 0 {
            return;
        }

        let mut row = self.cursor.row;
        let mut byte_col;

        if self.cursor.col == 0 {
            row -= 1;
            byte_col = self.buffer.line(row).map_or(0, str::len);
        } else {
            byte_col = self
                .buffer
                .line(row)
                .map_or(0, |line| char_to_byte_index(line, self.cursor.col));
        }

        // Skip whitespace first.
        loop {
            if byte_col == 0 {
                if row > 0 {
                    row -= 1;
                    byte_col = self.buffer.line(row).map_or(0, str::len);
                    continue;
                }

                self.cursor.row = 0;
                self.cursor.col = 0;
                return;
            }

            let Some(line) = self.buffer.line(row) else {
                self.cursor.row = 0;
                self.cursor.col = 0;
                return;
            };

            let Some(ch) = line[..byte_col].chars().next_back() else {
                self.cursor.row = 0;
                self.cursor.col = 0;
                return;
            };

            if ch.is_whitespace() {
                byte_col -= ch.len_utf8();
                continue;
            }

            break;
        }

        // Then jump one token left:
        // - identifier run: [A-Za-z0-9_]+
        // - punctuation/symbol: single character (e.g. ';', '(', ')')
        let Some(line) = self.buffer.line(row) else {
            self.cursor.row = 0;
            self.cursor.col = 0;
            return;
        };

        let Some(ch) = line[..byte_col].chars().next_back() else {
            self.cursor.row = row;
            self.cursor.col = 0;
            return;
        };

        if ch.is_alphanumeric() || ch == '_' {
            while byte_col > 0 {
                let Some(prev) = line[..byte_col].chars().next_back() else {
                    break;
                };
                if prev.is_alphanumeric() || prev == '_' {
                    byte_col -= prev.len_utf8();
                } else {
                    break;
                }
            }
        } else {
            byte_col -= ch.len_utf8();
        }

        self.cursor.row = row;
        self.cursor.col = line[..byte_col].chars().count();
    }

    fn move_word_right(&mut self) {
        let mut row = self.cursor.row;
        let mut byte_col = self
            .buffer
            .line(row)
            .map_or(0, |line| char_to_byte_index(line, self.cursor.col));

        // If we are at end of line, move to the next line before token scanning.
        if let Some(line) = self.buffer.line(row)
            && byte_col >= line.len()
            && row + 1 < self.buffer.line_count()
        {
            row += 1;
            byte_col = 0;
        }

        // Skip whitespace first.
        loop {
            let Some(line) = self.buffer.line(row) else {
                return;
            };

            if byte_col >= line.len() {
                if row + 1 < self.buffer.line_count() {
                    row += 1;
                    byte_col = 0;
                    continue;
                }
                self.cursor.row = row;
                self.cursor.col = line.chars().count();
                return;
            }

            let Some(ch) = line[byte_col..].chars().next() else {
                self.cursor.row = row;
                self.cursor.col = line.chars().count();
                return;
            };

            if ch.is_whitespace() {
                byte_col += ch.len_utf8();
                continue;
            }

            break;
        }

        // Then jump one token right:
        // - identifier run: [A-Za-z0-9_]+
        // - punctuation/symbol: single character (e.g. ';', '(', ')')
        let Some(line) = self.buffer.line(row) else {
            return;
        };

        let Some(ch) = line[byte_col..].chars().next() else {
            self.cursor.row = row;
            self.cursor.col = line.chars().count();
            return;
        };

        if ch.is_alphanumeric() || ch == '_' {
            while byte_col < line.len() {
                let Some(next) = line[byte_col..].chars().next() else {
                    break;
                };
                if next.is_alphanumeric() || next == '_' {
                    byte_col += next.len_utf8();
                } else {
                    break;
                }
            }
        } else {
            byte_col += ch.len_utf8();
        }

        self.cursor.row = row;
        self.cursor.col = line[..byte_col].chars().count();
    }

    fn move_right(&mut self) {
        let line_len = self.buffer.line_len_chars(self.cursor.row);
        if self.cursor.col < line_len {
            self.cursor.col += 1;
            return;
        }

        if self.cursor.row + 1 < self.buffer.line_count() {
            self.cursor.row += 1;
            self.cursor.col = 0;
        }
    }

    fn move_up(&mut self) {
        if self.cursor.row == 0 {
            return;
        }

        self.cursor.row -= 1;
        let line_len = self.buffer.line_len_chars(self.cursor.row);
        if self.cursor.col > line_len {
            self.cursor.col = line_len;
        }
    }

    fn move_down(&mut self) {
        if self.cursor.row + 1 >= self.buffer.line_count() {
            return;
        }

        self.cursor.row += 1;
        let line_len = self.buffer.line_len_chars(self.cursor.row);
        if self.cursor.col > line_len {
            self.cursor.col = line_len;
        }
    }

    fn insert_char(&mut self, ch: char) -> bool {
        let row = self.cursor.row;
        let col = self.cursor.col;
        let line = match self.buffer.lines.get_mut(row) {
            Some(value) => value,
            None => return false,
        };
        let line_len = line.chars().count();
        let col = col.min(line_len);
        let byte_idx = char_to_byte_index(line, col);

        line.insert(byte_idx, ch);

        self.cursor.col = col + 1;

        true
    }

    fn insert_newline(&mut self) -> bool {
        let row = self.cursor.row;
        let col = self.cursor.col;
        let line = match self.buffer.lines.get_mut(row) {
            Some(value) => value,
            None => return false,
        };
        let line_len = line.chars().count();
        let col = col.min(line_len);
        let byte_idx = char_to_byte_index(line, col);
        let right = line.split_off(byte_idx);

        self.buffer.lines.insert(row + 1, right);
        self.cursor.row = row + 1;
        self.cursor.col = 0;

        true
    }

    fn delete_backward(&mut self) -> bool {
        let row = self.cursor.row;
        let col = self.cursor.col;

        if row >= self.buffer.line_count() {
            return false;
        }

        if col > 0 {
            let line = match self.buffer.lines.get_mut(row) {
                Some(value) => value,
                None => return false,
            };

            let line_len = line.chars().count();
            let col = col.min(line_len);

            if col == 0 {
                return false;
            }

            let remove_col = col - 1;
            let start = char_to_byte_index(line, remove_col);
            let end = char_to_byte_index(line, remove_col + 1);

            line.replace_range(start..end, "");

            self.cursor.col = remove_col;

            return true;
        }

        if row > 0 {
            let prev_row = row - 1;
            let prev_len = self.buffer.line_len_chars(prev_row);
            let current_line = self.buffer.lines.remove(row);

            if let Some(prev_line) = self.buffer.lines.get_mut(prev_row) {
                prev_line.push_str(&current_line);
                self.cursor.row = prev_row;
                self.cursor.col = prev_len;
                return true;
            }
        }

        false
    }

    fn delete_forward(&mut self) -> bool {
        let row = self.cursor.row;
        let col = self.cursor.col;

        if row >= self.buffer.line_count() {
            return false;
        }

        let line_len = self.buffer.line_len_chars(row);
        if col < line_len {
            let line = match self.buffer.lines.get_mut(row) {
                Some(value) => value,
                None => return false,
            };
            let start = char_to_byte_index(line, col);
            let end = char_to_byte_index(line, col + 1);
            line.replace_range(start..end, "");
            return true;
        }

        if col == line_len && row + 1 < self.buffer.line_count() {
            let next_line = self.buffer.lines.remove(row + 1);
            if let Some(line) = self.buffer.lines.get_mut(row) {
                line.push_str(&next_line);
                return true;
            }
        }

        false
    }
}
