/// Physical frame allocator — bitmap-based.
///
/// Manages physical memory in 4KB frames using a bitmap where:
/// - bit = 1: frame is USED (allocated)
/// - bit = 0: frame is FREE (available)
///
/// The bitmap covers the entire 32-bit address space (4GB = 1M frames = 128KB bitmap).
/// During init, all frames start as USED, then we mark usable regions
/// from the multiboot memory map as FREE, then re-mark the kernel
/// and low memory as USED.

use crate::{printkln, kernel_panic};
use super::{PAGE_SIZE, align_up, align_down};

// ──────────────────────────────────────────────
//  Bitmap storage
// ──────────────────────────────────────────────

/// Maximum number of physical frames (4GB / 4KB)
const MAX_FRAMES: usize = 1024 * 1024;

/// Bitmap size in bytes (1 bit per frame)
const BITMAP_SIZE: usize = MAX_FRAMES / 8; // 128 KB

/// The frame bitmap. Stored in BSS (auto-zeroed by boot.s).
/// We initialize all bits to 1 (used) in init(), then clear usable regions.
static mut BITMAP: [u8; BITMAP_SIZE] = [0u8; BITMAP_SIZE];

/// Total number of frames managed
static mut TOTAL_FRAMES: u32 = 0;

/// Number of frames currently in use
static mut USED_FRAMES: u32 = 0;

/// Multiboot info address (saved for later use)
static mut MULTIBOOT_INFO_ADDR: u32 = 0;

// ──────────────────────────────────────────────
//  Bitmap bit manipulation
// ──────────────────────────────────────────────

/// Set bit (mark frame as USED)
fn bitmap_set(frame: u32) {
    let idx = (frame / 8) as usize;
    let bit = frame % 8;
    if idx < BITMAP_SIZE {
        unsafe {
            *BITMAP.as_mut_ptr().add(idx) |= 1 << bit;
        }
    }
}

/// Clear bit (mark frame as FREE)
fn bitmap_clear(frame: u32) {
    let idx = (frame / 8) as usize;
    let bit = frame % 8;
    if idx < BITMAP_SIZE {
        unsafe {
            *BITMAP.as_mut_ptr().add(idx) &= !(1 << bit);
        }
    }
}

/// Test bit (returns true if frame is USED)
fn bitmap_test(frame: u32) -> bool {
    let idx = (frame / 8) as usize;
    let bit = frame % 8;
    if idx < BITMAP_SIZE {
        unsafe { (*BITMAP.as_ptr().add(idx) & (1 << bit)) != 0 }
    } else {
        true // out of range = used
    }
}

// ──────────────────────────────────────────────
//  Public API
// ──────────────────────────────────────────────

