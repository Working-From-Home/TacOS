/// Global Descriptor Table (GDT)
/// 
/// 
/// The GDT is a fundamental data structure in x86 protected mode that defines memory segments.
/// Even though modern operating systems use paging for memory management and isolation, the GDT
/// is still required to set up the CPU's segmentation model.
/// 
/// Each entry in the GDT describes a segment's base address, limit, and access rights. The CPU
/// uses this information to translate logical addresses into linear addresses and to enforce
/// access control based on privilege levels.
/// 
/// A segment descriptor is packed into 64 bits as follows:
/// 
///   63               56 55      52 51          48 47                 40 39                   32
///  +-------------------+----------+--------------+---------------------+-----------------------+
///  |    Base[31:24]    |  Flags   | Limit[19:16] |        Access       |      Base [23:16]     |
///  +-------------------+----------+--------------+---------------------+-----------------------+
///  +---------------------------------------------+---------------------------------------------+
///  |                 Base [15:0]                 |                 Limit [15:0]                |
///  +---------------------------------------------+---------------------------------------------+
///   31                                         16 15                                          0
/// 
/// Field breakdown:
/// 
/// - Base (32 bits, split across bytes 2-4 and 7):
///     Defines the linear base address of the segment.
/// 
/// - Limit (20 bits, split across bytes 0-1 and 6):
///     Defines the maximum offset allowed within the segment.
///     If the flags Granularity bit is set, limit is in 4KB pages instead of bytes.
/// 
/// - Access byte (8 bits, byte 5):
///     7   (P)     Present                     Must be 1 for a valid segment
///     6–5 (DPL)   Descriptor Privilege Level  0 = ring 0 (kernel), 3 = ring 3 (user)
///     4   (S)     Descriptor type             1 = code/data segment, 0 = system segment
///     3   (E)     Executable                  1 = code segment, 0 = data segment
///     2   (DC)    Direction / Conforming      Data: 0 = grows up, 1 = grows down
///                                             Code: 0 = non-conforming, 1 = conforming
///     1   (RW)    Readable / Writable         Data: 0 = non-writable, 1 = writable
///                                             Code: 0 = non-readable, 1 = readable
///     0   (A)     Accessed                    Set by CPU on access (initialize to 0)
///
/// - Flags (4 bits, bits 52–55):
///     3   (G)     Granularity                 0 = limit in bytes, 1 = limit in 4KB pages
///     2   (D/B)   Default operand size        0 = 16-bit segment, 1 = 32-bit segment
///     1   (L)     Long mode (IA-32e only)     0 = disabled (protected mode), 1 = 64-bit code segment
///     0   (AVL)   Available for software      Ignored by the CPU
/// 
/// This GDT contains 7 entries at physical address 0x00000800:
///     0x00: Null descriptor (mandatory)
///     0x08: Kernel Code
///     0x10: Kernel Data
///     0x18: Kernel Stack
///     0x20: User Code
///     0x28: User Data
///     0x30: User Stack

use core::arch::asm;
use crate::{printkln, println};

/// -----------------------
/// GDT Constants
/// -----------------------

/// Number of GDT entries
const GDT_ENTRIES: usize = 7;

/// GDT physical address
const GDT_BASE_ADDR: u32 = 0x00000800;

/// Access bytes values for different segment types (P/DPL/S/E/DC/RW/A):
const KERNEL_CODE_ACCESS:   u8 = 0b1001_1010; // 0x9A — P=1, DPL=0, S=1, E=1, RW=1
const KERNEL_DATA_ACCESS:   u8 = 0b1001_0010; // 0x92 — P=1, DPL=0, S=1, E=0, RW=1
const KERNEL_STACK_ACCESS:  u8 = 0b1001_0110; // 0x96 — P=1, DPL=0, S=1, E=0, DC=1, RW=1
const USER_CODE_ACCESS:     u8 = 0b1111_1010; // 0xFA — P=1, DPL=3, S=1, E=1, RW=1
const USER_DATA_ACCESS:     u8 = 0b1111_0010; // 0xF2 — P=1, DPL=3, S=1, E=0, RW=1
const USER_STACK_ACCESS:    u8 = 0b1111_0110; // 0xF6 — P=1, DPL=3, S=1, E=0, DC=1, RW=1

