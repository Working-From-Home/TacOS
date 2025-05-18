use crate::drivers::vga;
use crate::drivers::keyboard;
use crate::klib::string;

const INPUT_BUFFER_SIZE: usize = 256;
static mut INPUT_BUFFER: [u8; INPUT_BUFFER_SIZE] = [0; INPUT_BUFFER_SIZE];
static mut CURSOR_POS: usize = 0;

pub fn run() -> ! {
    print_welcome();
    print_prompt();

    loop {
        unsafe {
            if let Some(c) = keyboard::get_char() {
                match c {
                    '\n' => handle_enter(),
                    '\x08' | '\x7f' => handle_backspace(), // backspace
                    c =>  handle_char(c as u8),
                }
            }
        }
    }
}

fn handle_char(c: u8) {
    unsafe {
        if CURSOR_POS < INPUT_BUFFER_SIZE - 1 {
            INPUT_BUFFER[CURSOR_POS] = c;
            CURSOR_POS += 1;
            vga::putchar(c);
        }
    }
}

fn handle_enter() {
    vga::putchar(b'\n');
    unsafe {
        if let Some(slot) = INPUT_BUFFER.get_mut(CURSOR_POS) {
            *slot = 0;
        }

        //process_command(INPUT_BUFFER.as_ptr());

        CURSOR_POS = 0;
        INPUT_BUFFER.fill(0);
    }
    print_prompt();
}

fn handle_backspace() {
    unsafe {
        if CURSOR_POS > 0 {
            CURSOR_POS -= 1;
            vga::backspace();
        }
    }
}

fn process_command(ptr: *const u8) {
    let len = unsafe { string::strlen(ptr) };
    if len == 0 {
        return;
    }

    // command processing logic
}


fn print_welcome() {
    let c = vga::get_color_code(vga::Color::LightGray, vga::Color::Black);

    vga::putstr_colored(b"_/_/_/_/_/                      _/_/      _/_/_/      _/\n\0".as_ptr(), c);
    vga::putstr_colored(b"   _/      _/_/_/    _/_/_/  _/    _/  _/            _/\n\0".as_ptr(), c);
    vga::putstr_colored(b"  _/    _/    _/  _/        _/    _/    _/_/        _/\n\0".as_ptr(), c);
    vga::putstr_colored(b" _/    _/    _/  _/        _/    _/        _/\n\0".as_ptr(), c);
    vga::putstr_colored(b"_/      _/_/_/    _/_/_/    _/_/    _/_/_/        _/\n\n\n\0".as_ptr(), c);
}

fn print_prompt() {
    let c = vga::get_color_code(vga::Color::LightGray, vga::Color::Black);
    vga::putstr_colored(b"> \0".as_ptr(), c);
}