/// Initialize the frame allocator.
///
/// 1. Mark all frames as USED
/// 2. Walk multiboot memory map, mark usable regions as FREE
/// 3. Re-mark low memory (0-1MB) and kernel as USED
pub fn init(multiboot_info_addr: u32) {
    unsafe { MULTIBOOT_INFO_ADDR = multiboot_info_addr; }

    // Step 1: Mark all frames as used (set all bits to 1)
    let bitmap_ptr = unsafe { BITMAP.as_mut_ptr() };
    crate::klib::memory::memset(bitmap_ptr, 0xFF, BITMAP_SIZE);

    // Step 2: Walk multiboot memory map, mark usable regions as free
    let mut total_usable: u32 = 0;
    super::walk_memory_map(multiboot_info_addr, |base, length| {
        let start_frame = align_up(base, PAGE_SIZE) / PAGE_SIZE;
        let end_addr = base.saturating_add(length);
        let end_frame = align_down(end_addr, PAGE_SIZE) / PAGE_SIZE;

        let mut frame = start_frame;
        while frame < end_frame {
            bitmap_clear(frame);
            frame += 1;
        }
        total_usable += (end_frame - start_frame) * PAGE_SIZE;
    });

    // Step 3: Mark low memory (0 - 1MB) as used
    // This protects BIOS, IVT, VGA buffer, GDT, etc.
    let low_end_frame = 0x100000 / PAGE_SIZE; // frame 256
    let mut f: u32 = 0;
    while f < low_end_frame {
        bitmap_set(f);
        f += 1;
    }

    // Step 4: Mark kernel region as used
    let kernel_start_frame = super::kernel_start() / PAGE_SIZE;
    let kernel_end_frame = align_up(super::kernel_end(), PAGE_SIZE) / PAGE_SIZE;
    f = kernel_start_frame;
    while f < kernel_end_frame {
        bitmap_set(f);
        f += 1;
    }

    // Count total and used frames
    let max_frame = if super::total_memory() > 0 {
        super::total_memory() / PAGE_SIZE
    } else {
        MAX_FRAMES as u32
    };

    unsafe {
        TOTAL_FRAMES = max_frame;
        USED_FRAMES = 0;
    }

    // Count used frames
    let mut used: u32 = 0;
    f = 0;
    while f < max_frame {
        if bitmap_test(f) {
            used += 1;
        }
        f += 1;
    }
    unsafe { USED_FRAMES = used; }

    let free = max_frame - used;
    printkln!("  Frame allocator: {} total, {} used, {} free ({} KB free)",
        max_frame, used, free, free * 4);
}

/// Allocate a single physical frame.
/// Returns the physical address of the frame, or 0 on failure.
pub fn alloc_frame() -> u32 {
    let total = unsafe { TOTAL_FRAMES };

    // Search for a free frame (first-fit)
    // Start searching from frame 256 (above 1MB) to avoid low memory
    let start = 256_u32; // 1MB / 4KB
    let mut frame = start;

    while frame < total {
        if !bitmap_test(frame) {
            bitmap_set(frame);
            unsafe { USED_FRAMES += 1; }
            return frame * PAGE_SIZE;
        }
        frame += 1;
    }

    // Also check below start (unlikely to have free frames there)
    frame = 0;
    while frame < start {
        if !bitmap_test(frame) {
            bitmap_set(frame);
            unsafe { USED_FRAMES += 1; }
            return frame * PAGE_SIZE;
        }
        frame += 1;
    }

    0 // Out of memory
}

/// Free a previously allocated physical frame.
pub fn free_frame(addr: u32) {
    if addr % PAGE_SIZE != 0 {
        kernel_panic!("free_frame: address not page-aligned");
    }

    let frame = addr / PAGE_SIZE;
    if !bitmap_test(frame) {
        kernel_panic!("free_frame: double free detected");
    }

    bitmap_clear(frame);
    unsafe {
        if USED_FRAMES > 0 {
            USED_FRAMES -= 1;
        }
    }
}

/// Allocate `count` contiguous physical frames.
/// Returns the physical address of the first frame, or 0 on failure.
pub fn alloc_frames(count: u32) -> u32 {
    if count == 0 {
        return 0;
    }
    if count == 1 {
        return alloc_frame();
    }

    let total = unsafe { TOTAL_FRAMES };
    let start = 256_u32;
    let mut frame = start;

    while frame + count <= total {
        // Check if `count` consecutive frames starting at `frame` are free
        let mut all_free = true;
        let mut i: u32 = 0;
        while i < count {
            if bitmap_test(frame + i) {
                all_free = false;
                frame = frame + i + 1; // skip past the used frame
                break;
            }
            i += 1;
        }

        if all_free {
            // Mark all frames as used
            i = 0;
            while i < count {
                bitmap_set(frame + i);
                i += 1;
            }
            unsafe { USED_FRAMES += count; }
            return frame * PAGE_SIZE;
        }
    }

    0 // Not enough contiguous frames
}

