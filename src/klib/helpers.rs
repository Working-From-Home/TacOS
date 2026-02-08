#![allow(dead_code)]

use crate::io::display;

pub fn print_hex(byte: u8) {
    fn to_hex_char(n: u8) -> u8 {
        match n {
            0..=9 => b'0' + n,
            10..=15 => b'a' + (n - 10),
            _ => b'?', // should never happen
        }
    }

    let high = to_hex_char((byte >> 4) & 0x0F);
    let low = to_hex_char(byte & 0x0F);

    display::write_char(high);
    display::write_char(low);
    display::write_char(b' ');
}
