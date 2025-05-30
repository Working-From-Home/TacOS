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

/// Prints a line of text from the buffer to the VGA buffer at 0xb8000.
pub fn write_buffer_line(buffer: &[u8], len: usize, start_pos: usize, cursor_y: usize, clear_tail_len: usize) {
    for i in start_pos..len.min(buffer.len()) {
        let c = buffer[i];
        write_char_at(c, i, cursor_y);
    }
    for i in 0..clear_tail_len {
        write_char_at(b' ', len + i, cursor_y);
    }
}