# edit_core

A small, dependency-free text editing core for building a terminal or GUI editor. It models a buffer of lines, a cursor, and a viewport, and applies editing actions (movement, insertion, deletion) while keeping the cursor visible.

## Features

- Line-oriented buffer with UTF-8 safe, character-based indexing.
- Cursor movement across lines with automatic clamping.
- UTF-8-aware word movement (`MoveWordLeft`, `MoveWordRight`) with consistent token jumps.
- Insert, delete, and newline actions.
- Viewport scrolling with visible line slicing.
- Dirty flag tracking for mutations only.

## Usage

```rust
use edit_core::{Action, Buffer, Editor, Viewport};

let buffer = Buffer::from_text("hello\nworld");
let viewport = Viewport::new(5, 40);
let mut editor = Editor::new(buffer, viewport);

editor.apply(Action::MoveRight);
editor.apply(Action::Insert('!'));

assert_eq!(editor.buffer().as_text(), "h!ello\nworld");
```

## Public API

### Types

- `Buffer`: Stores the text as a vector of lines.
- `Cursor`: Row and column position (character-based).
- `Viewport`: Visible window with row/column offsets and size.
- `Action`: Editing actions (character movement, word movement, insertion, deletion, newline).
- `Editor`: Applies actions to a buffer while managing cursor, viewport, and dirty state.

### `Buffer`

- `Buffer::new() -> Buffer`
- `Buffer::from_text(text: &str) -> Buffer`
- `Buffer::as_text(&self) -> String`
- `Buffer::line(&self, row: usize) -> Option<&str>`
- `Buffer::line_count(&self) -> usize`

### `Cursor`

- `Cursor::new(row: usize, col: usize) -> Cursor`
- Fields: `row`, `col`

### `Viewport`

- `Viewport::new(height: usize, width: usize) -> Viewport`
- Fields: `row_offset`, `col_offset`, `height`, `width`

### `Action`

- `MoveLeft`, `MoveRight`, `MoveWordLeft`, `MoveWordRight`, `MoveUp`, `MoveDown`
- `Insert(char)`
- `DeleteBackward`, `DeleteForward`
- `Newline`

Word movement behavior:
- Skip leading whitespace first.
- Jump an identifier run `[A-Za-z0-9_]+` as one token.
- Treat punctuation/symbol characters as single-token jumps.
- Cross line boundaries when needed.

### `Editor`

- `Editor::new(buffer: Buffer, viewport: Viewport) -> Editor`
- `Editor::buffer(&self) -> &Buffer`
- `Editor::cursor(&self) -> Cursor`
- `Editor::viewport(&self) -> Viewport`
- `Editor::set_viewport(&mut self, viewport: Viewport)`
- `Editor::is_dirty(&self) -> bool`
- `Editor::reset_dirty(&mut self)`
- `Editor::apply(&mut self, action: Action)`
- `Editor::visible_lines(&self) -> Vec<String>`
- `Editor::cursor_screen_pos(&self) -> (usize, usize)`

## Notes

- Rows and columns are character indices, not byte offsets.
- `visible_lines` returns slices based on the current viewport offsets and size.

## Internal Layout

- `src/lib.rs`: Crate wiring and re-exports.
- `src/types.rs`: Public types (`Action`, `Cursor`, `Viewport`).
- `src/buffer.rs`: Line-based text storage.
- `src/editor.rs`: Editing engine and cursor/viewport behavior.
- `src/text.rs`: UTF-8 helper routines for character-safe slicing/indexing.
