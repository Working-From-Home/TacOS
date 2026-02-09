use crate::io::{cursor, console};
use crate::drivers::vga::{draw_char_at, DEFAULT_COLOR};

/// Number of spaces per tab stop.
pub const TAB_SIZE: usize = 8;

/// Prints a character to the VGA buffer at 0xb8000 with the default color.
pub fn write_char(c: u8) {
    let (x, y) = cursor::get_pos();
    draw_char_at(x, y, c, DEFAULT_COLOR);
}

/// Prints a character to the VGA buffer at 0xb8000 at a specific position.
pub fn write_char_at(c: u8, x: usize, y: usize) {
    draw_char_at(x + console::input_start_col(), y, c, DEFAULT_COLOR);
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

/// Computes the visual column width of buffer[0..pos], accounting for tab stops
/// aligned to absolute screen columns (including the prompt offset).
pub fn visual_col(buffer: &[u8], pos: usize) -> usize {
    let offset = console::input_start_col();
    let mut abs_col: usize = offset;
    let end = pos.min(buffer.len());
    let mut i: usize = 0;
    while i < end {
        let c = unsafe { *buffer.as_ptr().add(i) };
        if c == b'\t' {
            abs_col += TAB_SIZE - (abs_col % TAB_SIZE);
        } else {
            abs_col += 1;
        }
        i += 1;
    }
    abs_col - offset
}

/// Prints a line of text from the buffer to the VGA buffer, handling tabs.
/// `clear_tail_vcols` is the number of extra visual columns to blank after the content.
pub fn write_buffer_line(buffer: &[u8], len: usize, start_pos: usize, cursor_y: usize, clear_tail_vcols: usize) {
    let end = len.min(buffer.len());
    // Compute starting visual column
    let mut vcol = visual_col(buffer, start_pos);
    let mut i = start_pos;
    while i < end {
        let c = unsafe { *buffer.as_ptr().add(i) };
        if c == b'\t' {
            let tab_end = vcol + (TAB_SIZE - (vcol % TAB_SIZE));
            while vcol < tab_end {
                write_char_at(b' ', vcol, cursor_y);
                vcol += 1;
            }
        } else {
            write_char_at(c, vcol, cursor_y);
            vcol += 1;
        }
        i += 1;
    }
    let mut j: usize = 0;
    while j < clear_tail_vcols {
        write_char_at(b' ', vcol + j, cursor_y);
        j += 1;
    }
}

/// Writes a single byte to the screen at the current cursor position,
/// advancing the cursor. Interprets \n as newline and \t as a tab-aligned space.
pub fn put_char(c: u8) {
    match c {
        0x07 => {} // bell: no visible output
        b'\n' => cursor::new_line(),
        0x0B => {
            // vertical tab: move cursor down, keep column
            let (x, y) = cursor::get_pos();
            if y + 1 >= crate::drivers::vga::VGA_HEIGHT {
                crate::drivers::vga::scroll_buffer_up();
                cursor::set_pos(x, y);
            } else {
                cursor::set_pos(x, y + 1);
            }
        }
        b'\t' => {
            let (x, _) = cursor::get_pos();
            let spaces = TAB_SIZE - (x % TAB_SIZE);
            let mut s = 0;
            while s < spaces {
                write_colored_char(b' ', DEFAULT_COLOR);
                cursor::move_right();
                s += 1;
            }
        }
        _ => {
            write_colored_char(c, DEFAULT_COLOR);
            cursor::move_right();
        }
    }
}

/// Writes a string to the screen at the current cursor position,
/// advancing the cursor. Interprets \n as newline.
pub fn put_str(s: &str) {
    let ptr = s.as_ptr();
    let len = s.len();
    let mut i: usize = 0;
    while i < len {
        unsafe {
            put_char(*ptr.add(i));
        }
        i += 1;
    }
}

/// Writes a byte slice to the screen at the current cursor position,
/// advancing the cursor. Interprets \n as newline.
pub fn put_bytes(bytes: &[u8]) {
    let ptr = bytes.as_ptr();
    let len = bytes.len();
    let mut i: usize = 0;
    while i < len {
        unsafe {
            put_char(*ptr.add(i));
        }
        i += 1;
    }
}
