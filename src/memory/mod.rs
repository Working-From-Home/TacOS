/// Memory management subsystem for TacOS.
///
/// Provides:
/// - Physical frame allocator (bitmap-based, 4KB frames)
/// - Paging (page directory + page tables, identity mapping)
/// - Kernel heap (kmalloc/kfree/ksize/kbrk)
/// - Virtual memory (vmalloc/vfree/vsize/vbrk)
/// - Kernel/User space separation

pub mod frame;
pub mod paging;
pub mod heap;
pub mod virt;

use crate::printkln;

// ──────────────────────────────────────────────
//  Constants
// ──────────────────────────────────────────────

/// Size of a page/frame (4 KB)
pub const PAGE_SIZE: u32 = 4096;

/// Kernel space starts at 0xC0000000 (upper 1GB) — conceptual boundary.
/// In practice, our kernel is identity-mapped at 1MB, but this defines
/// the conceptual split for future process isolation.
pub const KERNEL_SPACE_START: u32 = 0xC0000000;

/// User space: 0x00000000 - 0xBFFFFFFF (lower 3GB)
pub const USER_SPACE_END: u32 = 0xBFFFFFFF;

// ──────────────────────────────────────────────
//  Linker-provided symbols
// ──────────────────────────────────────────────

extern "C" {
    static __kernel_start: u8;
    static __kernel_end: u8;
    static __bss_start: u8;
    static __bss_end: u8;
}

/// Returns the physical address where the kernel starts (1MB).
pub fn kernel_start() -> u32 {
    unsafe { &__kernel_start as *const u8 as u32 }
}

/// Returns the physical address where the kernel ends.
pub fn kernel_end() -> u32 {
    unsafe { &__kernel_end as *const u8 as u32 }
}

/// Returns the kernel size in bytes.
pub fn kernel_size() -> u32 {
    kernel_end() - kernel_start()
}

// ──────────────────────────────────────────────
//  Multiboot info structures
// ──────────────────────────────────────────────

/// Multiboot info structure passed by GRUB (Multiboot 1).
/// We only define the fields we use.
#[repr(C, packed)]
struct MultibootInfo {
    flags: u32,           // offset 0
    mem_lower: u32,       // offset 4  (KB below 1MB, valid if flags bit 0)
    mem_upper: u32,       // offset 8  (KB above 1MB, valid if flags bit 0)
    boot_device: u32,     // offset 12
    cmdline: u32,         // offset 16
    mods_count: u32,      // offset 20
    mods_addr: u32,       // offset 24
    syms: [u32; 4],       // offset 28-43
    mmap_length: u32,     // offset 44 (valid if flags bit 6)
    mmap_addr: u32,       // offset 48 (valid if flags bit 6)
}

/// Multiboot memory map entry.
#[repr(C, packed)]
struct MultibootMmapEntry {
    size: u32,            // size of the rest of this entry
    base_addr_low: u32,
    base_addr_high: u32,
    length_low: u32,
    length_high: u32,
    entry_type: u32,      // 1 = usable RAM
}

/// Total physical memory detected (in bytes).
static mut TOTAL_MEMORY: u32 = 0;

/// Returns the total detected physical memory in bytes.
pub fn total_memory() -> u32 {
    unsafe { TOTAL_MEMORY }
}

// ──────────────────────────────────────────────
//  Initialization
// ──────────────────────────────────────────────

