/// GDT — Global Descriptor Table
///
/// The GDT defines memory segments for the CPU in protected mode.
/// Each entry is 8 bytes and describes the base address, limit,
/// and access permissions of a memory segment.
///
/// Our GDT has 7 entries:
///   0: Null descriptor (required by CPU)
///   1: Kernel Code  (0x08)
///   2: Kernel Data  (0x10)
///   3: Kernel Stack (0x18)
///   4: User Code    (0x23)
///   5: User Data    (0x2B)
///   6: User Stack   (0x33)
///
/// The GDT is placed at physical address 0x00000800 as required.

use core::arch::asm;
use crate::printkln;

/// Number of GDT entries
const GDT_ENTRIES: usize = 7;

/// GDT base address as required by subject
const GDT_BASE_ADDR: u32 = 0x00000800;

/// A single GDT entry (segment descriptor), 8 bytes.
///
/// Layout (bit fields packed into 8 bytes):
///   - Limit [0:15]        (bytes 0-1)
///   - Base  [0:15]        (bytes 2-3)
///   - Base  [16:23]       (byte 4)
///   - Access byte         (byte 5)
///   - Limit [16:19] + Flags (byte 6): low nibble = limit[16:19], high nibble = flags
///   - Base  [24:31]       (byte 7)
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct GdtEntry {
    limit_low: u16,     // Limit bits 0-15
    base_low: u16,      // Base bits 0-15
    base_mid: u8,       // Base bits 16-23
    access: u8,         // Access byte
    granularity: u8,    // Limit bits 16-19 (low nibble) + flags (high nibble)
    base_high: u8,      // Base bits 24-31
}

/// The GDTR register structure, used by `lgdt`.
#[repr(C, packed)]
pub struct GdtPointer {
    limit: u16,     // Size of GDT - 1
    base: u32,      // Linear address of GDT
}

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

    /// Creates a GDT entry from raw components.
    ///
    /// - `base`:   32-bit linear base address of the segment
    /// - `limit`:  20-bit segment limit (in units defined by granularity)
    /// - `access`: Access byte (Present, DPL, Type, etc.)
    /// - `flags`:  4-bit flags (Granularity, Size, etc.) — upper nibble of granularity byte
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

    /// Extract the full 32-bit base address.
    fn base(&self) -> u32 {
        (self.base_low as u32)
            | ((self.base_mid as u32) << 16)
            | ((self.base_high as u32) << 24)
    }

    /// Extract the full 20-bit limit.
    fn limit(&self) -> u32 {
        (self.limit_low as u32) | (((self.granularity & 0x0F) as u32) << 16)
    }

    /// Extract the 4-bit flags (upper nibble of granularity byte).
    fn flags(&self) -> u8 {
        (self.granularity >> 4) & 0x0F
    }
}

// ──────────────────────────────────────────────
//  Access byte bits
// ──────────────────────────────────────────────
//
//  Bit 7    : Present (P)         — 1 = segment is present in memory
//  Bit 6-5  : DPL (Descriptor Privilege Level) — 0 = kernel, 3 = user
//  Bit 4    : Descriptor type (S) — 1 = code/data segment, 0 = system segment
//  Bit 3    : Executable (E)      — 1 = code, 0 = data
//  Bit 2    : Direction/Conforming
//               Data: 0=grows up, 1=grows down
//               Code: 0=non-conforming, 1=conforming
//  Bit 1    : Readable/Writable
//               Code: 1=readable
//               Data: 1=writable
//  Bit 0    : Accessed (A)        — CPU sets this, init to 0

/// Present + DPL 0 + Code/Data + Executable + Readable
const KERNEL_CODE_ACCESS: u8 = 0b1001_1010; // 0x9A — P=1, DPL=0, S=1, E=1, RW=1
/// Present + DPL 0 + Code/Data + Writable
const KERNEL_DATA_ACCESS: u8 = 0b1001_0010; // 0x92 — P=1, DPL=0, S=1, E=0, RW=1
/// Present + DPL 0 + Code/Data + Writable + Direction=down (grows down for stack)
const KERNEL_STACK_ACCESS: u8 = 0b1001_0110; // 0x96 — P=1, DPL=0, S=1, E=0, DC=1, RW=1
/// Present + DPL 3 + Code/Data + Executable + Readable
const USER_CODE_ACCESS: u8 = 0b1111_1010;   // 0xFA — P=1, DPL=3, S=1, E=1, RW=1
/// Present + DPL 3 + Code/Data + Writable
const USER_DATA_ACCESS: u8 = 0b1111_0010;   // 0xF2 — P=1, DPL=3, S=1, E=0, RW=1
/// Present + DPL 3 + Code/Data + Writable + Direction=down (grows down for stack)
const USER_STACK_ACCESS: u8 = 0b1111_0110;  // 0xF6 — P=1, DPL=3, S=1, E=0, DC=1, RW=1

