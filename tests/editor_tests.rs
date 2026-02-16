use edit_core::{Action, Buffer, Cursor, Editor, Viewport};

#[test]
fn buffer_from_text_roundtrip() {
    // Init
    let buffer = Buffer::from_text("hello\nworld\n");

    // Assert
    assert_eq!(buffer.line_count(), 3);
    assert_eq!(buffer.line(2), Some(""));
    assert_eq!(buffer.as_text(), "hello\nworld\n");

    // Init
    let empty = Buffer::from_text("");

    // Assert
    assert_eq!(empty.line_count(), 1);
    assert_eq!(empty.as_text(), "");
}

#[test]
fn editor_insert_newline_delete_join() {
    // Init
    let buffer = Buffer::from_text("hi");
    let mut editor = Editor::new(buffer, Viewport::new(5, 10));

    // Act
    editor.apply(Action::MoveRight);
    editor.apply(Action::MoveRight);
    editor.apply(Action::Insert('!'));

    // Assert
    assert_eq!(editor.buffer().as_text(), "hi!");

    // Act
    editor.apply(Action::Newline);
    editor.apply(Action::Insert('a'));

    // Assert
    assert_eq!(editor.buffer().as_text(), "hi!\na");

    // Act
    editor.apply(Action::DeleteBackward);
    editor.apply(Action::DeleteBackward);

    // Assert
    assert_eq!(editor.buffer().as_text(), "hi!");
}

#[test]
fn editor_move_across_lines() {
    // Init
    let buffer = Buffer::from_text("ab\ncd");
    let mut editor = Editor::new(buffer, Viewport::new(5, 10));

    // Act
    editor.apply(Action::MoveRight);
    editor.apply(Action::MoveRight);
    editor.apply(Action::MoveRight);

    // Assert
    assert_eq!(editor.cursor(), Cursor::new(1, 0));

    // Act
    editor.apply(Action::MoveLeft);

    // Assert
    assert_eq!(editor.cursor(), Cursor::new(0, 2));
}

#[test]
fn delete_forward_joins_lines() {
    // Init
    let buffer = Buffer::from_text("ab\ncd");
    let mut editor = Editor::new(buffer, Viewport::new(5, 10));

    // Act
    editor.apply(Action::MoveRight);
    editor.apply(Action::MoveRight);
    editor.apply(Action::DeleteForward);

    // Assert
    assert_eq!(editor.buffer().as_text(), "abcd");
    assert_eq!(editor.cursor(), Cursor::new(0, 2));
}

#[test]
fn horizontal_scrolling_keeps_cursor_visible() {
    // Init
    let buffer = Buffer::from_text("abcdef");
    let mut editor = Editor::new(buffer, Viewport::new(1, 3));

    // Act
    for _ in 0..4 {
        editor.apply(Action::MoveRight);
    }

    // Assert
    assert_eq!(editor.viewport().col_offset, 2);
    assert_eq!(editor.cursor_screen_pos(), (0, 2));
    assert_eq!(editor.visible_lines(), vec!["cde".to_string()]);
}

#[test]
fn unicode_editing_uses_char_indices() {
    // Init
    let buffer = Buffer::from_text("aé");
    let mut editor = Editor::new(buffer, Viewport::new(1, 10));

    // Act
    editor.apply(Action::MoveRight);
    editor.apply(Action::MoveRight);
    editor.apply(Action::DeleteBackward);

    // Assert
    assert_eq!(editor.buffer().as_text(), "a");
}

#[test]
fn dirty_flag_only_tracks_mutations() {
    // Init
    let buffer = Buffer::from_text("a");
    let mut editor = Editor::new(buffer, Viewport::new(1, 1));

    // Assert
    assert!(!editor.is_dirty());

    // Act
    editor.apply(Action::MoveRight);

    // Assert
    assert!(!editor.is_dirty());

    // Act
    editor.apply(Action::Insert('b'));

    // Assert
    assert!(editor.is_dirty());
    editor.reset_dirty();

    // Assert
    assert!(!editor.is_dirty());
}

#[test]
fn move_word_left_treats_punctuation_as_token() {
    // Init
    let buffer = Buffer::from_text("foo(); bar");
    let mut editor = Editor::new(buffer, Viewport::new(1, 20));

    // Move to end of line.
    for _ in 0..10 {
        editor.apply(Action::MoveRight);
    }
    assert_eq!(editor.cursor(), Cursor::new(0, 10));

    // Act + Assert
    editor.apply(Action::MoveWordLeft);
    assert_eq!(editor.cursor(), Cursor::new(0, 7)); // before "bar"

    editor.apply(Action::MoveWordLeft);
    assert_eq!(editor.cursor(), Cursor::new(0, 5)); // before ';'

    editor.apply(Action::MoveWordLeft);
    assert_eq!(editor.cursor(), Cursor::new(0, 4)); // before ')'

    editor.apply(Action::MoveWordLeft);
    assert_eq!(editor.cursor(), Cursor::new(0, 3)); // before '('

    editor.apply(Action::MoveWordLeft);
    assert_eq!(editor.cursor(), Cursor::new(0, 0)); // before "foo"
}

#[test]
fn move_word_right_treats_punctuation_as_token() {
    // Init
    let buffer = Buffer::from_text("foo(); bar");
    let mut editor = Editor::new(buffer, Viewport::new(1, 20));

    // Act + Assert
    editor.apply(Action::MoveWordRight);
    assert_eq!(editor.cursor(), Cursor::new(0, 3)); // after "foo"

    editor.apply(Action::MoveWordRight);
    assert_eq!(editor.cursor(), Cursor::new(0, 4)); // after '('

    editor.apply(Action::MoveWordRight);
    assert_eq!(editor.cursor(), Cursor::new(0, 5)); // after ')'

    editor.apply(Action::MoveWordRight);
    assert_eq!(editor.cursor(), Cursor::new(0, 6)); // after ';'

    editor.apply(Action::MoveWordRight);
    assert_eq!(editor.cursor(), Cursor::new(0, 10)); // after "bar"
}

#[test]
fn move_word_crosses_lines_consistently() {
    // Init
    let buffer = Buffer::from_text("foo\n   ;bar");
    let mut editor = Editor::new(buffer, Viewport::new(2, 20));

    // Move to start of second line.
    for _ in 0..4 {
        editor.apply(Action::MoveRight);
    }
    assert_eq!(editor.cursor(), Cursor::new(1, 0));

    // Left jump from start of line should cross to previous line's word start.
    editor.apply(Action::MoveWordLeft);
    assert_eq!(editor.cursor(), Cursor::new(0, 0));

    // Move to end of first line and jump right across newline+spaces onto punctuation.
    for _ in 0..3 {
        editor.apply(Action::MoveRight);
    }
    assert_eq!(editor.cursor(), Cursor::new(0, 3));

    editor.apply(Action::MoveWordRight);
    assert_eq!(editor.cursor(), Cursor::new(1, 4)); // after ';'

    editor.apply(Action::MoveWordRight);
    assert_eq!(editor.cursor(), Cursor::new(1, 7)); // after "bar"
}