/// Initialize the entire memory subsystem.
///
/// Call order:
/// 1. Parse multiboot info to detect available memory
/// 2. Initialize the physical frame allocator
/// 3. Set up paging (identity map)
/// 4. Initialize the kernel heap
/// 5. Initialize the virtual memory allocator
pub fn init(multiboot_info_addr: u32) {
    printkln!("=== Memory Subsystem Init ===");

    // Parse multiboot info
    let (mem_lower, mem_upper) = parse_multiboot_info(multiboot_info_addr);

    printkln!("  Kernel: {:#x} - {:#x} ({} KB)",
        kernel_start(), kernel_end(), kernel_size() / 1024);
    printkln!("  Lower memory: {} KB", mem_lower);
    printkln!("  Upper memory: {} KB", mem_upper);

    // Total memory = lower + upper + 1MB (the hole between them)
    // mem_lower is KB below 1MB (typically 640)
    // mem_upper is KB above 1MB
    let total_kb = mem_lower + mem_upper + 1024;
    unsafe { TOTAL_MEMORY = total_kb * 1024; }
    printkln!("  Total memory: {} KB ({} MB)", total_kb, total_kb / 1024);

    // Initialize frame allocator
    frame::init(multiboot_info_addr);

    // Initialize paging
    paging::init();

    // Initialize kernel heap
    heap::init();

    // Initialize virtual memory allocator
    virt::init();

    printkln!("=== Memory Init Complete ===");
    printkln!();
}

/// Parse the multiboot info structure to get memory information.
/// Returns (mem_lower_kb, mem_upper_kb).
fn parse_multiboot_info(addr: u32) -> (u32, u32) {
    if addr == 0 {
        // No multiboot info, assume 128MB
        printkln!("  [WARN] No multiboot info, assuming 128MB RAM");
        return (640, 130048);
    }

    let info = addr as *const MultibootInfo;
    let flags = unsafe { (*info).flags };

    let mut mem_lower: u32 = 640;   // default
    let mut mem_upper: u32 = 130048; // default (128MB - 1MB - 640KB, roughly)

    // Check if memory info is available (flags bit 0)
    if flags & 0x1 != 0 {
        mem_lower = unsafe { (*info).mem_lower };
        mem_upper = unsafe { (*info).mem_upper };
    }

    (mem_lower, mem_upper)
}

/// Walk the multiboot memory map and call the callback for each usable region.
/// callback receives (base_addr: u32, length: u32)
pub fn walk_memory_map(multiboot_info_addr: u32, mut callback: impl FnMut(u32, u32)) {
    if multiboot_info_addr == 0 {
        // No multiboot info — assume 0-640KB and 1MB-128MB are usable
        callback(0, 640 * 1024);
        callback(0x100000, 127 * 1024 * 1024);
        return;
    }

    let info = multiboot_info_addr as *const MultibootInfo;
    let flags = unsafe { (*info).flags };

    // Check if memory map is available (flags bit 6)
    if flags & (1 << 6) != 0 {
        let mmap_addr = unsafe { (*info).mmap_addr };
        let mmap_length = unsafe { (*info).mmap_length };

        let mut offset: u32 = 0;
        while offset < mmap_length {
            let entry = (mmap_addr + offset) as *const MultibootMmapEntry;
            let size = unsafe { (*entry).size };
            let base_low = unsafe { (*entry).base_addr_low };
            let base_high = unsafe { (*entry).base_addr_high };
            let len_low = unsafe { (*entry).length_low };
            let etype = unsafe { (*entry).entry_type };

            // Only use entries with base in 32-bit range
            if etype == 1 && base_high == 0 {
                callback(base_low, len_low);
            }

            offset += size + 4;
        }
    } else if flags & 0x1 != 0 {
        // Fall back to mem_lower/mem_upper
        let mem_lower = unsafe { (*info).mem_lower };
        let mem_upper = unsafe { (*info).mem_upper };
        callback(0, mem_lower * 1024);
        callback(0x100000, mem_upper * 1024);
    } else {
        // No memory info at all, assume defaults
        callback(0, 640 * 1024);
        callback(0x100000, 127 * 1024 * 1024);
    }
}

// ──────────────────────────────────────────────
//  Alignment helpers
// ──────────────────────────────────────────────

/// Align address up to the next page boundary.
pub fn align_up(addr: u32, align: u32) -> u32 {
    (addr + align - 1) & !(align - 1)
}

/// Align address down to the previous page boundary.
pub fn align_down(addr: u32, align: u32) -> u32 {
    addr & !(align - 1)
}
