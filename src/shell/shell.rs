use crate::drivers::keyboard;
use crate::drivers::port::outb;
use crate::io::{io_manager, console};
use crate::drivers::vga;
use crate::klib::string::strcat;
use core::arch::asm;

pub fn run() -> ! {
    console::show_welcome_message();
    console::show_prompt();

    loop {
        if let Some(event) = keyboard::get_key_event() {
            io_manager::handle_key_event(event);
        }
    }
}

pub fn handle_command(command: &'static [u8]) {
    if command.is_empty() {
        return;
    }
    match command {
        b"help" => {
            console::write_line(b"Available commands: help, tacos, shutdown\0".as_ptr());
        }
        b"shutdown" => {
            shutdown();
        }
        b"tacos" => {
            tacos();
        }
        _ => {
            // unknown command case
            console::show_error("Command not found\0");
        }
    }
}

fn tacos() {
    static mut tacos_counter: u8 = 0;
    unsafe {
        tacos_counter += 1;
    } 
    match unsafe {tacos_counter} {
        21 => console::write_line(b"Still loving those tacos!\0".as_ptr()),
        42 => console::write_line(b"Why are you still here? Go eat tacos!\0".as_ptr()),
        _ => console::write_line(b"Yum! Tacos are delicious!\0".as_ptr()),
    }
}

fn shutdown() {
    outb(0xF4, 0x00);
    loop {
        unsafe { asm!("hlt"); }
    }
}