/// Flags for 32-bit protected mode segments with 4KB granularity
const FLAGS_32BIT_4K: u8 = 0b1100;

/// -----------------------
/// GDT Data Structures
/// -----------------------

/// 8-byte GDT segment descriptor
/// 
/// Must be packed to match the CPU layout
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

/// GDTR structure for the `lgdt` instruction
/// 
/// Must be packed and aligned as required by the CPU
/// - `limit` : size of the GDT in bytes minus 1
/// - `base`  : linear base address of the GDT
#[repr(C, packed)]
pub struct GdtPointer {
    limit: u16,
    base: u32,
}

/// -----------------------
/// GDT Functions
/// -----------------------

impl GdtEntry {
    /// Creates a null GDT entry.
    const fn null() -> Self {
        GdtEntry {
            limit_low: 0,
            base_low: 0,
            base_mid: 0,
            access: 0,
            granularity: 0,
            base_high: 0,
        }
    }

    /// Creates a GDT entry.
    const fn new(base: u32, limit: u32, access: u8, flags: u8) -> Self {
        GdtEntry {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_mid: ((base >> 16) & 0xFF) as u8,
            access,
            granularity: ((limit >> 16) & 0x0F) as u8 | ((flags & 0x0F) << 4),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }

    /// Extracts the full 32-bit base address.
    fn base(&self) -> u32 {
        (self.base_low as u32) | ((self.base_mid as u32) << 16) | ((self.base_high as u32) << 24)
    }

    /// Extracts the full 20-bit limit.
    fn limit(&self) -> u32 {
        (self.limit_low as u32) | (((self.granularity & 0x0F) as u32) << 16)
    }

    /// Extracts the 4-bit flags.
    fn flags(&self) -> u8 {
        (self.granularity >> 4) & 0x0F
    }
}


/// GDT initialization function
///
/// Creates 7 segment descriptors, copies them to physical address 0x00000800,
/// and reloads the GDTR and segment registers.
pub fn init() {
    printkln!("Initializing GDT...");
    
    let gdt: [GdtEntry; GDT_ENTRIES] = [
        GdtEntry::null(),
        GdtEntry::new(0x00000000, 0xFFFFF, KERNEL_CODE_ACCESS, FLAGS_32BIT_4K),
        GdtEntry::new(0x00000000, 0xFFFFF, KERNEL_DATA_ACCESS, FLAGS_32BIT_4K),
        GdtEntry::new(0x00000000, 0xFFFFF, KERNEL_STACK_ACCESS, FLAGS_32BIT_4K),
        GdtEntry::new(0x00000000, 0xFFFFF, USER_CODE_ACCESS, FLAGS_32BIT_4K),
        GdtEntry::new(0x00000000, 0xFFFFF, USER_DATA_ACCESS, FLAGS_32BIT_4K),
        GdtEntry::new(0x00000000, 0xFFFFF, USER_STACK_ACCESS, FLAGS_32BIT_4K),
    ];

    unsafe {
        let src = gdt.as_ptr() as *const u8;
        let dst = GDT_BASE_ADDR as *mut u8;
        let size = GDT_ENTRIES * 8;
        let mut i = 0;
        while i < size {
            *dst.add(i) = *src.add(i);
            i += 1;
        }
    }

    let gdt_ptr = GdtPointer {
        limit: ((GDT_ENTRIES * 8) - 1) as u16,
        base: GDT_BASE_ADDR,
    };

    unsafe {
        load_gdt(&gdt_ptr);
    }

    printkln!("GDT initialized successfully.");
}

/// Loads the GDT into the CPU and reloads all segment registers.
///
/// This function performs the critical steps required after defining a new GDT:
///
/// 1. **Load GDTR**: Executes `lgdt` to load the GDT base address and limit into
///    the CPU's GDTR register.
///
/// 2. **Reload CS**: Performs a far jump (`ljmp`) to reload the Code Segment register.
///    This is mandatory because CS cannot be directly modified with `mov`.
///    The far jump forces the CPU to fetch the new CS descriptor from the GDT.
///
/// 3. **Reload Data Segments**: Updates DS, ES, FS, GS with the kernel data selector (0x10).
///    These registers cache segment descriptors and must be explicitly reloaded.
///
/// 4. **Reload Stack Segment**: Updates SS with the kernel stack selector (0x18).
///
/// After this function completes, the CPU is running in protected mode with all
/// segment registers pointing to the appropriate GDT entries. The kernel operates
/// in ring 0 with flat memory model (all segments span 0-4GB).
///
/// # Safety
///
/// This function is unsafe because:
/// - It directly manipulates CPU segment registers via inline assembly
/// - Invalid selectors or GDT configuration will cause a General Protection Fault
/// - Must only be called during kernel initialization with interrupts disabled
unsafe fn load_gdt(gdt_ptr: &GdtPointer) {
    asm!(
        // Load the GDTR with the address of our GDT
        "lgdt ({gdt_ptr})",

        // Reload CS with the new GDT's kernel code segment (0x08)
        "ljmp $0x08, $2f",
        "2:",

        // Reload data segment registers with kernel data segment selector (0x10)
        "movw $0x10, %ax",
        "movw %ax, %ds",
        "movw %ax, %es",
        "movw %ax, %fs",
        "movw %ax, %gs",

        // Load kernel stack segment selector (0x18) into SS
        "movw $0x18, %ax",
        "movw %ax, %ss",

        // Pass the GdtPointer address as a 32-bit register input to the assembly block
        gdt_ptr = in(reg) gdt_ptr as *const GdtPointer as u32,
        // Use AT&T syntax for the inline assembly (GAS-compatible)
        options(att_syntax)
    );
}

/// Prints the GDT contents in a human-readable format.
pub fn print_gdt() {
    // Read back the actual GDTR to verify it's loaded correctly
    let mut gdtr_buf: [u8; 6] = [0; 6];
    unsafe {
        asm!(
            "sgdt ({buf})",
            buf = in(reg) gdtr_buf.as_mut_ptr() as u32,
            options(att_syntax, nostack)
        );
    }
    let gdtr_limit = (gdtr_buf[0] as u16) | ((gdtr_buf[1] as u16) << 8);
    let gdtr_base = (gdtr_buf[2] as u32)
        | ((gdtr_buf[3] as u32) << 8)
        | ((gdtr_buf[4] as u32) << 16)
        | ((gdtr_buf[5] as u32) << 24);
    let num_entries = ((gdtr_limit as u32) + 1) / 8;

    println!("=== Global Descriptor Table ===");
    println!(
        "  GDTR: base={:#x}  limit={:#x}  ({} entries)",
        gdtr_base,
        gdtr_limit as u32,
        num_entries
    );
    println!("  Idx  Selector  Base        Limit       Access  Flags  Type");
    println!("  ---  --------  ----------  ----------  ------  -----  ----");

    let names: [&str; GDT_ENTRIES] = [
        "Null",
        "Kernel Code",
        "Kernel Data",
        "Kernel Stack",
        "User Code",
        "User Data",
        "User Stack",
    ];

    for i in 0..GDT_ENTRIES {
        // Read from the GDT at its fixed address
        let entry = unsafe {
            let ptr = (GDT_BASE_ADDR as *const GdtEntry).add(i);
            *ptr
        };

        let selector = (i * 8) as u32;
        let base = entry.base();
        let limit = entry.limit();
        let access = entry.access;
        let flags = entry.flags();

        println!(
            "  [{}]  {:#x}      {:#x}  {:#x}    {:#x}    {:#x}   {}",
            i as u32,
            selector,
            base,
            limit,
            access as u32,
            flags as u32,
            names[i]
        );
    }
    println!("=== End GDT ===");
}
