use crate::drivers::port;
use crate::klib::string;

static mut CURSOR_X: usize = 0;
static mut CURSOR_Y: usize = 0;

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

const VGA_PORT_COMMAND: u16 = 0x3D4;
const VGA_PORT_DATA: u16 = 0x3D5;
const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

pub const DEFAULT_COLOR: u8 = 0x0B; // LightCyan on black

#[allow(dead_code)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xA,
    LightCyan = 0xB,
    LightRed = 0xC,
    Pink = 0xD,
    Yellow = 0xE,
    White = 0xF,
}

/// Returns the color code for the given foreground and background colors.
pub fn get_color_code(fg: Color, bg: Color) -> u8 {
    ((bg as u8) << 4) | ((fg as u8) & 0x0F)
}

/// Updates the cursor position on the screen.
fn update_cursor(x: usize, y: usize) {
    let pos = (y * VGA_WIDTH + x) as u16;
    unsafe {
        port::outb(VGA_PORT_COMMAND, 0x0E);  // Higher byte
        port::outb(VGA_PORT_DATA, (pos >> 8) as u8);
        port::outb(VGA_PORT_COMMAND, 0x0F);  // Lower byte
        port::outb(VGA_PORT_DATA, (pos & 0xFF) as u8);
    }
}

/// Scrolls the screen up by one line
pub fn scroll() {
    unsafe {
        // Copies each line to the line above
        for row in 1..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                let from = ((row * VGA_WIDTH + col) * 2) as isize;
                let to = (((row - 1) * VGA_WIDTH + col) * 2) as isize;

                *VGA_BUFFER.offset(to) = *VGA_BUFFER.offset(from);
                *VGA_BUFFER.offset(to + 1) = *VGA_BUFFER.offset(from + 1);
            }
        }

        // Deletes last line
        let last_line_offset = ((VGA_HEIGHT - 1) * VGA_WIDTH * 2) as isize;
        for col in 0..VGA_WIDTH {
            *VGA_BUFFER.offset(last_line_offset + (col as isize) * 2) = b' ';
            *VGA_BUFFER.offset(last_line_offset + (col as isize) * 2 + 1) = 0xb; // couleur claire
        }

        // updates cursor position
        CURSOR_Y = VGA_HEIGHT - 1;
        CURSOR_X = 0;
    }
}

/// Prints a character to the VGA buffer at 0xb8000.
fn _putchar_core(c: u8, color: u8) {
    unsafe {
        match c {
            b'\n' => {
                CURSOR_X = 0;
                CURSOR_Y += 1;
            }
            b'\r' => {
                CURSOR_X = 0;
            }
            _ => {
                let offset = (CURSOR_Y * VGA_WIDTH + CURSOR_X) * 2;
                *VGA_BUFFER.offset(offset as isize) = c;
                *VGA_BUFFER.offset(offset as isize + 1) = color;
                CURSOR_X += 1;
                if CURSOR_X >= VGA_WIDTH {
                    CURSOR_X = 0;
                    CURSOR_Y += 1;
                }
            }
        }

        if CURSOR_Y >= VGA_HEIGHT {
            scroll();
        }

        update_cursor(CURSOR_X, CURSOR_Y);
    }
}

/// Prints a character to the VGA buffer at 0xb8000 with the default color.
pub fn putchar(c: u8) {
    _putchar_core(c, DEFAULT_COLOR);
}

/// Prints a character to the VGA buffer at 0xb8000 with a specific color.
pub fn putchar_colored(c: u8, color: u8) {
    _putchar_core(c, color);
}

/// Prints a null-terminated string to the VGA buffer at 0xb8000.
fn _putstr_core(s: *const u8, color: u8) {
    let len = string::strlen(s);
    for i in 0..len {
        unsafe {
            let c = *s.add(i);
            putchar_colored(c, color);
        }
    }
}

pub fn backspace() {
    unsafe {
        if CURSOR_X > 0 {
            CURSOR_X -= 1;
            let offset = (CURSOR_Y * VGA_WIDTH + CURSOR_X) * 2;
            *VGA_BUFFER.offset(offset as isize) = b' ';
            *VGA_BUFFER.offset(offset as isize + 1) = DEFAULT_COLOR;
            update_cursor(CURSOR_X, CURSOR_Y);
        }
    }
}

/// Prints a null-terminated string to the VGA buffer at 0xb8000 with the default color.
pub fn putstr(s: *const u8) {
    _putstr_core(s, DEFAULT_COLOR);
}

/// Prints a null-terminated string to the VGA buffer at 0xb8000 with a specific color.
pub fn putstr_colored(s: *const u8, color: u8) {
    _putstr_core(s, color);
}
