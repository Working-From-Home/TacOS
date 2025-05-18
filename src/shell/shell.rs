use crate::drivers::{keyboard, vga};
use crate::ui::{console, cursor, input, display};

pub fn run() -> ! {

    console::show_welcome_message();
    console::show_prompt();

    loop {
        if let Some(c) = keyboard::get_char() {
            match c {
                '\n' => handle_enter(),
                '\x08' | '\x7f' => input::remove_char(), // backspace or delete
                '\x1b' => {
                    // handle escape sequences
                    if let Some(next_c) = keyboard::get_char() {
                        match next_c {
                            'D' => handle_left(),  // left arrow
                            'C' => handle_right(), // right arrow
                            _ => {}
                        }
                    }
                }
                c =>  input::insert_char(c as u8),
            }
        }
    }
}

fn handle_left() {
    cursor::move_left();
}

fn handle_right() {
    cursor::move_right();
}

fn handle_enter() {
    cursor::new_line();
    // TODO: implement command processing
    input::clear();
    console::show_prompt();
}