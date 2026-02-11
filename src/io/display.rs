use crate::drivers::vga::{draw_char_at, DEFAULT_COLOR, VGA_HEIGHT, VGA_WIDTH, scroll_buffer_up, update_cursor};

/// Number of spaces per tab stop.
pub const TAB_SIZE: usize = 8;

// ──────────────────────────────────────────────
//  Cursor state
// ──────────────────────────────────────────────

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

/// Moves the cursor one position to the left, wrapping to the previous line if needed.
pub fn move_left() {
    unsafe {
        if CURSOR_X > 0 {
            CURSOR_X -= 1;
        } else if CURSOR_Y > 0 {
            CURSOR_Y -= 1;
            CURSOR_X = VGA_WIDTH - 1;
        }
    }
    sync_cursor();
}

/// Moves the cursor one position to the right, wrapping/scrolling if needed.
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
    sync_cursor();
}

/// Moves the cursor to the beginning of the next line, scrolling if needed.
pub fn new_line() {
    unsafe {
        CURSOR_X = 0;
        if CURSOR_Y + 1 >= VGA_HEIGHT {
            scroll_buffer_up();
        } else {
            CURSOR_Y += 1;
        }
    }
    sync_cursor();
}

/// Synchronizes the software cursor position with the VGA hardware cursor.
fn sync_cursor() {
    let (x, y) = get_pos();
    update_cursor(x, y);
}

// ──────────────────────────────────────────────
//  Low-level drawing (no cursor movement)
//
//  These paint pixels to VGA memory without touching the cursor.
//  Only used internally and by io_manager for input-line redraw.
// ──────────────────────────────────────────────

/// Draws a character at a fixed VGA position (no cursor movement).
/// Used internally by `redraw_input_line` for in-place input editing.
fn draw_at(c: u8, x: usize, y: usize) {
    draw_char_at(x, y, c, DEFAULT_COLOR);
}

// ──────────────────────────────────────────────
//  High-level write API (cursor-advancing)
//
//  These are the primary functions the rest of the kernel should use
//  to write text to the screen. They draw AND advance the cursor.
// ──────────────────────────────────────────────

/// Writes a single byte to the screen, advancing the cursor.
/// Interprets `\n`, `\t`, vertical tab (0x0B), and bell (0x07).
pub fn put_char(c: u8) {
    put_colored_char(c, DEFAULT_COLOR);
}

/// Writes a single byte to the screen with a specific color, advancing the cursor.
/// Interprets `\n`, `\t`, vertical tab (0x0B), and bell (0x07).
pub fn put_colored_char(c: u8, color: u8) {
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
                let (cx, cy) = get_pos();
                draw_char_at(cx, cy, b' ', color);
                move_right();
                s += 1;
            }
        }
        _ => {
            let (x, y) = get_pos();
            draw_char_at(x, y, c, color);
            move_right();
        }
    }
}

/// Writes a string slice to the screen, advancing the cursor.
pub fn put_str(s: &str) {
    for &b in s.as_bytes() {
        put_char(b);
    }
}

/// Writes a byte slice to the screen, advancing the cursor.
pub fn put_bytes(bytes: &[u8]) {
    for &b in bytes {
        put_char(b);
    }
}

/// Writes a string slice to the screen with a specific color, advancing the cursor.
pub fn put_colored_str(s: &str, color: u8) {
    for &b in s.as_bytes() {
        put_colored_char(b, color);
    }
}

// ──────────────────────────────────────────────
//  Input-line redraw (used by io_manager)
//
//  Redraws part of the input buffer at a fixed row without
//  moving the logical cursor. This is needed for insert/delete
//  in the middle of the input line.
// ──────────────────────────────────────────────

/// Redraws (part of) the input buffer on a single VGA row without moving the cursor.
/// `start_pos`..`len` are redrawn; `clear_tail_len` trailing cells are blanked.
pub fn redraw_input_line(buffer: &[u8], len: usize, start_pos: usize, cursor_y: usize, clear_tail_len: usize, input_offset: usize) {
    for i in start_pos..len.min(buffer.len()) {
        draw_at(buffer[i], i + input_offset, cursor_y);
    }
    for i in 0..clear_tail_len {
        draw_at(b' ', len + i + input_offset, cursor_y);
    }
}
