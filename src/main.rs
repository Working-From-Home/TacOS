#![no_std]
#![no_main]

#![allow(dead_code)]    // temporary solution to avoid warnings for unused functions

mod drivers;
mod gdt;
mod memory;
mod panic;
mod shell;
mod io;
mod klib;

use core::panic::PanicInfo;

#[panic_handler]
fn rust_panic(info: &PanicInfo) -> ! {
    // Disable interrupts
    unsafe { core::arch::asm!("cli"); }

    io::display::set_color(0x4F); // White on Red

    printkln!();
    printkln!("!!! RUST PANIC !!!");
    if let Some(location) = info.location() {
        printkln!("  at {}:{}", location.file(), location.line());
    }

    klib::stack::print_stack(&[]);

    io::display::set_color(0x4F);
    printkln!();
    printkln!("System halted.");

    loop {
        unsafe { core::arch::asm!("cli; hlt"); }
    }
}

#[no_mangle]
pub extern "C" fn rust_main(multiboot_info_addr: u32) -> ! {
    gdt::init();
    memory::init(multiboot_info_addr);
    crate::shell::run();
}
