#![no_std]
#![no_main]

mod klib;

use core::panic::PanicInfo;
use klib::string;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    let msg = b"42\0";
    string::putstr(msg.as_ptr());
    loop {}
}
