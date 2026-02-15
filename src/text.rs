pub(crate) fn char_to_byte_index(text: &str, char_index: usize) -> usize {
    if char_index == 0 {
        return 0;
    }

    for (count, (byte_idx, _)) in text.char_indices().enumerate() {
        if count == char_index {
            return byte_idx;
        }
    }

    text.len()
}

pub(crate) fn slice_line(text: &str, start_col: usize, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let start = char_to_byte_index(text, start_col);
    let end = char_to_byte_index(text, start_col + width);
    if start >= end || start >= text.len() {
        return String::new();
    }

    text[start..end].to_string()
}
