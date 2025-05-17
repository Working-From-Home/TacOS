#![no_std]
#![no_main]

mod klib;
mod io;
mod drivers;

use core::panic::PanicInfo;

use drivers::keyboard;
use drivers::vga;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    let msg = b"42\0";
    vga::putstr(msg.as_ptr());
    
    loop {
        unsafe {
            if let Some(scancode) = keyboard::read_scancode() {
                let ascii = keyboard::scancode_to_ascii(scancode);
                if let Some(c) = ascii {
                    vga::putchar(c as u8);
                }
            }
        }
    }
}
