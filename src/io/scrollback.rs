/// Scrollback buffer — stores past terminal lines so the user can scroll up.
///
/// We keep a ring buffer of `SCROLLBACK_LINES` lines, each `VGA_WIDTH` chars
/// wide (character + color pairs). When the VGA scrolls a line off the top,
/// we save it here. PageUp / PageDown let the user view older output.

use crate::drivers::vga::{self, VGA_WIDTH, VGA_HEIGHT, DEFAULT_COLOR};

/// How many lines of history to keep.
const SCROLLBACK_LINES: usize = 200;

/// Each cell is (character, color).
#[derive(Copy, Clone)]
struct Cell {
    ch: u8,
    color: u8,
}

impl Cell {
    const fn blank() -> Self {
        Cell { ch: b' ', color: DEFAULT_COLOR }
    }
}

/// Ring buffer of saved lines.
static mut BUFFER: [[Cell; VGA_WIDTH]; SCROLLBACK_LINES] =
    [[Cell::blank(); VGA_WIDTH]; SCROLLBACK_LINES];

/// Index of the next line to write (ring buffer head).
static mut HEAD: usize = 0;

/// Total number of lines saved so far (saturates at SCROLLBACK_LINES).
static mut COUNT: usize = 0;

/// How many lines the user has scrolled back (0 = live view).
static mut SCROLL_OFFSET: usize = 0;

/// Pointer helper: get a pointer to BUFFER[row][col].
#[inline(always)]
unsafe fn buf_cell(row: usize, col: usize) -> *mut Cell {
    let ptr = BUFFER.as_mut_ptr() as *mut Cell;
    ptr.add(row * VGA_WIDTH + col)
}

/// Pointer helper: get a pointer to LIVE_SCREEN[row][col].
#[inline(always)]
unsafe fn live_cell(row: usize, col: usize) -> *mut Cell {
    let ptr = LIVE_SCREEN.as_mut_ptr() as *mut Cell;
    ptr.add(row * VGA_WIDTH + col)
}

/// Saves the top row of the VGA buffer into the scrollback ring buffer.
/// Called just before the VGA `scroll_buffer_up`.
pub fn save_top_line() {
    unsafe {
        let vga = 0xb8000 as *const u8;
        let mut col = 0;
        while col < VGA_WIDTH {
            let off = col * 2;
            let cell = buf_cell(HEAD, col);
            (*cell).ch = *vga.add(off);
            (*cell).color = *vga.add(off + 1);
            col += 1;
        }
        HEAD = (HEAD + 1) % SCROLLBACK_LINES;
        if COUNT < SCROLLBACK_LINES {
            COUNT += 1;
        }
        // If user was scrolled back, keep their view stable
        if SCROLL_OFFSET > 0 && SCROLL_OFFSET < COUNT {
            SCROLL_OFFSET += 1;
        }
    }
}

/// Scroll up (show older lines). Returns true if the view changed.
pub fn scroll_up(lines: usize) -> bool {
    unsafe {
        let max = COUNT;
        if max == 0 {
            return false;
        }
        let old = SCROLL_OFFSET;
        SCROLL_OFFSET += lines;
        if SCROLL_OFFSET > max {
            SCROLL_OFFSET = max;
        }
        if SCROLL_OFFSET != old {
            redraw();
            true
        } else {
            false
        }
    }
}

/// Scroll down (show newer lines). Returns true if the view changed.
pub fn scroll_down(lines: usize) -> bool {
    unsafe {
        if SCROLL_OFFSET == 0 {
            return false;
        }
        let old = SCROLL_OFFSET;
        if lines >= SCROLL_OFFSET {
            SCROLL_OFFSET = 0;
        } else {
            SCROLL_OFFSET -= lines;
        }
        if SCROLL_OFFSET != old {
            if SCROLL_OFFSET == 0 {
                restore_live();
            } else {
                redraw();
            }
            true
        } else {
            false
        }
    }
}

/// Returns true if currently viewing scrollback (not live).
pub fn is_scrolled_back() -> bool {
    unsafe { SCROLL_OFFSET > 0 }
}

/// Saves the current live VGA screen so we can restore it later.
static mut LIVE_SCREEN: [[Cell; VGA_WIDTH]; VGA_HEIGHT] =
    [[Cell::blank(); VGA_WIDTH]; VGA_HEIGHT];

pub fn save_live_screen() {
    unsafe {
        let vga = 0xb8000 as *const u8;
        let mut row = 0;
        while row < VGA_HEIGHT {
            let mut col = 0;
            while col < VGA_WIDTH {
                let off = (row * VGA_WIDTH + col) * 2;
                let cell = live_cell(row, col);
                (*cell).ch = *vga.add(off);
                (*cell).color = *vga.add(off + 1);
                col += 1;
            }
            row += 1;
        }
    }
}

fn restore_live() {
    unsafe {
        let vga = 0xb8000 as *mut u8;
        let mut row = 0;
        while row < VGA_HEIGHT {
            let mut col = 0;
            while col < VGA_WIDTH {
                let off = (row * VGA_WIDTH + col) * 2;
                let cell = live_cell(row, col);
                *vga.add(off) = (*cell).ch;
                *vga.add(off + 1) = (*cell).color;
                col += 1;
            }
            row += 1;
        }
        // Restore cursor
        let (cx, cy) = crate::io::cursor::get_pos();
        vga::update_cursor(cx, cy);
    }
}

/// Redraws the VGA screen from the scrollback buffer.
fn redraw() {
    unsafe {
        let vga = 0xb8000 as *mut u8;

        let mut row = 0;
        while row < VGA_HEIGHT {
            let lines_from_bottom = VGA_HEIGHT - 1 - row;
            let sb_offset = SCROLL_OFFSET - 1 + lines_from_bottom;

            if sb_offset < COUNT {
                let idx = (HEAD + SCROLLBACK_LINES - 1 - sb_offset) % SCROLLBACK_LINES;
                let mut col = 0;
                while col < VGA_WIDTH {
                    let off = (row * VGA_WIDTH + col) * 2;
                    let cell = buf_cell(idx, col);
                    *vga.add(off) = (*cell).ch;
                    *vga.add(off + 1) = (*cell).color;
                    col += 1;
                }
            } else {
                let mut col = 0;
                while col < VGA_WIDTH {
                    let off = (row * VGA_WIDTH + col) * 2;
                    *vga.add(off) = b' ';
                    *vga.add(off + 1) = DEFAULT_COLOR;
                    col += 1;
                }
            }
            row += 1;
        }

        // Hide cursor while viewing scrollback
        vga::update_cursor(VGA_WIDTH, VGA_HEIGHT);
    }
}
