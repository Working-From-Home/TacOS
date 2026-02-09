/// Kernel stack trace — walks the EBP chain to display call frames.
///
/// On x86 (cdecl / System V ABI), each stack frame looks like:
///
///   [ebp+8]   → first argument
///   [ebp+4]   → return address (EIP of caller)
///   [ebp]     → saved EBP (pointer to previous frame)
///
/// We walk from the current EBP upward until we hit a null EBP or
/// reach a maximum depth. Each frame shows the return address, which
/// can be mapped to function names if symbols are available.

use core::arch::asm;
use crate::printkln;

/// Maximum number of frames to walk (prevents infinite loops).
const MAX_FRAMES: usize = 20;

/// Reads the current EBP register value.
#[inline(always)]
fn get_ebp() -> u32 {
    let ebp: u32;
    unsafe {
        asm!("mov {}, ebp", out(reg) ebp);
    }
    ebp
}

/// Reads the current ESP register value.
#[inline(always)]
fn get_esp() -> u32 {
    let esp: u32;
    unsafe {
        asm!("mov {}, esp", out(reg) esp);
    }
    esp
}

/// Prints a human-friendly kernel stack trace.
///
/// Walks the frame pointer (EBP) chain and displays each frame's
/// saved EBP and return address. This is the function required by
/// the KFS-2 subject.
pub fn print_stack() {
    let esp = get_esp();
    let ebp = get_ebp();

    printkln!("=== Kernel Stack Trace ===");
    printkln!("  ESP: {:#x}\n  EBP: {:#x}\n", esp, ebp);
    printkln!("  Frame  EBP         Return Addr");
    printkln!("  -----  ----------  -----------");

    let mut current_ebp = ebp;
    let mut frame: usize = 0;

    while current_ebp != 0 && frame < MAX_FRAMES {
        // Read saved EBP and return address from the stack frame
        let saved_ebp = unsafe { *(current_ebp as *const u32) };
        let return_addr = unsafe { *((current_ebp + 4) as *const u32) };

        printkln!(
            "  [{}]    {:#x}    {:#x}",
            frame as u32,
            current_ebp,
            return_addr
        );

        // Sanity check: EBP should increase as we walk up the stack
        // (stack grows downward, so older frames have higher addresses)
        if saved_ebp != 0 && saved_ebp <= current_ebp {
            printkln!("  (frame chain broken: saved_ebp <= current_ebp)");
            break;
        }

        current_ebp = saved_ebp;
        frame += 1;
    }

    if frame == MAX_FRAMES {
        printkln!("  ... (max depth reached)");
    }

    printkln!("=== End Stack Trace ({} frames) ===", frame as u32);
}
