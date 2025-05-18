use crate::ui::cursor;
use crate::drivers::vga::{draw_char_at, DEFAULT_COLOR};

/// Prints a character to the VGA buffer at 0xb8000.
fn write_char_core(c: u8, color: u8) {
    let (x, y) = cursor::get_pos();
    draw_char_at(x, y, c, color);
}

/// Prints a character to the VGA buffer at 0xb8000 with the default color.
pub fn write_char(c: u8) {
    write_char_core(c, DEFAULT_COLOR);
}

/// Prints a character to the VGA buffer at 0xb8000 with a specific color.
pub fn write_colored_char(c: u8, color: u8) {
    write_char_core(c, color);
}