use crate::drivers::keyboard;
use crate::drivers::port::outb;
use crate::io::{console, io_manager};
use crate::{print, println, printk, printkln};
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

type CmdHandler = fn(argv: &'static [&'static [u8]]);

struct Command {
    name: &'static [u8],
    handler: CmdHandler,
}

static COMMANDS: &[Command] = &[
    Command { name: b"help",     handler: |_| help() },
    Command { name: b"echo",     handler: super::builtin::echo::echo },
    Command { name: b"tacos",    handler: |_| tacos() },
    Command { name: b"shutdown", handler: |_| shutdown() },
    Command { name: b"halt",     handler: |_| shutdown() },
    Command { name: b"reboot",   handler: |_| reboot() },
    Command { name: b"printk",   handler: |_| printk_test() },
    Command { name: b"stack",    handler: |_| crate::klib::stack::print_stack() },
    Command { name: b"stack_test",    handler: |_| stack_test() },
    Command { name: b"gdt",      handler: |_| crate::gdt::print_gdt() },
    Command { name: b"dmesg",    handler: super::builtin::dmesg::dmesg },
];

fn starts_with(haystack: &[u8], needle: &[u8]) -> bool {
    if haystack.len() < needle.len() {
        return false;
    }
    let mut i = 0;
    while i < needle.len() {
        if unsafe { *haystack.get_unchecked(i) } != unsafe { *needle.get_unchecked(i) } {
            return false;
        }
        i += 1;
    }
    true
}

pub fn handle_command(input: &'static [u8]) {
    if input.is_empty() {
        return;
    }

    let argv = parse_input(input);
    if argv.is_empty() {
        return;
    }

    let cmd_name = argv[0];

    for entry in COMMANDS.iter() {
        if starts_with(cmd_name, entry.name) && cmd_name.len() == entry.name.len() {
            (entry.handler)(argv);
            return;
        }
    }
    println!("Unknown command: {}", cmd_name);
}

fn help() {
    println!("Available commands:");
    let mut first = true;
    for entry in COMMANDS.iter() {
        if first {
            print!(" ");
            first = false;
        } else {
            print!(", ");
        }
        print!("{}", entry.name);
    }
    println!();
}

const MAX_ARGS: usize = 16;
const PARSE_BUF_SIZE: usize = 80;

static mut PARSE_BUF: [u8; PARSE_BUF_SIZE] = [0; PARSE_BUF_SIZE];
static mut ARGV: [&'static [u8]; MAX_ARGS] = [&[]; MAX_ARGS];

/// Parses input into argv-style array.
/// Returns slice of argument slices where argv[0] is command, argv[1..] are args.
fn parse_input(input: &[u8]) -> &'static [&'static [u8]] {
    let buf = unsafe { &mut PARSE_BUF };
    let argv = unsafe { &mut ARGV };

    let mut out: usize = 0; // Position in PARSE_BUF
    let mut argc: usize = 0; // Number of arguments
    let mut i: usize = 0;
    let len = input.len();

    // Skip leading spaces
    while i < len && input[i] == b' ' {
        i += 1;
    }

    // Parse tokens
    while i < len && argc < MAX_ARGS {
        if input[i] == b' ' {
            // Skip consecutive spaces
            while i < len && input[i] == b' ' {
                i += 1;
            }
        } else {
            // Parse one token
            let start = out;
            i = parse_token(input, i, buf, &mut out);

            // Store this argument
            argv[argc] = unsafe { PARSE_BUF.get_unchecked(start..out) };
            argc += 1;
        }
    }

    unsafe { ARGV.get_unchecked(..argc) }
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
        _ => 0xFF,
    }
}

fn tacos() {
    static mut tacos_counter: u8 = 1;
    println!("You ate {} tacos!\0", unsafe { tacos_counter });
    unsafe {
        tacos_counter += 3;
    }
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
    printk!("Two new lines...\n\nDone!\n");
    printkln!("=== end test ===");
    printkln!();
    println!("printk test done! Use `dmesg` to see the output.");
}

fn stack_test() {
    #[inline(never)]
    fn recursive(n: u32) {
        if n == 0 {
            crate::klib::stack::print_stack();
        } else {
            recursive(n - 1);
            // Prevent tail-call optimization: force the compiler to keep the frame alive
            unsafe {
                core::arch::asm!("", options(nomem, nostack));
            }
        }
    }
    recursive(5);
}

fn shutdown() {
    outb(0xF4, 0x00);
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

fn reboot() {
    println!("Rebooting...");

    let mut status = crate::drivers::port::inb(0x64);
    while status & 0x02 != 0 {
       status = crate::drivers::port::inb(0x64);
    }
    outb(0x64, 0xFE);
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
