/// klog — Kernel log ring buffer for `dmesg`.
///
/// Every byte written through `printk` is saved in that buffer.
///
/// `dmesg` dumps the current contents to the VGA screen.
/// printk is pretty hard to implement (concurrency/deadlocks, reentrant calls, latency, crash -> ringbuffer missing messages, interfering with normal operations...)
/// but since TacOS is single-threaded and doesn't have interrupts yet, we can get away with a very simple implementation.
/// See this conference to understand the complexities of a real printk implementation
///  : https://www.youtube.com/watch?v=saPQZ_tnxwE

use crate::io::display;

/// Ring buffer sized to one full VGA screen (25 rows * 80 cols) - 1 for cursor line.
/// (The console doesn't support scrollback, useless to keep more data than what fits on screen.
const KLOG_BUF_SIZE: usize = 24 * 80;

static mut BUF: [u8; KLOG_BUF_SIZE] = [0; KLOG_BUF_SIZE];
static mut HEAD: usize = 0; // Write cursor — next position to write into.
static mut TOTAL: usize = 0; // Total bytes ever written (to detect wrap-around).

// ──────────────────────────────────────────────
//  Write API (called from printk)
// ──────────────────────────────────────────────

/// Append a single byte to the kernel log buffer.
#[inline]
pub fn log_byte(c: u8) {
    unsafe {
        *BUF.as_mut_ptr().add(HEAD) = c;
        HEAD += 1;
        if HEAD >= KLOG_BUF_SIZE {
            HEAD = 0;
        }
        TOTAL += 1;
    }
}

/// Append a string slice to the kernel log buffer.
pub fn log_str(s: &str) {
    let ptr = s.as_ptr();
    let len = s.len();
    let mut i: usize = 0;
    while i < len {
        unsafe {
            log_byte(*ptr.add(i));
        }
        i += 1;
    }
}

/// Append a byte slice to the kernel log buffer.
pub fn log_bytes(bytes: &[u8]) {
    let ptr = bytes.as_ptr();
    let len = bytes.len();
    let mut i: usize = 0;
    while i < len {
        unsafe {
            log_byte(*ptr.add(i));
        }
        i += 1;
    }
}

// ──────────────────────────────────────────────
//  Read API (called by dmesg)
// ──────────────────────────────────────────────

/// Dump the entire kernel log to the VGA display.
///
/// If the buffer has not yet wrapped, we print `BUF[0..HEAD]`.
/// If it has wrapped, we print from the oldest data (`HEAD`) forward through the ring, covering `KLOG_BUF_SIZE` bytes.
pub fn dump() {
    unsafe {
        if TOTAL <= KLOG_BUF_SIZE {
            // No wrap — everything is in BUF[0..HEAD]
            let mut i: usize = 0;
            while i < HEAD {
                display::put_char(*BUF.as_ptr().add(i));
                i += 1;
            }
        } else {
            // Wrapped — oldest byte is at HEAD, read the full ring
            let mut i: usize = 0;
            while i < KLOG_BUF_SIZE {
                let idx = (HEAD + i) % KLOG_BUF_SIZE;
                display::put_char(*BUF.as_ptr().add(idx));
                i += 1;
            }
        }
    }
}

/// Clear the kernel log buffer.
pub fn clear() {
    unsafe {
        HEAD = 0;
        TOTAL = 0;
    }
}
