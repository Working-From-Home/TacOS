/// PS/2 Mouse driver — handles scroll wheel for terminal scrollback.
///
/// Initializes the PS/2 mouse with IntelliMouse scroll wheel support
/// and provides a polling interface. Only scroll wheel events are used;
/// X/Y movement is ignored.

use crate::drivers::port;

/// Mouse packet state machine.
/// IntelliMouse sends 4-byte packets; standard mouse sends 3.
static mut PACKET: [u8; 4] = [0; 4];
static mut PACKET_IDX: usize = 0;
static mut HAS_SCROLL_WHEEL: bool = false;

/// Wait until the PS/2 controller input buffer is empty (ready for a command).
fn wait_write() {
    let mut timeout: u32 = 100_000;
    while timeout > 0 {
        if port::inb(0x64) & 0x02 == 0 {
            return;
        }
        timeout -= 1;
    }
}

/// Wait until the PS/2 controller output buffer is full (data available).
fn wait_read() {
    let mut timeout: u32 = 100_000;
    while timeout > 0 {
        if port::inb(0x64) & 0x01 != 0 {
            return;
        }
        timeout -= 1;
    }
}

/// Send a command to the mouse (through the PS/2 controller).
fn mouse_write(cmd: u8) {
    wait_write();
    port::outb(0x64, 0xD4); // tell controller: next byte goes to mouse
    wait_write();
    port::outb(0x60, cmd);
}

/// Read a byte from the mouse (ACK or data).
fn mouse_read() -> u8 {
    wait_read();
    port::inb(0x60)
}

/// Send a command and wait for ACK (0xFA).
fn mouse_cmd(cmd: u8) {
    mouse_write(cmd);
    let _ack = mouse_read(); // consume ACK
}

/// Set sample rate (needed for IntelliMouse magic sequence).
fn set_sample_rate(rate: u8) {
    mouse_cmd(0xF3); // "Set Sample Rate"
    mouse_cmd(rate);
}

/// Initialize the PS/2 mouse with scroll wheel support.
pub fn init() {
    // Enable the auxiliary (mouse) PS/2 port
    wait_write();
    port::outb(0x64, 0xA8);

    // Enable IRQ12 and IRQ1 in the controller config byte
    wait_write();
    port::outb(0x64, 0x20); // read config byte
    wait_read();
    let config = port::inb(0x60);
    wait_write();
    port::outb(0x64, 0x60); // write config byte
    wait_write();
    port::outb(0x60, config | 0x02); // bit 1 = enable IRQ12 (mouse)

    // Reset mouse to defaults
    mouse_cmd(0xFF);
    let _self_test = mouse_read(); // 0xAA = passed
    let _mouse_id = mouse_read();  // 0x00 = standard

    // IntelliMouse magic sequence: set sample rate 200, 100, 80
    // This switches the mouse from 3-byte to 4-byte packets with scroll
    set_sample_rate(200);
    set_sample_rate(100);
    set_sample_rate(80);

    // Read device ID to confirm scroll wheel
    mouse_cmd(0xF2); // "Get Device ID"
    let device_id = mouse_read();
    unsafe {
        HAS_SCROLL_WHEEL = device_id == 3 || device_id == 4;
    }

    // Enable data reporting
    mouse_cmd(0xF4);

    // Flush any leftover data
    let mut flush = 0;
    while flush < 16 {
        let status = port::inb(0x64);
        if status & 0x01 != 0 {
            let _ = port::inb(0x60);
        }
        flush += 1;
    }
}

/// Scroll direction returned by the mouse poll.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScrollEvent {
    Up,
    Down,
}

/// Check if there's mouse data available and process it.
/// Returns a scroll event if a complete packet with scroll info is ready.
pub fn poll() -> Option<ScrollEvent> {
    let status = port::inb(0x64);

    // Bit 0 = output buffer full, bit 5 = data from auxiliary port (mouse)
    if status & 0x21 != 0x21 {
        return None;
    }

    let byte = port::inb(0x60);

    unsafe {
        let packet_size: usize = if HAS_SCROLL_WHEEL { 4 } else { 3 };

        // Byte 0 must have bit 3 set (always-1 bit in PS/2 protocol)
        // Use this to resync if we get out of alignment
        if PACKET_IDX == 0 && (byte & 0x08) == 0 {
            return None; // not a valid first byte, skip
        }

        *PACKET.as_mut_ptr().add(PACKET_IDX) = byte;
        PACKET_IDX += 1;

        if PACKET_IDX >= packet_size {
            PACKET_IDX = 0;

            // Only process scroll if we have a scroll wheel
            if HAS_SCROLL_WHEEL {
                let z = *PACKET.as_ptr().add(3) as i8;
                if z < 0 {
                    return Some(ScrollEvent::Up);   // scroll wheel up
                } else if z > 0 {
                    return Some(ScrollEvent::Down);  // scroll wheel down
                }
            }
        }
    }

    None
}
