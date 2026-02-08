use crate::drivers::keyboard;
use crate::drivers::port::outb;
use crate::io::{console, io_manager};
use crate::{printk, printkln};
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

pub fn handle_command(input: &'static [u8]) {
    if input.is_empty() {
        return;
    }

    let (cmd, args) = parse_input(input);

    match cmd {
        b"help" => {
            console::write_line(
                b"Available commands: help, echo, tacos, shutdown, halt, printk\0".as_ptr(),
            );
        }
        b"shutdown" | b"halt" => {
            shutdown();
        }
        b"tacos" => {
            tacos();
        }
        b"printk" => {
            printk_test();
        }
        b"echo" => {
            super::builtin::echo::echo(args);
        }
        _ => {
            printkln!("Unknown command: {}", cmd);
        }
    }
}

/// Splits input into (command, args) by finding the first unquoted space.
/// Handles quote stripping and escape sequences at the parser level so
/// commands receive clean, pre-processed arguments.
///
/// Examples:
///   `echo "hello world"`  → cmd=`echo`, args=`hello world`
///   `echo"v"`             → cmd=`echov`, args=``  (like zsh)
///   `echo "a\nb"`         → cmd=`echo`, args=`a<LF>b`
///   `echo \z`             → cmd=`echo`, args=`` (unknown escape dropped)

const PARSE_BUF_SIZE: usize = 80;
static mut PARSE_BUF: [u8; PARSE_BUF_SIZE] = [0; PARSE_BUF_SIZE];

fn parse_input(input: &[u8]) -> (&'static [u8], &'static [u8]) {
    let buf = unsafe { &mut PARSE_BUF };
    let mut out: usize = 0;
    let mut i: usize = 0;
    let len = input.len();

    // Skip leading spaces
    while i < len && input[i] == b' ' {
        i += 1;
    }

    // Parse command token (stops at unquoted space)
    let cmd_start: usize = 0;
    i = parse_token(input, i, buf, &mut out);
    let cmd_end = out;

    // Skip spaces between command and args
    while i < len && input[i] == b' ' {
        i += 1;
    }

    // Parse args — preserves single space between tokens
    let args_start = out;
    while i < len {
        if input[i] == b' ' {
            // Collapse multiple spaces into one
            while i < len && input[i] == b' ' {
                i += 1;
            }
            // Only add separator if there's more content
            if i < len && out < PARSE_BUF_SIZE {
                buf[out] = b' ';
                out += 1;
            }
        } else {
            i = parse_token(input, i, buf, &mut out);
        }
    }
    let args_end = out;

    // SAFETY: all indices are <= PARSE_BUF_SIZE, validated by bounds checks above
    let cmd = unsafe { PARSE_BUF.get_unchecked(cmd_start..cmd_end) };
    let args = unsafe { PARSE_BUF.get_unchecked(args_start..args_end) };
    (cmd, args)
}

/// Parses a single token from `input[i..]` into `buf[*out..]`.
/// Handles quotes (single/double) and escape sequences.
/// Stops at the first unquoted space or end of input.
/// Returns the new position in `input`.
fn parse_token(
    input: &[u8],
    mut i: usize,
    buf: &mut [u8; PARSE_BUF_SIZE],
    out: &mut usize,
) -> usize {
    let len = input.len();

    while i < len && input[i] != b' ' {
        if input[i] == b'"' || input[i] == b'\'' {
            let quote = input[i];
            i += 1;
            while i < len && input[i] != quote {
                if input[i] == b'\\' && i + 1 < len {
                    i += 1;
                    let esc = escape_char(input[i]);
                    if esc != 0xFF && *out < PARSE_BUF_SIZE {
                        buf[*out] = esc;
                        *out += 1;
                    }
                } else if *out < PARSE_BUF_SIZE {
                    buf[*out] = input[i];
                    *out += 1;
                }
                i += 1;
            }
            // Skip closing quote
            if i < len && input[i] == quote {
                i += 1;
            }
        } else if input[i] == b'\\' && i + 1 < len {
            i += 1;
            let esc = escape_char(input[i]);
            if esc != 0xFF && *out < PARSE_BUF_SIZE {
                buf[*out] = esc;
                *out += 1;
            }
            i += 1;
        } else {
            if *out < PARSE_BUF_SIZE {
                buf[*out] = input[i];
                *out += 1;
            }
            i += 1;
        }
    }
    i
}

/// Converts the character after a backslash into the corresponding escape byte.
/// Returns 0xFF for unknown escapes (sentinel: character is dropped).
fn escape_char(c: u8) -> u8 {
    match c {
        b'n' => b'\n',
        b't' => b'\t',
        b'r' => b'\r',
        b'v' => 0x0B,
        b'\\' => b'\\',
        b'\'' => b'\'',
        b'"' => b'"',
        b'0' => 0,
        _ => 0xFF, // unknown escape: drop
    }
}

fn tacos() {
    static mut tacos_counter: u8 = 0;
    unsafe {
        tacos_counter += 9;
    }
    printkln!("You ate {} tacos!\0", unsafe { tacos_counter });
}

fn printk_test() {
    printkln!("=== printk test ===");
    printkln!("String: {}", "Hello from TacOS");
    printkln!("Integer: {}", 42);
    printkln!("Negative: {}", -1_i32);
    printkln!("Hex lower: {:x}", 255_u32);
    printkln!("Hex upper: {:X}", 255_u32);
    printkln!("Hex alt of {} : {:#x}", 0xDEAD_u32, 0xDEAD_u32);
    printkln!("Binary of {} : {:b}", 42_u32, 42_u32);
    printkln!("Binary alt of {} : {:#b}", 10_u32, 10_u32);
    printkln!("Octal of {} : {:o}", 42_u32, 42_u32);
    printkln!("Octal alt of {} : {:#o}", 42_u32, 42_u32);
    printkln!("Bool: {} {}", true, false);
    printkln!("Multi: {} + {} = {}", 1, 2, 3);
    printk!("No newline...");
    printkln!(" done!");
    printkln!("Literal braces: {{}}");
    printk!("Two new lines...\n\nDone!");
    printkln!("=== end test ===");
    printkln!();
}

fn shutdown() {
    outb(0xF4, 0x00);
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
