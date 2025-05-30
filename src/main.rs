#![no_std]
#![no_main]

#![allow(dead_code)]    // temporary solution to avoid warnings for unused functions

mod drivers;
mod shell;
mod io;
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
