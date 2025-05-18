#![no_std]
#![no_main]

mod drivers;
mod shell;
mod klib;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    crate::shell::run();
}
