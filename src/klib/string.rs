/// Prints a null-terminated string to the VGA buffer at 0xb8000.
pub fn putstr(s: *const u8) {
    let vga_buffer = 0xb8000 as *mut u8;
    let len = strlen(s);
    for i in 0..len {
        unsafe {
            *vga_buffer.offset((i as isize) * 2) = *s.add(i);
            *vga_buffer.offset((i as isize) * 2 + 1) = 0xb;
        }
    }
}

/// Returns the length of a null-terminated string (C-style).
pub fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    unsafe {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}
