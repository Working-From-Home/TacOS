use crate::io::{console, cursor, display, input_buffer};
use crate::drivers::{keyboard::KeyEvent};

pub fn handle_key_event(event: KeyEvent) {
    match event {
        KeyEvent::Char(c) => handle_insert(c),
        KeyEvent::Backspace => handle_delete(),
        KeyEvent::Enter => handle_enter(),
        KeyEvent::ArrowLeft => handle_arrow_left(),
        KeyEvent::ArrowRight => handle_arrow_right(),
        _ => {}
    }
}

fn handle_insert(c: char) {
    if input_buffer::insert_char(c as u8) {
        let buffer = input_buffer::get_buffer();
        let len = input_buffer::get_len();
        //let (start_pos, cursor_y) = cursor::get_pos();
        let start_pos = input_buffer::get_pos() - 1;
        let cursor_y = cursor::get_pos().1;
        display::refresh_after_insert(buffer, len, start_pos, cursor_y);
        cursor::move_right();
    }
}

fn handle_delete() {
    if input_buffer::remove_char() {

        let buffer = input_buffer::get_buffer();
        let len = input_buffer::get_len();
        //let (start_pos, cursor_y) = cursor::get_pos();
        let start_pos = input_buffer::get_pos() - 1;
        let cursor_y = cursor::get_pos().1;
        display::refresh_after_delete(buffer, len, start_pos, cursor_y);
        cursor::move_left();
    }
}

fn handle_enter() {
    let _command = input_buffer::flush();
    
    // tmp
    for byte in _command {
        display::write_char(*byte);
    }

    //crate::shell::handle_command(command); TODO
    cursor::new_line();
    console::show_prompt();
}

fn handle_arrow_left() {
    if input_buffer::can_move_left() {
        crate::io::input_buffer::move_left();
        crate::io::cursor::move_left();
    }
}

fn handle_arrow_right() {
    if input_buffer::can_move_right() {
        crate::io::input_buffer::move_right();
        crate::io::cursor::move_right();
    }
}