use crate::io::{cursor, display};
use crate::drivers::vga;
use crate::klib::string;

pub fn write_line(s: &str) {
    for byte in s.bytes() {
        display::write_char(byte);
    }
    cursor::new_line();
}

pub fn write_colored_line(s: *const u8, color: u8) {
    let len = string::strlen(s);
    for i in 0..len {
        unsafe {
            let c = *s.add(i);
            display::write_colored_char(c, color);
            cursor::move_right();
        }
    }
    cursor::new_line();
}

pub fn show_welcome_message() {
    let c = vga::get_color_code(vga::Color::LightGray, vga::Color::Black);

    write_colored_line(b"_/_/_/_/_/                      _/_/      _/_/_/      _/\0".as_ptr(), c);
    write_colored_line(b"   _/      _/_/_/    _/_/_/  _/    _/  _/            _/\0".as_ptr(), c);
    write_colored_line(b"  _/    _/    _/  _/        _/    _/    _/_/        _/\0".as_ptr(), c);
    write_colored_line(b" _/    _/    _/  _/        _/    _/        _/\0".as_ptr(), c);
    write_colored_line(b"_/      _/_/_/    _/_/_/    _/_/    _/_/_/        _/\0".as_ptr(), c);

    cursor::new_line();
    cursor::new_line();
    cursor::new_line();
}

pub fn show_prompt() {
    let color = vga::get_color_code(vga::Color::LightGray, vga::Color::Black);
    let prompt = b"$ ";
    for &c in prompt {
        display::write_colored_char(c, color);
        cursor::move_right();
    }
}

// temporary. need to find a better way to handle this
pub const PROMPT_LEN: usize = 2; // "$ "

pub fn show_error(msg: &str) {
    let color: u8 = vga::get_color_code(vga::Color::Red, vga::Color::Black);
    for &c in msg.as_bytes() {
        display::write_colored_char(c, color);
        cursor::move_right();
    }
    display::write_char(b'\n');
    cursor::new_line();
}