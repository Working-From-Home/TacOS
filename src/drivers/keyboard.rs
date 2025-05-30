use crate::drivers::port;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEvent {
    Char(char),
    Enter,
    Backspace,
    Tab,            // not implemented yet
    ArrowLeft,
    ArrowRight,
    ArrowUp,        // not implemented yet
    ArrowDown,      // not implemented yet
    Unknown,
}

static mut SHIFT_PRESSED: bool = false;

pub fn get_key_event() -> Option<KeyEvent> {
    if let Some(scancode) = read_scancode() {
        handle_scancode(scancode)
    } else {
        None
    }
}

#[inline(always)]
fn read_scancode() -> Option<u8> {
    let status = port::inb(0x64);
    if status & 0x01 != 0 {
        Some(port::inb(0x60))
    } else {
        None
    }
}

fn handle_scancode(scancode: u8) -> Option<KeyEvent> {
    match scancode {
        0x2A | 0x36 => { unsafe { SHIFT_PRESSED = true }; None },   // Shift press
        0xAA | 0xB6 => { unsafe { SHIFT_PRESSED = false }; None },  // Shift release
        _ => {
            let map: &[Option<KeyEvent>; 128] = unsafe {
                if SHIFT_PRESSED {
                    &SHIFTED_SCANCODE_MAP
                } else {
                    &SCANCODE_MAP
                }
            };
            map.get(scancode as usize).copied().flatten()
        }
    }
}

/// Table de mapping scancode -> KeyEvent
const SCANCODE_MAP: [Option<KeyEvent>; 128] = {
    let mut map: [Option<KeyEvent>; 128] = [None; 128];

    map[0x02] = Some(KeyEvent::Char('1')); map[0x03] = Some(KeyEvent::Char('2'));
    map[0x04] = Some(KeyEvent::Char('3')); map[0x05] = Some(KeyEvent::Char('4'));
    map[0x06] = Some(KeyEvent::Char('5')); map[0x07] = Some(KeyEvent::Char('6'));
    map[0x08] = Some(KeyEvent::Char('7')); map[0x09] = Some(KeyEvent::Char('8'));
    map[0x0A] = Some(KeyEvent::Char('9')); map[0x0B] = Some(KeyEvent::Char('0'));
    map[0x0C] = Some(KeyEvent::Char('-')); map[0x0D] = Some(KeyEvent::Char('='));
    map[0x0E] = Some(KeyEvent::Backspace); map[0x0F] = Some(KeyEvent::Tab);
    map[0x10] = Some(KeyEvent::Char('q')); map[0x11] = Some(KeyEvent::Char('w'));
    map[0x12] = Some(KeyEvent::Char('e')); map[0x13] = Some(KeyEvent::Char('r'));
    map[0x14] = Some(KeyEvent::Char('t')); map[0x15] = Some(KeyEvent::Char('y'));
    map[0x16] = Some(KeyEvent::Char('u')); map[0x17] = Some(KeyEvent::Char('i'));
    map[0x18] = Some(KeyEvent::Char('o')); map[0x19] = Some(KeyEvent::Char('p'));
    map[0x1A] = Some(KeyEvent::Char('[')); map[0x1B] = Some(KeyEvent::Char(']'));
    map[0x1C] = Some(KeyEvent::Enter);

    map[0x1E] = Some(KeyEvent::Char('a')); map[0x1F] = Some(KeyEvent::Char('s'));
    map[0x20] = Some(KeyEvent::Char('d')); map[0x21] = Some(KeyEvent::Char('f'));
    map[0x22] = Some(KeyEvent::Char('g')); map[0x23] = Some(KeyEvent::Char('h'));
    map[0x24] = Some(KeyEvent::Char('j')); map[0x25] = Some(KeyEvent::Char('k'));
    map[0x26] = Some(KeyEvent::Char('l')); map[0x27] = Some(KeyEvent::Char(';'));
    map[0x28] = Some(KeyEvent::Char('\'')); map[0x29] = Some(KeyEvent::Char('`'));

    map[0x2B] = Some(KeyEvent::Char('\\')); map[0x2C] = Some(KeyEvent::Char('z'));
    map[0x2D] = Some(KeyEvent::Char('x')); map[0x2E] = Some(KeyEvent::Char('c'));
    map[0x2F] = Some(KeyEvent::Char('v')); map[0x30] = Some(KeyEvent::Char('b'));
    map[0x31] = Some(KeyEvent::Char('n')); map[0x32] = Some(KeyEvent::Char('m'));
    map[0x33] = Some(KeyEvent::Char(',')); map[0x34] = Some(KeyEvent::Char('.'));
    map[0x35] = Some(KeyEvent::Char('/'));

    map[0x39] = Some(KeyEvent::Char(' '));
    map[0x48] = Some(KeyEvent::ArrowUp);
    map[0x4B] = Some(KeyEvent::ArrowLeft); 
    map[0x4D] = Some(KeyEvent::ArrowRight);
    map[0x50] = Some(KeyEvent::ArrowDown);

    map
};

