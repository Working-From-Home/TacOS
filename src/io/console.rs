use crate::io::{cursor, display};
use crate::drivers::vga;
use crate::klib::string;

pub fn write_line(s: *const u8) {
    let len = string::strlen(s);

    let color = vga::get_color_code(vga::Color::LightGray, vga::Color::Black);

    for i in 0..len {
        unsafe {
            let c = *s.add(i);
            display::write_colored_char(c, color);
            cursor::move_right();
        }
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

    write_colored_line(b"  _/  _/      _/_/      _/_/_/_/_/                      _/_/      _/_/_/    _/\0".as_ptr(), c);   
    write_colored_line(b" _/  _/    _/    _/        _/      _/_/_/    _/_/_/  _/    _/  _/          _/ \0".as_ptr(), c);   
    write_colored_line(b"_/_/_/_/      _/          _/    _/    _/  _/        _/    _/    _/_/      _/  \0".as_ptr(), c);   
    write_colored_line(b"   _/      _/            _/    _/    _/  _/        _/    _/        _/         \0".as_ptr(), c);   
    write_colored_line(b"  _/    _/_/_/_/        _/      _/_/_/    _/_/_/    _/_/    _/_/_/      _/    \0".as_ptr(), c);   

    cursor::new_line();
    cursor::new_line();
    cursor::new_line();
}

static mut PROMPT_START_COL: usize = 0;

pub fn show_prompt() {
    let (x, _) = cursor::get_pos();
    unsafe { PROMPT_START_COL = x; }
    let color = vga::get_color_code(vga::Color::LightGray, vga::Color::Black);
    let prompt = b"$ ";
    for &c in prompt {
        display::write_colored_char(c, color);
        cursor::move_right();
    }
}

pub const PROMPT_LEN: usize = 2; // "$ "

/// Returns the column where the input area begins (after the prompt).
pub fn input_start_col() -> usize {
    unsafe { PROMPT_START_COL + PROMPT_LEN }
}

/// Returns the maximum number of input characters that fit on the current line.
pub fn max_input_len() -> usize {
    let start = input_start_col();
    if start >= crate::drivers::vga::VGA_WIDTH {
        0
    } else {
        crate::drivers::vga::VGA_WIDTH - start
    }
}

pub fn show_error(msg: &str) {
    let color: u8 = vga::get_color_code(vga::Color::Red, vga::Color::Black);
    write_colored_line(msg.as_ptr(), color);
}