use crate::drivers::keyboard;
use crate::io::{io_manager, console};

pub fn run() -> ! {

    console::show_welcome_message();
    console::show_prompt();

    loop {
        if let Some(event) = keyboard::get_key_event() {
            io_manager::handle_key_event(event);

            // match key {
            //     KeyEvent::Char(c) =>input_buffer::insert_char(c as u8),
            //     KeyEvent::Enter => {
            //         let cmd: *const u8 = input_buffer::handle_enter();
            //         // process_command(cmd);

            //         console::write_colored_line(cmd, vga::get_color_code(vga::Color::Green, vga::Color::Black));

            //         console::show_prompt();
            //     },
            //     KeyEvent::Backspace => input_buffer::remove_char(),
            //     KeyEvent::Space => input_buffer::insert_char(b' '),
            //     KeyEvent::Tab => input_buffer::insert_char(b'\t'),
            //     KeyEvent::ArrowLeft => input_buffer::handle_left(),
            //     KeyEvent::ArrowRight => input_buffer::handle_right(),
            //     KeyEvent::Unknown => {}
            // }
        }
    }
}