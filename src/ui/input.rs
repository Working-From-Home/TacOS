use crate::ui::cursor;
use crate::ui::display;

const INPUT_BUFFER_SIZE: usize = 256;
static mut INPUT_BUFFER: [u8; INPUT_BUFFER_SIZE] = [0; INPUT_BUFFER_SIZE];
static mut INPUT_LEN: usize = 0;
static mut CURSOR_POS: usize = 0;

/// Inserts a character at the current cursor position in the input buffer.
pub fn insert_char(c: u8) {
    unsafe {
        if INPUT_LEN < INPUT_BUFFER_SIZE - 1 && CURSOR_POS < INPUT_BUFFER_SIZE - 1 {
            for i in (CURSOR_POS..INPUT_LEN).rev() {
                INPUT_BUFFER[i + 1] = INPUT_BUFFER[i];
            }

            INPUT_BUFFER[CURSOR_POS] = c;
            CURSOR_POS += 1;
            INPUT_LEN += 1;

            display::write_char(c);
            cursor::move_right();
        }
    }
}

/// Removes the character at the current cursor position in the input buffer.
pub fn remove_char() {
    unsafe {
        if CURSOR_POS == 0 || INPUT_LEN == 0 {
            return;
        }
        if CURSOR_POS > INPUT_LEN || CURSOR_POS > INPUT_BUFFER_SIZE || INPUT_LEN > INPUT_BUFFER_SIZE {
            return;
        }

        CURSOR_POS -= 1;

        // Move the rest of the buffer to the left
        let mut i = CURSOR_POS;
        while i + 1 < INPUT_LEN {
            INPUT_BUFFER[i] = INPUT_BUFFER[i + 1];
            i += 1;
        }
        if INPUT_LEN > 0 {
            INPUT_BUFFER[INPUT_LEN - 1] = 0;
            INPUT_LEN -= 1;
        }

        cursor::move_left();
        display::write_char(b' ');
    }
}

pub fn clear() {
    unsafe {
        for i in 0..INPUT_BUFFER_SIZE {
            INPUT_BUFFER[i] = 0;
        }
        CURSOR_POS = 0;
        INPUT_LEN = 0;
    }
}

/// For testing purposes
pub fn as_ptr() -> *const u8 {
    unsafe { INPUT_BUFFER.as_ptr() }
}