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

type CmdHandler = fn(args: &'static [u8]);

struct Command {
    name: &'static [u8],
    handler: CmdHandler,
}

static COMMANDS: &[Command] = &[
    Command { name: b"help",     handler: help },
    Command { name: b"echo",     handler: super::builtin::echo::echo },
    Command { name: b"tacos",    handler: tacos },
    Command { name: b"shutdown", handler: shutdown },
    Command { name: b"halt",     handler: shutdown },
    Command { name: b"reboot",   handler: reboot },
    Command { name: b"printk",   handler: printk_test },
    Command { name: b"stack",    handler: crate::klib::stack::print_stack },
    Command { name: b"gdt",      handler: crate::gdt::print_gdt },
    Command { name: b"mem",      handler: crate::memory::frame::print_info },
    Command { name: b"mmap",     handler: crate::memory::frame::print_mmap },
    Command { name: b"paging",   handler: crate::memory::paging::print_info },
    Command { name: b"heap",     handler: crate::memory::heap::print_info },
    Command { name: b"vmalloc",  handler: crate::memory::virt::print_info },
    Command { name: b"panic",    handler: trigger_panic },
    Command { name: b"alloc",    handler: test_alloc },
];

/// Checks if `args` starts with `name` followed by a space or end of input.
/// Returns the remainder after the command (skipping the separating space).
/// Checks if `haystack` starts with `needle`.
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

    let args = parse_input(input);
    if args.is_empty() {
        return;
    }

    for entry in COMMANDS.iter() {
        let n = entry.name.len();
        if starts_with(args, entry.name)
            && (args.len() == n || unsafe { *args.get_unchecked(n) } == b' ')
        {
            (entry.handler)(args);
            return;
        }
    }
    printkln!("Unknown command: {}", args);
}

fn help(_args: &[u8]) {
    printk!("Available commands:");
    let mut first = true;
    for entry in COMMANDS.iter() {
        if first {
            printk!(" ");
            first = false;
        } else {
            printk!(", ");
        }
        printk!("{}", entry.name);
    }
    printkln!();
}

/// Parses all tokens from `input` into a static argv array.
/// Each whitespace-delimited token becomes one entry in `ARGV`.
/// Handles quote stripping and escape sequences so commands
/// receive clean, pre-processed arguments.
/// Returns the number of arguments (argc).
///
/// Examples:
///   `echo "hello world"`  → argv = [`echo`, `hello world`]
///   `echo"v"`             → argv = [`echov`]  (like zsh)
///   `echo "a\nb"`         → argv = [`echo`, `a<LF>b`]
///   `echo \z`             → argv = [`echo`] (unknown escape dropped)

const PARSE_BUF_SIZE: usize = 80;
static mut PARSE_BUF: [u8; PARSE_BUF_SIZE] = [0; PARSE_BUF_SIZE];

fn parse_input(input: &[u8]) -> &'static [u8] {
    let buf = unsafe { &mut PARSE_BUF };
    let mut out: usize = 0;
    let mut i: usize = 0;
    let len = input.len();

    // Skip leading spaces
    while i < len && input[i] == b' ' {
        i += 1;
    }

    // Parse all tokens into buffer, separated by single spaces
    while i < len {
        if input[i] == b' ' {
            while i < len && input[i] == b' ' {
                i += 1;
            }
            if i < len && out < PARSE_BUF_SIZE {
                buf[out] = b' ';
                out += 1;
            }
        } else {
            i = parse_token(input, i, buf, &mut out);
        }
    }

    unsafe { PARSE_BUF.get_unchecked(..out) }
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

fn tacos(_args: &[u8]) {
    static mut tacos_counter: u8 = 1;
    printkln!("You ate {} tacos!\0", unsafe { tacos_counter });
    unsafe {
        tacos_counter += 3;
    }
}

fn printk_test(_args: &[u8]) {
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

fn shutdown(_args: &[u8]) {
    outb(0xF4, 0x00);
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

fn reboot(_args: &[u8]) {
    printkln!("Rebooting...");
    // Send reset command (0xFE) to the keyboard controller (port 0x64)
    // This triggers a CPU reset on real hardware and in QEMU
    // Wait for the keyboard controller input buffer to be empty
    let mut status = crate::drivers::port::inb(0x64);
    while status & 0x02 != 0 {
        status = crate::drivers::port::inb(0x64);
    }
    outb(0x64, 0xFE);
    // If the above didn't work, triple-fault by loading a null IDT and interrupting
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

fn trigger_panic(_args: &[u8]) {
    crate::kernel_panic!("User-triggered panic from shell");
}

fn test_alloc(_args: &[u8]) {
    use crate::memory::heap;
    use crate::memory::virt;

    printkln!("=== Memory Allocation Test ===");

    // Test kmalloc
    printkln!("Testing kmalloc...");
    let ptr1 = heap::kmalloc(64);
    printkln!("  kmalloc(64) = {:#x}", ptr1 as u32);
    printkln!("  ksize = {}", heap::ksize(ptr1));

    let ptr2 = heap::kmalloc(128);
    printkln!("  kmalloc(128) = {:#x}", ptr2 as u32);
    printkln!("  ksize = {}", heap::ksize(ptr2));

    // Write to allocated memory
    if !ptr1.is_null() {
        unsafe {
            *ptr1 = 0xAA;
            *(ptr1.add(63)) = 0xBB;
        }
        printkln!("  Write/read OK: first={:#x}, last={:#x}",
            unsafe { *ptr1 } as u32, unsafe { *(ptr1.add(63)) } as u32);
    }

    // Test kfree
    printkln!("Testing kfree...");
    heap::kfree(ptr1);
    printkln!("  kfree(ptr1) OK");

    // Reallocate from freed space
    let ptr3 = heap::kmalloc(32);
    printkln!("  kmalloc(32) = {:#x} (should reuse freed space)", ptr3 as u32);

    heap::kfree(ptr2);
    heap::kfree(ptr3);
    printkln!("  kfree all OK");

    // Test vmalloc
    printkln!("Testing vmalloc...");
    let vptr = virt::vmalloc(4096);
    printkln!("  vmalloc(4096) = {:#x}", vptr as u32);
    printkln!("  vsize = {}", virt::vsize(vptr));

    if !vptr.is_null() {
        unsafe {
            *vptr = 0xCC;
            *(vptr.add(4095)) = 0xDD;
        }
        printkln!("  Write/read OK: first={:#x}, last={:#x}",
            unsafe { *vptr } as u32, unsafe { *(vptr.add(4095)) } as u32);
        virt::vfree(vptr);
        printkln!("  vfree OK");
    }

    printkln!("=== All allocation tests passed ===");
}
