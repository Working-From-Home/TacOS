use crate::drivers::port;

static mut SHIFT_PRESSED: bool = false;

pub fn get_char() -> Option<char> {
    if let Some(scancode) = read_scancode() {
        handle_scancode(scancode)
    } else {
        None
    }
}

/// Reads a scancode from the keyboard controller.
#[inline(always)]
fn read_scancode() -> Option<u8> {
    let status = port::inb(0x64);
    if status & 0x01 != 0 {
        Some(port::inb(0x60))
    } else {
        None
    }
}

fn handle_scancode(scancode: u8) -> Option<char> {
    match scancode {
        0x1C => Some('\n'), // Enter
        0x0E => Some('\x08'), // Backspace
        0x2A | 0x36 => { unsafe { SHIFT_PRESSED = true }; None },   // Shift press
        0xAA | 0xB6 => { unsafe { SHIFT_PRESSED = false }; None },  // Shift release
        _=> { scancode_to_ascii(scancode) }
    }
}

fn scancode_to_ascii(scancode: u8) -> Option<char> {
    const MAP: [Option<char>; 128] = {
        let mut m = [None; 128];
        m[0x02] = Some('1'); m[0x03] = Some('2'); m[0x04] = Some('3');
        m[0x05] = Some('4'); m[0x06] = Some('5'); m[0x07] = Some('6');
        m[0x08] = Some('7'); m[0x09] = Some('8'); m[0x0A] = Some('9');
        m[0x0B] = Some('0'); m[0x0C] = Some('-'); m[0x0D] = Some('=');
        m[0x10] = Some('q'); m[0x11] = Some('w'); m[0x12] = Some('e');
        m[0x13] = Some('r'); m[0x14] = Some('t'); m[0x15] = Some('y');
        m[0x16] = Some('u'); m[0x17] = Some('i'); m[0x18] = Some('o');
        m[0x19] = Some('p'); m[0x1E] = Some('a'); m[0x1F] = Some('s');
        m[0x20] = Some('d'); m[0x21] = Some('f'); m[0x22] = Some('g');
        m[0x23] = Some('h'); m[0x24] = Some('j'); m[0x25] = Some('k');
        m[0x26] = Some('l'); m[0x2C] = Some('z'); m[0x2D] = Some('x');
        m[0x2E] = Some('c'); m[0x2F] = Some('v'); m[0x30] = Some('b');
        m[0x31] = Some('n'); m[0x32] = Some('m'); m[0x33] = Some(',');
        m[0x34] = Some('.'); m[0x35] = Some('/'); m[0x39] = Some(' ');
        m
    };

    let base = MAP.get(scancode as usize).copied().flatten();
    unsafe {
        if SHIFT_PRESSED {
            base.map(shift_char)
        } else {
            base
        }
    }
}

fn shift_char(c: char) -> char {
    match c {
        '1' => '!', '2' => '@', '3' => '#', '4' => '$',
        '5' => '%', '6' => '^', '7' => '&', '8' => '*',
        '9' => '(', '0' => ')', '-' => '_', '=' => '+',
        '[' => '{', ']' => '}', '\\' => '|', ';' => ':',
        '\'' => '"', ',' => '<', '.' => '>', '/' => '?',
        '`' => '~',
        c => c.to_ascii_uppercase(),
    }
}