/// Shell console: prompt display and input-line geometry.

use crate::io::display;
use crate::drivers::vga;

const PROMPT: &str = "$ ";

pub fn show_welcome_message() {
    let c = vga::get_color_code(vga::Color::LightGray, vga::Color::Black);

    display::put_str_colored("  _/  _/      _/_/      _/_/_/_/_/                      _/_/      _/_/_/    _/\n", c);
    display::put_str_colored(" _/  _/    _/    _/        _/      _/_/_/    _/_/_/  _/    _/  _/          _/ \n", c);
    display::put_str_colored("_/_/_/_/      _/          _/    _/    _/  _/        _/    _/    _/_/      _/  \n", c);
    display::put_str_colored("   _/      _/            _/    _/    _/  _/        _/    _/        _/         \n", c);
    display::put_str_colored("  _/    _/_/_/_/        _/      _/_/_/    _/_/_/    _/_/    _/_/_/      _/    \n", c);

    display::put_str("\n\n\n");
}

static mut PROMPT_START_COL: usize = 0;

pub fn show_prompt() {
    let (x, _) = display::get_pos();
    unsafe { PROMPT_START_COL = x; }
    let color = vga::get_color_code(vga::Color::LightGray, vga::Color::Black);
    display::put_str_colored(PROMPT, color);
}

/// Returns the column where the input area begins (after the prompt).
pub fn input_start_col() -> usize {
    unsafe { PROMPT_START_COL + PROMPT.len() }
}

/// Returns the maximum number of input characters that fit on the current line.
pub fn max_input_len() -> usize {
    let start = input_start_col();
    if start >= vga::VGA_WIDTH {
        0
    } else {
        vga::VGA_WIDTH - start
    }
}

