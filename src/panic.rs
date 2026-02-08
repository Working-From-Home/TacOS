/// Kernel panic handling — fatal and non-fatal error reporting.
///
/// Two levels of panic:
/// - Fatal (`kernel_panic!`): prints error in red, dumps stack, halts CPU
/// - Warning (`kernel_warn!`): prints warning in yellow, continues execution

use crate::io::display;
use crate::klib::stack;

/// Color codes for panic output
const PANIC_COLOR: u8 = 0x4F;  // White on Red
const WARN_COLOR: u8 = 0x0E;   // Yellow on Black

/// Internal function called by the kernel_panic! macro.
/// Prints the panic message, location, and stack trace, then halts.
#[inline(never)]
pub fn _kernel_panic(msg: &str, file: &str, line: u32) -> ! {
    // Disable interrupts immediately
    unsafe { core::arch::asm!("cli"); }

    let prev_color = display::get_color();
    let _ = prev_color; // won't restore, we're halting

    display::set_color(PANIC_COLOR);

    crate::printkln!();
    crate::printkln!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    crate::printkln!("!!!        KERNEL PANIC             !!!");
    crate::printkln!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    crate::printkln!();
    crate::printkln!("  {}", msg);
    crate::printkln!("  at {}:{}", file, line);
    crate::printkln!();

    // Print stack trace
    stack::print_stack(&[]);

    display::set_color(PANIC_COLOR);
    crate::printkln!();
    crate::printkln!("System halted.");

    loop {
        unsafe { core::arch::asm!("cli; hlt"); }
    }
}

/// Internal function called by the kernel_warn! macro.
pub fn _kernel_warn(msg: &str, file: &str, line: u32) {
    let prev_color = display::get_color();
    display::set_color(WARN_COLOR);
    crate::printkln!("[WARN] {} ({}:{})", msg, file, line);
    display::set_color(prev_color);
}

/// Fatal kernel panic — prints message, stack trace, halts CPU.
///
/// # Examples
/// ```
/// kernel_panic!("Out of memory");
/// kernel_panic!("Invalid page table entry");
/// ```
#[macro_export]
macro_rules! kernel_panic {
    ($msg:expr) => {
        $crate::panic::_kernel_panic($msg, file!(), line!())
    };
}

/// Non-fatal kernel warning — prints message, continues execution.
///
/// # Examples
/// ```
/// kernel_warn!("Low memory");
/// ```
#[macro_export]
macro_rules! kernel_warn {
    ($msg:expr) => {
        $crate::panic::_kernel_warn($msg, file!(), line!())
    };
}
