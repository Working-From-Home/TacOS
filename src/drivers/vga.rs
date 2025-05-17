use crate::klib::string::strlen;

static mut VGA_INDEX: usize = 0;

/// Prints a character to the VGA buffer at 0xb8000.
pub fn putchar(c: u8) {
    let vga_buffer = 0xb8000 as *mut u8;
    unsafe {
        *vga_buffer.offset((VGA_INDEX * 2) as isize) = c;
        *vga_buffer.offset((VGA_INDEX * 2 + 1) as isize) = 0xb; // TODO set color
        VGA_INDEX += 1;
    }
}

/// Prints a null-terminated string to the VGA buffer at 0xb8000.
pub fn putstr(s: *const u8) {
    let len = strlen(s);
    for i in 0..len {
        unsafe {
            let c = *s.add(i);
            putchar(c);
        }
    }
}