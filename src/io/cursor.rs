use crate::drivers::vga;

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
        vga::update_cursor(CURSOR_X, CURSOR_Y);
    }
}

/// Moves the cursor to the left. If it reaches the beginning of the line, it wraps to the previous line.
pub fn move_left() {
    unsafe {
        if CURSOR_X > 0 {
            CURSOR_X -= 1;
        } else if CURSOR_Y > 0 {
            CURSOR_Y -= 1;
            CURSOR_X = vga::VGA_WIDTH - 1;
        }
    }
    sync_to_hardware();
}

/// Moves the cursor to the right. If it reaches the end of the line, it wraps to the next line.
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
    sync_to_hardware();
}

/// Moves the cursor to the beginning of the next line.
pub fn new_line() {
    unsafe {
        CURSOR_X = 0;
        if CURSOR_Y + 1 >= vga::VGA_HEIGHT {
            vga::scroll_buffer_up();
        } else {
            CURSOR_Y += 1;
        }
    }
    sync_to_hardware();
}

/// Synchronizes the cursor position with the hardware.
fn sync_to_hardware() {
    let (x, y) = get_pos();
    vga::update_cursor(x, y);
}
