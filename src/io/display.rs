use crate::drivers::vga::{draw_char_at, DEFAULT_COLOR, VGA_HEIGHT, VGA_WIDTH, scroll_buffer_up, update_cursor};

/// Number of spaces per tab stop.
pub const TAB_SIZE: usize = 8;

// Cursor state
static mut CURSOR_X: usize = 0;
static mut CURSOR_Y: usize = 0;

/// Returns the current cursor position as a tuple (x, y).
pub fn get_pos() -> (usize, usize) {
    unsafe { (CURSOR_X, CURSOR_Y) }
}

/// Sets the cursor position to the specified coordinates.
pub fn set_pos(x: usize, y: usize) {
    unsafe {
        CURSOR_X = x;
        CURSOR_Y = y;
        update_cursor(CURSOR_X, CURSOR_Y);
    }
}

/// Moves the cursor to the left. If it reaches the beginning of the line, it wraps to the previous line.
pub fn move_left() {
    unsafe {
        if CURSOR_X > 0 {
            CURSOR_X -= 1;
        } else if CURSOR_Y > 0 {
            CURSOR_Y -= 1;
            CURSOR_X = VGA_WIDTH - 1;
        }
    }
    sync_to_hardware();
}

/// Moves the cursor to the right. If it reaches the end of the line, it wraps to the next line.
pub fn move_right() {
    unsafe {
        CURSOR_X += 1;
        if CURSOR_X >= VGA_WIDTH {
            CURSOR_X = 0;
            if CURSOR_Y + 1 >= VGA_HEIGHT {
                scroll_buffer_up();
            } else {
                CURSOR_Y += 1;
            }
        }
    }
    sync_to_hardware();
}

/// Moves the cursor to the beginning of the next line.
pub fn new_line() {
    unsafe {
        CURSOR_X = 0;
        if CURSOR_Y + 1 >= VGA_HEIGHT {
            scroll_buffer_up();
        } else {
            CURSOR_Y += 1;
        }
    }
    sync_to_hardware();
}

/// Synchronizes the cursor position with the hardware.
fn sync_to_hardware() {
    let (x, y) = get_pos();
    update_cursor(x, y);
}

/// Prints a character to the VGA buffer at 0xb8000 with the default color.
pub fn write_char(c: u8) {
    let (x, y) = get_pos();
    draw_char_at(x, y, c, DEFAULT_COLOR);
}

/// Prints a character to the VGA buffer at 0xb8000 at a specific position.
pub fn write_char_at(c: u8, x: usize, y: usize, input_offset: usize) {
    draw_char_at(x + input_offset, y, c, DEFAULT_COLOR);
}

/// Prints a character to the VGA buffer at 0xb8000 with a specific color.
pub fn write_colored_char(c: u8, color: u8) {
    let (x, y) = get_pos();
    draw_char_at(x, y, c, color);
}

/// Prints a character to the VGA buffer at 0xb8000 with a specific color.
pub fn write_colored_char_at(c: u8, color: u8, x: usize, y: usize) {
    draw_char_at(x, y, c, color);
}

/// Prints a line of text from the buffer to the VGA buffer at 0xb8000.
pub fn write_buffer_line(buffer: &[u8], len: usize, start_pos: usize, cursor_y: usize, clear_tail_len: usize, input_offset: usize) {
    for i in start_pos..len.min(buffer.len()) {
        let c = buffer[i];
        write_char_at(c, i, cursor_y, input_offset);
    }
    for i in 0..clear_tail_len {
        write_char_at(b' ', len + i, cursor_y, input_offset);
    }
}

/// Writes a single byte to the screen at the current cursor position,
/// advancing the cursor. Interprets \n as newline and \t as a tab-aligned space.
pub fn put_char(c: u8) {
    match c {
        0x07 => {} // bell: no visible output
        b'\n' => new_line(),
        0x0B => {
            // vertical tab: move cursor down, keep column
            let (x, y) = get_pos();
            if y + 1 >= VGA_HEIGHT {
                scroll_buffer_up();
                set_pos(x, y);
            } else {
                set_pos(x, y + 1);
            }
        }
        b'\t' => {
            let (x, _) = get_pos();
            let spaces = TAB_SIZE - (x % TAB_SIZE);
            let mut s = 0;
            while s < spaces {
                write_colored_char(b' ', DEFAULT_COLOR);
                move_right();
                s += 1;
            }
        }
        _ => {
            write_colored_char(c, DEFAULT_COLOR);
            move_right();
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
