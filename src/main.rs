#![no_std]
#![no_main]

use core::panic::PanicInfo;

use tacos::{io::print, printkln, println};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    printkln!("Welcome to {} TacOS!", 42);
    tacos::gdt::init();
    tacos::shell::run();
}
