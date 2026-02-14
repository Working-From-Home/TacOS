/// Line-editing input buffer.
///
/// Fixed-size buffer with cursor position, supporting
/// insert, delete, and cursor movement.

const BUFFER_SIZE: usize = 78;

pub struct InputBuffer {
    pub buffer: [u8; BUFFER_SIZE],
    pub len: usize,
    pub pos: usize,
}

impl InputBuffer {
    pub const fn new() -> Self {
        InputBuffer {
            buffer: [0; BUFFER_SIZE],
            len: 0,
            pos: 0,
        }
    }

    /// Inserts `c` at cursor. Returns `false` if full.
    pub fn insert_char(&mut self, c: u8, max_len: usize) -> bool {
        let max = max_len.min(BUFFER_SIZE - 1);
        if self.len >= max || self.pos >= max {
            return false;
        }

        // Shift the buffer to the right from the end to the current position
        let mut i = self.len;
        while i > self.pos {
            unsafe {
                *self.buffer.get_unchecked_mut(i) = *self.buffer.get_unchecked(i - 1);
            }
            i -= 1;
        }

        unsafe { *self.buffer.get_unchecked_mut(self.pos) = c; }
        self.pos += 1;
        self.len += 1;

        true
    }

    /// Removes the character before the cursor (backspace).
    pub fn remove_char(&mut self) -> bool {
        if self.pos == 0 || self.len == 0 {
            return false;
        }

        self.pos -= 1;

        // Shift the rest of the buffer to the left
        let mut i = self.pos;
        while i + 1 < self.len {
            unsafe {
                *self.buffer.get_unchecked_mut(i) = *self.buffer.get_unchecked(i + 1);
            }
            i += 1;
        }

        // Clear the last character
        unsafe { *self.buffer.get_unchecked_mut(self.len - 1) = 0; }
        self.len -= 1;

        true
    }

    /// Returns `true` if cursor can move left.
    pub fn can_move_left(&self) -> bool {
        self.pos > 0
    }

    /// Returns `true` if cursor can move right.
    pub fn can_move_right(&self) -> bool {
        self.pos < self.len
    }

    /// Moves cursor one position left.
    pub fn move_left(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }
    }

    /// Moves cursor one position right.
    pub fn move_right(&mut self) {
        if self.pos < self.len {
            self.pos += 1;
        }
    }

    /// Returns the buffer content and resets state.
    pub fn flush(&mut self) -> &[u8] {
        let len = self.len.min(BUFFER_SIZE);
        let slice = unsafe { self.buffer.get_unchecked(..len) };
        self.len = 0;
        self.pos = 0;
        slice
    }

    /// Returns the active portion of the buffer.
    pub fn get_buffer(&self) -> &[u8] {
        unsafe { self.buffer.get_unchecked(..self.len.min(BUFFER_SIZE)) }
    }

    /// Returns the number of characters in the buffer.
    pub fn get_len(&self) -> usize {
        self.len
    }

    /// Returns the current cursor position.
    pub fn get_pos(&self) -> usize {
        self.pos
    }
}

// ──────────────────────────────────────────────
//  Global instance + free-function wrappers
// ──────────────────────────────────────────────

static mut INPUT: InputBuffer = InputBuffer::new();

/// Inserts `c` at cursor.
pub fn insert_char(c: u8, max_len: usize) -> bool {
    unsafe { (*&raw mut INPUT).insert_char(c, max_len) }
}

/// Removes the character before the cursor.
pub fn remove_char() -> bool {
    unsafe { (*&raw mut INPUT).remove_char() }
}

/// Returns `true` if cursor can move left.
pub fn can_move_left() -> bool {
    unsafe { (*&raw const INPUT).can_move_left() }
}

/// Returns `true` if cursor can move right.
pub fn can_move_right() -> bool {
    unsafe { (*&raw const INPUT).can_move_right() }
}

/// Moves cursor one position left.
pub fn move_left() {
    unsafe { (*&raw mut INPUT).move_left(); }
}

/// Moves cursor one position right.
pub fn move_right() {
    unsafe { (*&raw mut INPUT).move_right(); }
}

/// Returns the buffer content and resets state.
pub fn flush() -> &'static [u8] {
    unsafe { (*&raw mut INPUT).flush() }
}

/// Returns the active portion of the buffer.
pub fn get_buffer() -> &'static [u8] {
    unsafe { (*&raw const INPUT).get_buffer() }
}

/// Returns the number of characters in the buffer.
pub fn get_len() -> usize {
    unsafe { (*&raw const INPUT).get_len() }
}

/// Returns the current cursor position.
pub fn get_pos() -> usize {
    unsafe { (*&raw const INPUT).get_pos() }
}