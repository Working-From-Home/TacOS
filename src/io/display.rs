use crate::io::{cursor, console};
use crate::drivers::vga::{draw_char_at, DEFAULT_COLOR};

/// Prints a character to the VGA buffer at 0xb8000 with the default color.
pub fn write_char(c: u8) {
    let (x, y) = cursor::get_pos();
    draw_char_at(x, y, c, DEFAULT_COLOR);
}

/// Prints a character to the VGA buffer at 0xb8000 at a specific position.
pub fn write_char_at(c: u8, x: usize, y: usize) {
    draw_char_at(x + console::PROMPT_LEN, y, c, DEFAULT_COLOR);
}

/// Prints a character to the VGA buffer at 0xb8000 with a specific color.
pub fn write_colored_char(c: u8, color: u8) {
    let (x, y) = cursor::get_pos();
    draw_char_at(x, y, c, color);
}

/// Prints a character to the VGA buffer at 0xb8000 with a specific color.
pub fn write_colored_char_at(c: u8, color: u8, x: usize, y: usize) {
    draw_char_at(x, y, c, color);
}

/// Refreshes the display after an insert operation.
pub fn refresh_after_insert(buffer: &[u8], len: usize, start_pos: usize, cursor_y: usize) {
    for i in start_pos..len.min(buffer.len()) {
        let c = buffer[i];
        write_char_at(c, i, cursor_y);
    }
}

/// Refreshes the display after a delete operation.
pub fn refresh_after_delete(buffer: &[u8], len: usize, start_pos: usize, cursor_y: usize) {
    for i in start_pos..len.min(buffer.len()) {
        let c = buffer[i];
        write_char_at(c, i, cursor_y);
    }
    write_char_at(b' ', len, cursor_y);
}