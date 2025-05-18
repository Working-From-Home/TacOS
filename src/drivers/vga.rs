use crate::drivers::port;

pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;

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

/// Draws a character at the specified (x, y) position with the given color.
pub fn draw_char_at(x: usize, y: usize, c: u8, color: u8) {
    unsafe {
        let offset = (y * VGA_WIDTH + x) * 2;
        *VGA_BUFFER.offset(offset as isize) = c;
        *VGA_BUFFER.offset(offset as isize + 1) = color;
    }
}

/// Clears the character at the specified (x, y) position.
pub fn clear_char_at(x: usize, y: usize) {
    draw_char_at(x, y, b' ', DEFAULT_COLOR);
}

/// Updates the cursor position on the screen.
pub fn update_cursor(x: usize, y: usize) {
    let pos = (y * VGA_WIDTH + x) as u16;
    port::outb(VGA_PORT_COMMAND, 0x0E);  // Higher byte
    port::outb(VGA_PORT_DATA, (pos >> 8) as u8);
    port::outb(VGA_PORT_COMMAND, 0x0F);  // Lower byte
    port::outb(VGA_PORT_DATA, (pos & 0xFF) as u8);
}

/// Shifts all lines up by one, and clears the last line.
pub fn scroll_buffer_up() {
    unsafe {
        for row in 1..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                let from = ((row * VGA_WIDTH + col) * 2) as isize;
                let to = (((row - 1) * VGA_WIDTH + col) * 2) as isize;

                *VGA_BUFFER.offset(to) = *VGA_BUFFER.offset(from);
                *VGA_BUFFER.offset(to + 1) = *VGA_BUFFER.offset(from + 1);
            }
        }

        let last_line_offset = ((VGA_HEIGHT - 1) * VGA_WIDTH * 2) as isize;
        for col in 0..VGA_WIDTH {
            *VGA_BUFFER.offset(last_line_offset + (col as isize) * 2) = b' ';
            *VGA_BUFFER.offset(last_line_offset + (col as isize) * 2 + 1) = DEFAULT_COLOR;
        }
    }
}