/// Free `count` contiguous physical frames starting at `addr`.
pub fn free_frames(addr: u32, count: u32) {
    let start_frame = addr / PAGE_SIZE;
    let mut i: u32 = 0;
    while i < count {
        let f = start_frame + i;
        if bitmap_test(f) {
            bitmap_clear(f);
            unsafe {
                if USED_FRAMES > 0 {
                    USED_FRAMES -= 1;
                }
            }
        }
        i += 1;
    }
}

/// Returns the total number of physical frames.
pub fn total_frames() -> u32 {
    unsafe { TOTAL_FRAMES }
}

/// Returns the number of used physical frames.
pub fn used_frames() -> u32 {
    unsafe { USED_FRAMES }
}

/// Returns the number of free physical frames.
pub fn free_frames_count() -> u32 {
    let total = unsafe { TOTAL_FRAMES };
    let used = unsafe { USED_FRAMES };
    if total > used { total - used } else { 0 }
}

/// Reserve a specific frame (by physical address) as used.
/// Used by the heap allocator to claim identity-mapped frames.
pub fn reserve_frame(addr: u32) {
    let frame = addr / PAGE_SIZE;
    if !bitmap_test(frame) {
        bitmap_set(frame);
        unsafe { USED_FRAMES += 1; }
    }
}

/// Returns whether a specific frame (by physical address) is allocated.
pub fn is_frame_used(addr: u32) -> bool {
    bitmap_test(addr / PAGE_SIZE)
}

/// Print frame allocator statistics (for shell command).
pub fn print_info(_args: &[u8]) {
    let total = unsafe { TOTAL_FRAMES };
    let used = unsafe { USED_FRAMES };
    let free = if total > used { total - used } else { 0 };

    printkln!("=== Physical Memory ===");
    printkln!("  Page size:   {} bytes", PAGE_SIZE);
    printkln!("  Total frames: {} ({} KB)", total, total * 4);
    printkln!("  Used frames:  {} ({} KB)", used, used * 4);
    printkln!("  Free frames:  {} ({} KB)", free, free * 4);
    printkln!("  Kernel: {:#x} - {:#x} ({} KB)",
        super::kernel_start(), super::kernel_end(), super::kernel_size() / 1024);
}

/// Print the memory map from multiboot (for shell command).
pub fn print_mmap(_args: &[u8]) {
    let addr = unsafe { MULTIBOOT_INFO_ADDR };

    printkln!("=== Memory Map ===");

    if addr == 0 {
        printkln!("  No multiboot info available");
        return;
    }

    let info = addr as *const super::MultibootInfo;
    let flags = unsafe { (*info).flags };

    if flags & (1 << 6) != 0 {
        let mmap_addr = unsafe { (*info).mmap_addr };
        let mmap_length = unsafe { (*info).mmap_length };

        printkln!("  Base Address    Length          Type");
        printkln!("  ------------    ------          ----");

        let mut offset: u32 = 0;
        while offset < mmap_length {
            let entry = (mmap_addr + offset) as *const super::MultibootMmapEntry;
            let size = unsafe { (*entry).size };
            let base_low = unsafe { (*entry).base_addr_low };
            let len_low = unsafe { (*entry).length_low };
            let etype = unsafe { (*entry).entry_type };

            let type_str = match etype {
                1 => "Available",
                2 => "Reserved",
                3 => "ACPI Reclaimable",
                4 => "ACPI NVS",
                5 => "Bad Memory",
                _ => "Unknown",
            };

            printkln!("  {:#x}    {} KB    {}",
                base_low, len_low / 1024, type_str);

            offset += size + 4;
        }
    } else {
        printkln!("  Memory map not available from bootloader");
        if flags & 0x1 != 0 {
            let mem_lower = unsafe { (*info).mem_lower };
            let mem_upper = unsafe { (*info).mem_upper };
            printkln!("  Lower memory: {} KB", mem_lower);
            printkln!("  Upper memory: {} KB", mem_upper);
        }
    }
}
