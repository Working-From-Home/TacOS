use crate::io::display;
use crate::drivers::vga;

pub fn show_welcome_message() {
    let c = vga::get_color_code(vga::Color::LightGray, vga::Color::Black);

    display::put_colored_str("  _/  _/      _/_/      _/_/_/_/_/                      _/_/      _/_/_/    _/\n", c);
    display::put_colored_str(" _/  _/    _/    _/        _/      _/_/_/    _/_/_/  _/    _/  _/          _/ \n", c);
    display::put_colored_str("_/_/_/_/      _/          _/    _/    _/  _/        _/    _/    _/_/      _/  \n", c);
    display::put_colored_str("   _/      _/            _/    _/    _/  _/        _/    _/        _/         \n", c);
    display::put_colored_str("  _/    _/_/_/_/        _/      _/_/_/    _/_/_/    _/_/    _/_/_/      _/    \n", c);

    display::new_line();
    display::new_line();
    display::new_line();
}

static mut PROMPT_START_COL: usize = 0;

pub fn show_prompt() {
    let (x, _) = display::get_pos();
    unsafe { PROMPT_START_COL = x; }
    let color = vga::get_color_code(vga::Color::LightGray, vga::Color::Black);
    display::put_colored_str("$ ", color);
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

