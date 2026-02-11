use crate::io::{console, display, input_buffer};
use crate::drivers::{keyboard::KeyEvent};

pub fn handle_key_event(event: KeyEvent) {
    match event {
        KeyEvent::Char(c) => handle_insert(c),
        KeyEvent::Backspace => handle_delete(),
        KeyEvent::Enter => handle_enter(),
        KeyEvent::CtrlC => handle_ctrl_c(),
        KeyEvent::ArrowLeft => handle_arrow_left(),
        KeyEvent::ArrowRight => handle_arrow_right(),
        _ => {}
    }
}

fn handle_insert(c: char) {
    if input_buffer::insert_char(c as u8) {
        let buffer = input_buffer::get_buffer();
        let len = input_buffer::get_len();
        let start_pos = input_buffer::get_pos() - 1;
        let cursor_y = display::get_pos().1;
        let input_offset = console::input_start_col();
        display::redraw_input_line(buffer, len, start_pos, cursor_y, 0, input_offset);
        display::move_right();
    }
}

fn handle_delete() {
    if input_buffer::remove_char() {
        let buffer = input_buffer::get_buffer();
        let len = input_buffer::get_len();
        let start_pos = input_buffer::get_pos() - 1;
        let cursor_y = display::get_pos().1;
        let input_offset = console::input_start_col();
        display::redraw_input_line(buffer, len, start_pos, cursor_y, 1, input_offset);
        display::move_left();
    }
}

fn handle_enter() {
    let command = input_buffer::flush();
    display::new_line();
    crate::shell::handle_command(command);
    console::show_prompt();
}

fn handle_ctrl_c() {
    // Clear input, print ^C, new prompt
    input_buffer::flush();
    display::put_str("^C");
    display::new_line();
    console::show_prompt();
}

fn handle_arrow_left() {
    if input_buffer::can_move_left() {
        crate::io::input_buffer::move_left();
        crate::io::display::move_left();
    }
}

fn handle_arrow_right() {
    if input_buffer::can_move_right() {
        crate::io::input_buffer::move_right();
        crate::io::display::move_right();
    }
}