// ──────────────────────────────────────────────
//  Flags nibble (upper nibble of granularity byte)
// ──────────────────────────────────────────────
//
//  Bit 3 (7): Granularity — 0=byte, 1=4KiB pages
//  Bit 2 (6): Size        — 0=16-bit, 1=32-bit protected mode
//  Bit 1 (5): Long mode   — 0 for 32-bit
//  Bit 0 (4): Available   — 0

/// Granularity=4KiB pages, 32-bit protected mode
const FLAGS_32BIT_4K: u8 = 0b1100; // 0xC — G=1, D/B=1

/// Initializes the GDT at address 0x00000800 and loads it.
///
/// This writes 7 segment descriptors directly to the GDT memory region,
/// then calls `lgdt` followed by a far jump to reload CS and the other
/// segment registers.
pub fn init() {
    // Build GDT entries on the stack first
    let gdt: [GdtEntry; GDT_ENTRIES] = [
        // 0x00: Null descriptor (required)
        GdtEntry::null(),
        // 0x08: Kernel Code — base=0, limit=0xFFFFF (4GB with 4K granularity)
        GdtEntry::new(0x00000000, 0xFFFFF, KERNEL_CODE_ACCESS, FLAGS_32BIT_4K),
        // 0x10: Kernel Data — base=0, limit=0xFFFFF
        GdtEntry::new(0x00000000, 0xFFFFF, KERNEL_DATA_ACCESS, FLAGS_32BIT_4K),
        // 0x18: Kernel Stack — base=0, limit=0xFFFFF (grows down)
        GdtEntry::new(0x00000000, 0xFFFFF, KERNEL_STACK_ACCESS, FLAGS_32BIT_4K),
        // 0x20: User Code — base=0, limit=0xFFFFF
        GdtEntry::new(0x00000000, 0xFFFFF, USER_CODE_ACCESS, FLAGS_32BIT_4K),
        // 0x28: User Data — base=0, limit=0xFFFFF
        GdtEntry::new(0x00000000, 0xFFFFF, USER_DATA_ACCESS, FLAGS_32BIT_4K),
        // 0x30: User Stack — base=0, limit=0xFFFFF (grows down)
        GdtEntry::new(0x00000000, 0xFFFFF, USER_STACK_ACCESS, FLAGS_32BIT_4K),
    ];

    // Copy GDT to the required physical address 0x00000800
    unsafe {
        let src = gdt.as_ptr() as *const u8;
        let dst = GDT_BASE_ADDR as *mut u8;
        let size = GDT_ENTRIES * 8; // Each entry is 8 bytes
        let mut i = 0;
        while i < size {
            *dst.add(i) = *src.add(i);
            i += 1;
        }
    }

    // Create the GDTR pointer
    let gdt_ptr = GdtPointer {
        limit: ((GDT_ENTRIES * 8) - 1) as u16,
        base: GDT_BASE_ADDR,
    };

    // Load the GDT and reload segment registers
    unsafe {
        load_gdt(&gdt_ptr);
    }
}

/// Loads the GDTR and reloads all segment registers.
///
/// After `lgdt`, we must reload CS via a far jump (ljmp),
/// and then reload DS, ES, FS, GS, SS with the kernel data segment selector.
/// The kernel stack segment selector is loaded into SS.
unsafe fn load_gdt(gdt_ptr: &GdtPointer) {
    asm!(
        "lgdt ({gdt_ptr})",

        // Reload CS by performing a far jump
        // 0x08 = kernel code segment selector
        "ljmp $0x08, $2f",
        "2:",

        // Reload data segment registers with kernel data selector (0x10)
        "movw $0x10, %ax",
        "movw %ax, %ds",
        "movw %ax, %es",
        "movw %ax, %fs",
        "movw %ax, %gs",

        // Load kernel stack segment (0x18) into SS
        "movw $0x18, %ax",
        "movw %ax, %ss",

        gdt_ptr = in(reg) gdt_ptr as *const GdtPointer as u32,
        options(att_syntax)
    );
}

/// Prints the GDT contents in a human-readable format.
pub fn print_gdt(_args: &[u8]) {
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

    printkln!("=== Global Descriptor Table ===");
    printkln!("  GDTR: base={:#x}  limit={:#x}  ({} entries)", gdtr_base, gdtr_limit as u32, num_entries);
    printkln!("  Idx  Selector  Base        Limit       Access  Flags  Type");
    printkln!("  ---  --------  ----------  ----------  ------  -----  ----");

    let names: [&str; GDT_ENTRIES] = [
        "Null",
        "Kernel Code",
        "Kernel Data",
        "Kernel Stack",
        "User Code",
        "User Data",
        "User Stack",
    ];

    let mut i: usize = 0;
    while i < GDT_ENTRIES {
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

        printkln!(
            "  [{}]  {:#x}      {:#x}  {:#x}    {:#x}    {:#x}   {}",
            i as u32,
            selector,
            base,
            limit,
            access as u32,
            flags as u32,
            names[i]
        );

        i += 1;
    }
    printkln!("=== End GDT ===");
}
