/// Keyboard event dispatcher.
///
/// Translates `KeyEvent`s into input-buffer mutations
/// and display updates.

use crate::io::{display, input_buffer};
use crate::shell::console;
use crate::drivers::keyboard::KeyEvent;

/// Dispatches a keyboard event to the appropriate handler.
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

// ──────────────────────────────────────────────
//  Input-line redraw
// ──────────────────────────────────────────────

/// Redraws the input line from position `from` onward,
/// clears one trailing cell (for backspace), and repositions the cursor.
fn refresh_input_from(from: usize) {
    let buffer = input_buffer::get_buffer();
    let pos = input_buffer::get_pos();
    let (_, y) = display::get_pos();
    let offset = console::input_start_col();

    // Redraw characters from `from` to end of buffer
    for i in from..buffer.len() {
        display::put_char_at(i + offset, y, unsafe { *buffer.get_unchecked(i) });
    }

    // Clear the cell right after the buffer (handles backspace residue)
    display::put_char_at(buffer.len() + offset, y, b' ');

    // Reposition cursor to match the logical input position
    display::set_pos(pos + offset, y);
}

// ──────────────────────────────────────────────
//  Key event handlers
// ──────────────────────────────────────────────

/// Inserts a character and redraws from insertion point.
fn handle_insert(c: char) {
    if input_buffer::insert_char(c as u8, console::max_input_len()) {
        refresh_input_from(input_buffer::get_pos() - 1);
    }
}

/// Deletes the character before the cursor and redraws.
fn handle_delete() {
    if input_buffer::remove_char() {
        refresh_input_from(input_buffer::get_pos());
    }
}

/// Flushes the buffer, executes the command, shows prompt.
fn handle_enter() {
    let command = input_buffer::flush();
    display::new_line();
    crate::shell::handle_command(command);
    console::show_prompt();
}

/// Discards input, prints ^C, shows prompt.
fn handle_ctrl_c() {
    input_buffer::flush();
    display::put_str("^C\n");
    console::show_prompt();
}

/// Moves the input cursor one position left.
fn handle_arrow_left() {
    if input_buffer::can_move_left() {
        input_buffer::move_left();
        display::move_left();
    }
}

/// Moves the input cursor one position right.
fn handle_arrow_right() {
    if input_buffer::can_move_right() {
        input_buffer::move_right();
        display::move_right();
    }
}