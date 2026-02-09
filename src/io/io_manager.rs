use crate::io::{console, cursor, display, input_buffer, scrollback};
use crate::drivers::{keyboard::KeyEvent, mouse::ScrollEvent};

pub fn handle_key_event(event: KeyEvent) {
    // If scrolled back, any key except PageUp/PageDown returns to live view
    match event {
        KeyEvent::PageUp | KeyEvent::ScrollUp => {
            if !scrollback::is_scrolled_back() {
                scrollback::save_live_screen();
            }
            scrollback::scroll_up(5);
            return;
        }
        KeyEvent::PageDown | KeyEvent::ScrollDown => {
            scrollback::scroll_down(5);
            return;
        }
        _ => {
            if scrollback::is_scrolled_back() {
                scrollback::scroll_down(usize::MAX);
            }
        }
    }

    match event {
        KeyEvent::Char(c) => handle_insert(c),
        KeyEvent::Backspace => handle_delete(),
        KeyEvent::Enter => handle_enter(),
        KeyEvent::Tab => handle_tab(),
        KeyEvent::ArrowLeft => handle_arrow_left(),
        KeyEvent::ArrowRight => handle_arrow_right(),
        KeyEvent::ArrowUp => handle_history_up(),
        KeyEvent::ArrowDown => handle_history_down(),
        _ => {}
    }
}

fn handle_insert(c: char) {
    if input_buffer::insert_char(c as u8) {
        let buffer = input_buffer::get_buffer();
        let len = input_buffer::get_len();
        let pos = input_buffer::get_pos();
        let cursor_y = cursor::get_pos().1;
        display::write_buffer_line(buffer, len, pos - 1, cursor_y, 0);
        let vcol = display::visual_col(buffer, pos);
        cursor::set_pos(console::input_start_col() + vcol, cursor_y);
    }
}

fn handle_tab() {
    // Compute visual width before insertion
    let old_vlen = display::visual_col(input_buffer::get_buffer(), input_buffer::get_len());
    if input_buffer::insert_char(b'\t') {
        let buffer = input_buffer::get_buffer();
        let len = input_buffer::get_len();
        let new_vlen = display::visual_col(buffer, len);
        // Reject if it would overflow the line visually
        if new_vlen > console::max_input_len() {
            input_buffer::remove_char();
            return;
        }
        let pos = input_buffer::get_pos();
        let cursor_y = cursor::get_pos().1;
        display::write_buffer_line(buffer, len, pos - 1, cursor_y, 0);
        let vcol = display::visual_col(buffer, pos);
        cursor::set_pos(console::input_start_col() + vcol, cursor_y);
    }
}

fn handle_delete() {
    let old_vlen = display::visual_col(input_buffer::get_buffer(), input_buffer::get_len());
    if input_buffer::remove_char() {
        let buffer = input_buffer::get_buffer();
        let len = input_buffer::get_len();
        let pos = input_buffer::get_pos();
        let cursor_y = cursor::get_pos().1;
        let new_vlen = display::visual_col(buffer, len);
        let clear = if old_vlen > new_vlen { old_vlen - new_vlen } else { 0 };
        display::write_buffer_line(buffer, len, pos, cursor_y, clear);
        let vcol = display::visual_col(buffer, pos);
        cursor::set_pos(console::input_start_col() + vcol, cursor_y);
    }
}

fn handle_enter() {
    let command = input_buffer::flush();
    // Save to history before executing
    crate::io::history::push(command);
    crate::io::history::reset_browse();
    cursor::new_line();
    crate::shell::handle_command(command);
    console::show_prompt();
}

fn handle_arrow_left() {
    if input_buffer::can_move_left() {
        input_buffer::move_left();
        let buffer = input_buffer::get_buffer();
        let pos = input_buffer::get_pos();
        let vcol = display::visual_col(buffer, pos);
        let y = cursor::get_pos().1;
        cursor::set_pos(console::input_start_col() + vcol, y);
    }
}

fn handle_arrow_right() {
    if input_buffer::can_move_right() {
        input_buffer::move_right();
        let buffer = input_buffer::get_buffer();
        let pos = input_buffer::get_pos();
        let vcol = display::visual_col(buffer, pos);
        let y = cursor::get_pos().1;
        cursor::set_pos(console::input_start_col() + vcol, y);
    }
}

fn handle_history_up() {
    if let Some(cmd) = crate::io::history::up() {
        replace_input(cmd);
    }
}

fn handle_history_down() {
    if let Some(cmd) = crate::io::history::down() {
        replace_input(cmd);
    }
}

/// Replaces the current input line with `new_content` and redraws.
fn replace_input(new_content: &[u8]) {
    let old_vlen = display::visual_col(input_buffer::get_buffer(), input_buffer::get_len());
    let cursor_y = cursor::get_pos().1;

    // Set the new content
    input_buffer::set_content(new_content);

    let buffer = input_buffer::get_buffer();
    let new_len = input_buffer::get_len();
    let new_vlen = display::visual_col(buffer, new_len);

    // Redraw the line from position 0, clearing any leftover visual columns
    let clear = if old_vlen > new_vlen { old_vlen - new_vlen } else { 0 };
    display::write_buffer_line(buffer, new_len, 0, cursor_y, clear);

    // Move cursor to end of new input
    cursor::set_pos(console::input_start_col() + new_vlen, cursor_y);
}

/// Handle mouse scroll wheel events for terminal scrollback.
pub fn handle_scroll(event: ScrollEvent) {
    match event {
        ScrollEvent::Up => {
            if !scrollback::is_scrolled_back() {
                scrollback::save_live_screen();
            }
            scrollback::scroll_up(3);
        }
        ScrollEvent::Down => {
            scrollback::scroll_down(3);
        }
    }
}