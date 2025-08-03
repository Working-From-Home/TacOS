use crate::drivers::keyboard;
use crate::drivers::port::outb;
use crate::io::{io_manager, console};
use crate::drivers::vga;
use crate::klib::string::strcat;
use core::arch::asm;

macro_rules! give_six {
    () => {
        6
    };
    
}

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
        b"six" => {
            let six = give_six!();
            // in kernel, use our own itoa to convert integer to string
            let mut buffer = [0u8; 64];
            let message = u8_itoa(six, &mut buffer);
            console::write_line(message.as_ptr());
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

// Simple integer to string conversion for u8
fn u8_itoa(mut num: u8, buffer: &mut [u8]) -> & [u8] {
    let mut i = 0;
    if num == 0 {
        buffer[0] = b'0';
        buffer[1] = 0;
        return &buffer[..2];
    }
    while num > 0 {
        buffer[i] = b'0' + (num % 10);
        num /= 10;
        i += 1;
    }
    // Reverse the digits
    for j in 0..i/2 {
        let tmp = buffer[j];
        buffer[j] = buffer[i - j - 1];
        buffer[i - j - 1] = tmp;
    }
    buffer[i] = 0;
    &buffer[..i+1]
}