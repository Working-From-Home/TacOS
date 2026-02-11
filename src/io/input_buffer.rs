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

    /// Inserts a character at the current position in the input buffer.
    pub fn insert_char(&mut self, c: u8) -> bool {
        let max = crate::io::console::max_input_len().min(BUFFER_SIZE - 1);
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

    /// Removes the character at the current position in the input buffer.
    pub fn remove_char(&mut self) -> bool {
        if self.pos == 0 || self.len == 0 {
            return false;
        }
        if self.pos > self.len || self.pos > BUFFER_SIZE || self.len > BUFFER_SIZE {
            return false;
        }

        self.pos -= 1;

        // Move the rest of the buffer to the left
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

    pub fn can_move_left(&self) -> bool {
        self.pos > 0
    }

    pub fn can_move_right(&self) -> bool {
        self.pos < self.len
    }

    pub fn move_left(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.pos < self.len {
            self.pos += 1;
        }
    }

    pub fn flush(&mut self) -> &[u8] {
        let len = if self.len > BUFFER_SIZE { BUFFER_SIZE } else { self.len };
        let slice = &self.buffer[..len];
        self.len = 0;
        self.pos = 0;
        slice
    }

    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer[..self.len.min(self.buffer.len())]
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn get_pos(&self) -> usize {
        self.pos
    }
}

pub static mut INPUT: InputBuffer = InputBuffer::new();

pub fn insert_char(c: u8) -> bool{
    unsafe {
        INPUT.insert_char(c)
    }
}

pub fn remove_char() -> bool{
    unsafe {
        INPUT.remove_char()
    }
}

pub fn can_move_left() -> bool {
    unsafe {
        INPUT.can_move_left()
    }
}

pub fn can_move_right() -> bool {
    unsafe {
        INPUT.can_move_right()
    }
}

pub fn move_left() {
    unsafe {
        INPUT.move_left();
    }
}

pub fn move_right() {
    unsafe {
        INPUT.move_right();
    }
}

pub fn flush() -> &'static [u8] {
    unsafe {
        INPUT.flush()
    }
}

pub fn get_buffer() -> &'static [u8] {
    unsafe {
        INPUT.get_buffer()
    }
}

pub fn get_len() -> usize {
    unsafe {
        INPUT.get_len()
    }
}

pub fn get_pos() -> usize {
    unsafe {
        INPUT.get_pos()
    }
}