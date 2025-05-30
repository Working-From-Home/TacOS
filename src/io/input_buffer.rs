const BUFFER_SIZE: usize = 128;

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
        if self.len >= BUFFER_SIZE - 1 || self.pos >= BUFFER_SIZE - 1 {
            return false;
        }

        // Shift the buffer to the right from the end to the current position
        // for i in (self.pos..self.len).rev() {
        //     self.buffer[i + 1] = self.buffer[i];
        // }
        let mut i = self.len;
        while i > self.pos {
            unsafe {
                *self.buffer.get_unchecked_mut(i) = *self.buffer.get_unchecked(i - 1);
            }
            i -= 1;
        }
        
        self.buffer[self.pos] = c;
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
        // let mut i = self.pos;
        // while i + 1 < self.len {
        //     self.buffer[i] = self.buffer[i + 1];
        //     i += 1;
        // }
        let mut i = self.pos;
        while i + 1 < self.len {
            unsafe {
                *self.buffer.get_unchecked_mut(i) = *self.buffer.get_unchecked(i + 1);
            }
            i += 1;
        }

        // Clear the last character
        self.buffer[self.len - 1] = 0;

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


// static mut INPUT_BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
// static mut INPUT_LEN: usize = 0;

// Buffer pour la commande validée (Enter pressé)
// static mut CMD_BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
// static mut CMD_LEN: usize = 0;

// static mut CURSOR_POS: usize = 0;

// /// Inserts a character at the current cursor position in the input buffer.
// pub fn insert_char(c: u8) -> bool {
//     unsafe {
//         if INPUT_LEN >= BUFFER_SIZE - 1 || CURSOR_POS >= BUFFER_SIZE - 1 {
//             return false;
//         }

//         // Shift the buffer to the right from the end to the cursor position
//         for i in (CURSOR_POS..INPUT_LEN).rev() {
//             INPUT_BUFFER[i + 1] = INPUT_BUFFER[i];
//         }
        
//         // Insert the character in the input buffer at the cursor position
//         INPUT_BUFFER[CURSOR_POS] = c;

//         // Increment the cursor position and input length
//         CURSOR_POS += 1;
//         INPUT_LEN += 1;

//         // Move the cursor to the beginning of the line
//         // for _ in 0..CURSOR_POS.saturating_sub(1) {
//         //     cursor::move_left();
//         // }
//         // NOT WORKING HERE

//         // Display the entire input buffer
//         // for i in 0..INPUT_LEN {
//         //     display::write_char(INPUT_BUFFER[i]);
//         // }

//         // Clear the residual character
//         // display::write_char(b' ');

//         // Move the cursor after the inserted character
//         // let move_left_count = INPUT_LEN - CURSOR_POS;
//         // for _ in 0..move_left_count + 1 {
//         //     cursor::move_left();
//         // }
//         // cursor::move_right();
//     }
//     true
// }

// /// Removes the character at the current cursor position in the input buffer.
// pub fn remove_char() -> bool {
//     unsafe {
//         if CURSOR_POS == 0 || INPUT_LEN == 0 {
//             return false;
//         }
//         if CURSOR_POS > INPUT_LEN || CURSOR_POS > BUFFER_SIZE || INPUT_LEN > BUFFER_SIZE {
//             return false;
//         }

//         CURSOR_POS -= 1;

//         // Move the rest of the buffer to the left
//         let mut i = CURSOR_POS;
//         while i + 1 < INPUT_LEN {
//             INPUT_BUFFER[i] = INPUT_BUFFER[i + 1];
//             i += 1;
//         }
//         if INPUT_LEN > 0 {
//             INPUT_BUFFER[INPUT_LEN - 1] = 0;
//             INPUT_LEN -= 1;
//         }

//         // cursor::move_left();
//         // display::write_char(b' ');
//     }
//     true
// }

// pub fn clear_input_buffer() {
//     unsafe {
//         for i in 0..BUFFER_SIZE {
//             INPUT_BUFFER[i] = 0;
//         }
//         CURSOR_POS = 0;
//         INPUT_LEN = 0;
//     }
// }

// fn clear_cmd_buffer() {
//     unsafe {
//         for i in 0..BUFFER_SIZE {
//             CMD_BUFFER[i] = 0;
//         }
//         CMD_LEN = 0;
//     }
// }

// pub fn handle_left() {
//     unsafe {
//         if CURSOR_POS > 0 {
//             CURSOR_POS -= 1;
//             cursor::move_left();
//         }
//     }
// }

// pub fn handle_right() {
//     unsafe {
//         if CURSOR_POS < INPUT_LEN {
//             CURSOR_POS += 1;
//             cursor::move_right();
//         }
//     }
// }

// pub fn handle_enter() -> *const u8{
//     cursor::new_line();
//     clear_cmd_buffer();
//     unsafe {
//         // Copies the input buffer to the command buffer
//         let mut i = 0;
//         while i < INPUT_LEN && i < BUFFER_SIZE {
//             CMD_BUFFER[i] = INPUT_BUFFER[i];
//             i += 1;
//         }

//         CMD_LEN = INPUT_LEN;
//         clear_input_buffer();

//         CMD_BUFFER.as_ptr()
//     }
// }