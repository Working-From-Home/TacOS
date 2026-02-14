/// VGA display layer.
///
/// Three abstraction levels:
///   - `put_char_at` — write at fixed position, no cursor move
///   - `put_char`    — write at cursor, advance
///   - `put_str`     — interpret control characters

use crate::drivers::vga;

/// Number of spaces per tab stop.
pub const TAB_SIZE: usize = 8;

// ──────────────────────────────────────────────
//  Cursor state
// ──────────────────────────────────────────────

static mut CURSOR_X: usize = 0;
static mut CURSOR_Y: usize = 0;

/// Returns the current cursor position as `(x, y)`.
pub fn get_pos() -> (usize, usize) {
    unsafe { (CURSOR_X, CURSOR_Y) }
}

/// Sets the cursor position and syncs hardware.
pub fn set_pos(x: usize, y: usize) {
    unsafe {
        CURSOR_X = x;
        CURSOR_Y = y;
        vga::update_cursor(CURSOR_X, CURSOR_Y);
    }
}

/// Moves cursor left, wrapping to previous line.
pub fn move_left() {
    unsafe {
        if CURSOR_X > 0 {
            CURSOR_X -= 1;
        } else if CURSOR_Y > 0 {
            CURSOR_Y -= 1;
            CURSOR_X = vga::VGA_WIDTH - 1;
        }
    }
    sync_cursor();
}

/// Moves cursor right, wrapping and scrolling.
pub fn move_right() {
    unsafe {
        CURSOR_X += 1;
        if CURSOR_X >= vga::VGA_WIDTH {
            CURSOR_X = 0;
            if CURSOR_Y + 1 >= vga::VGA_HEIGHT {
                vga::scroll_buffer_up();
            } else {
                CURSOR_Y += 1;
            }
        }
    }
    sync_cursor();
}

/// Advances cursor to next line, scrolling if needed.
pub fn new_line() {
    unsafe {
        CURSOR_X = 0;
        if CURSOR_Y + 1 >= vga::VGA_HEIGHT {
            vga::scroll_buffer_up();
        } else {
            CURSOR_Y += 1;
        }
    }
    sync_cursor();
}

// Syncs software cursor with VGA hardware.
fn sync_cursor() {
    let (x, y) = get_pos();
    vga::update_cursor(x, y);
}

// ──────────────────────────────────────────────
//  Character output — no control-char handling
// ──────────────────────────────────────────────

/// Writes a byte at the cursor and advances. Default color.
#[inline]
pub fn put_char(c: u8) {
    let (x, y) = get_pos();
    put_char_at(x, y, c);
    move_right();
}

/// Writes a byte at the cursor and advances. Custom color.
#[inline]
pub fn put_char_colored(c: u8, color: u8) {
    let (x, y) = get_pos();
    put_char_at_colored(x, y, c, color);
    move_right();
}

/// Writes a byte at `(x, y)`. No cursor move. Default color.
#[inline]
pub fn put_char_at(x: usize, y: usize, c: u8) {
    vga::draw_char_at(x, y, c, vga::DEFAULT_COLOR);
}

/// Writes a byte at `(x, y)`. No cursor move. Custom color.
#[inline]
pub fn put_char_at_colored(x: usize, y: usize, c: u8, color: u8) {
    vga::draw_char_at(x, y, c, color);
}

// ──────────────────────────────────────────────
//  String output — with control-char handling
//
//  Interprets: \n, \t, \x08 (backspace),
//              \x07 (bell), \x0B (vertical tab)
// ──────────────────────────────────────────────

/// Writes a string, interpreting control characters.
pub fn put_str(s: &str) {
    put_str_colored(s, vga::DEFAULT_COLOR);
}

/// Writes a byte slice, interpreting control characters.
pub fn put_bytes(bytes: &[u8]) {
    for &b in bytes {
        write_byte(b, vga::DEFAULT_COLOR);
    }
}

/// Writes a string with custom color, interpreting controls.
pub fn put_str_colored(s: &str, color: u8) {
    for &b in s.as_bytes() {
        write_byte(b, color);
    }
}

/// Dispatches a byte: handles control chars, forwards
/// printable characters to `put_char_colored`.
pub fn write_byte(c: u8, color: u8) {
    match c {
        0x07 => {}                                          // bell (ignored)
        b'\n' => new_line(),                                // new_line
        0x08 => move_left(),                                // backspace
        0x0B => {                                           // vertical tab
            let (x, y) = get_pos();
            if y + 1 >= vga::VGA_HEIGHT {
                vga::scroll_buffer_up();
                set_pos(x, y);
            } else {
                set_pos(x, y + 1);
            }
        }
        b'\t' => {                                          // tab
            let (x, _) = get_pos();
            let spaces = TAB_SIZE - (x % TAB_SIZE);
            for _ in 0..spaces {
                put_char_colored(b' ', color);
            }
        }
        _ => put_char_colored(c, color),                    // printable
    }
}