const SHIFTED_SCANCODE_MAP: [Option<KeyEvent>; 128] = {
    let mut map: [Option<KeyEvent>; 128] = [None; 128];

    map[0x02] = Some(KeyEvent::Char('!')); map[0x03] = Some(KeyEvent::Char('@'));
    map[0x04] = Some(KeyEvent::Char('#')); map[0x05] = Some(KeyEvent::Char('$'));
    map[0x06] = Some(KeyEvent::Char('%')); map[0x07] = Some(KeyEvent::Char('^'));
    map[0x08] = Some(KeyEvent::Char('&')); map[0x09] = Some(KeyEvent::Char('*'));
    map[0x0A] = Some(KeyEvent::Char('(')); map[0x0B] = Some(KeyEvent::Char(')'));
    map[0x0C] = Some(KeyEvent::Char('_')); map[0x0D] = Some(KeyEvent::Char('+'));
    map[0x0E] = Some(KeyEvent::Backspace);
    
    map[0x10] = Some(KeyEvent::Char('Q')); map[0x11] = Some(KeyEvent::Char('W'));
    map[0x12] = Some(KeyEvent::Char('E')); map[0x13] = Some(KeyEvent::Char('R'));
    map[0x14] = Some(KeyEvent::Char('T')); map[0x15] = Some(KeyEvent::Char('Y'));
    map[0x16] = Some(KeyEvent::Char('U')); map[0x17] = Some(KeyEvent::Char('I'));
    map[0x18] = Some(KeyEvent::Char('O')); map[0x19] = Some(KeyEvent::Char('P'));
    map[0x1A] = Some(KeyEvent::Char('{')); map[0x1B] = Some(KeyEvent::Char('}'));
    map[0x1C] = Some(KeyEvent::Enter);

    map[0x1E] = Some(KeyEvent::Char('A')); map[0x1F] = Some(KeyEvent::Char('S'));
    map[0x20] = Some(KeyEvent::Char('D')); map[0x21] = Some(KeyEvent::Char('F'));
    map[0x22] = Some(KeyEvent::Char('G')); map[0x23] = Some(KeyEvent::Char('H'));
    map[0x24] = Some(KeyEvent::Char('J')); map[0x25] = Some(KeyEvent::Char('K'));
    map[0x26] = Some(KeyEvent::Char('L')); map[0x27] = Some(KeyEvent::Char(':'));
    map[0x28] = Some(KeyEvent::Char('"')); map[0x29] = Some(KeyEvent::Char('~'));
    
    map[0x2B] = Some(KeyEvent::Char('|')); map[0x2C] = Some(KeyEvent::Char('Z'));
    map[0x2D] = Some(KeyEvent::Char('X')); map[0x2E] = Some(KeyEvent::Char('C'));
    map[0x2F] = Some(KeyEvent::Char('V')); map[0x30] = Some(KeyEvent::Char('B'));
    map[0x31] = Some(KeyEvent::Char('N')); map[0x32] = Some(KeyEvent::Char('M'));
    map[0x33] = Some(KeyEvent::Char('<')); map[0x34] = Some(KeyEvent::Char('>'));
    map[0x35] = Some(KeyEvent::Char('?'));
    
    map[0x39] = Some(KeyEvent::Char(' '));
    map[0x48] = Some(KeyEvent::ArrowUp);
    map[0x4B] = Some(KeyEvent::ArrowLeft);
    map[0x4D] = Some(KeyEvent::ArrowRight);
    map[0x50] = Some(KeyEvent::ArrowDown);

    map
};
