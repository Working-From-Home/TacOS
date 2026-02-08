/// Virtual memory allocator — vmalloc/vfree/vsize/vbrk.
///
/// Unlike kmalloc (which allocates from the kernel heap in identity-mapped
/// physical memory), vmalloc allocates from a separate virtual address region
/// and maps non-contiguous physical frames into contiguous virtual addresses.
///
/// Virtual memory region: starts at VMALLOC_BASE (0xD0000000)
/// This is above the kernel identity-mapped region but below 0xFFFFFFFF.
///
/// vmalloc is useful when you need large, virtually contiguous allocations
/// but don't need physically contiguous memory (e.g., for DMA you'd use kmalloc).

use crate::{printkln, kernel_panic};
use super::{PAGE_SIZE, align_up, frame, paging};

// ──────────────────────────────────────────────
//  Configuration
// ──────────────────────────────────────────────

/// Base address for vmalloc region (above identity-mapped memory)
const VMALLOC_BASE: u32 = 0xD0000000;

/// Maximum size of the vmalloc region (256 MB)
const VMALLOC_MAX_SIZE: u32 = 256 * 1024 * 1024;

// ──────────────────────────────────────────────
//  Allocation tracking
// ──────────────────────────────────────────────

/// Maximum number of vmalloc allocations we can track
const MAX_VMALLOC_ENTRIES: usize = 256;

/// A vmalloc allocation entry.
#[derive(Copy, Clone)]
struct VmallocEntry {
    vaddr: u32,     // Virtual address of the allocation
    size: u32,      // Size in bytes (multiple of PAGE_SIZE)
    in_use: bool,   // Whether this entry is active
}

/// Table of vmalloc allocations
static mut VMALLOC_TABLE: [VmallocEntry; MAX_VMALLOC_ENTRIES] = [VmallocEntry {
    vaddr: 0,
    size: 0,
    in_use: false,
}; MAX_VMALLOC_ENTRIES];

/// Current vmalloc break (next available virtual address)
static mut VMALLOC_BRK: u32 = VMALLOC_BASE;

/// Number of active vmalloc allocations
static mut VMALLOC_COUNT: u32 = 0;

// ──────────────────────────────────────────────
//  Initialization
// ──────────────────────────────────────────────

/// Initialize the virtual memory allocator.
pub fn init() {
    unsafe {
        VMALLOC_BRK = VMALLOC_BASE;
        VMALLOC_COUNT = 0;
    }

    printkln!("  Vmalloc: region {:#x} - {:#x} ({} MB)",
        VMALLOC_BASE, VMALLOC_BASE + VMALLOC_MAX_SIZE, VMALLOC_MAX_SIZE / (1024 * 1024));
}

// ──────────────────────────────────────────────
//  vbrk — extend virtual memory region
// ──────────────────────────────────────────────

/// Extend the virtual memory region by `increment` bytes (rounded up to pages).
///
/// Allocates physical frames and maps them at the next available virtual address.
/// Returns the starting virtual address of the new region, or 0 on failure.
pub fn vbrk(increment: u32) -> u32 {
    if increment == 0 {
        return unsafe { VMALLOC_BRK };
    }

    let pages_needed = align_up(increment, PAGE_SIZE) / PAGE_SIZE;
    let brk = unsafe { VMALLOC_BRK };

    // Check we don't exceed the vmalloc region
    let new_brk = brk + pages_needed * PAGE_SIZE;
    if new_brk > VMALLOC_BASE + VMALLOC_MAX_SIZE {
        return 0; // Out of virtual address space
    }

    // Allocate physical frames and map them
    let mut page: u32 = 0;
    while page < pages_needed {
        let phys = frame::alloc_frame();
        if phys == 0 {
            // Rollback: unmap and free any pages we already allocated
            let mut rollback: u32 = 0;
            while rollback < page {
                let vaddr = brk + rollback * PAGE_SIZE;
                let old_phys = paging::unmap_page(vaddr);
                if old_phys != 0 {
                    frame::free_frame(old_phys);
                }
                rollback += 1;
            }
            return 0;
        }

        let vaddr = brk + page * PAGE_SIZE;
        paging::map_page(vaddr, phys, paging::PAGE_PRESENT | paging::PAGE_WRITABLE);

        // Zero the page
        crate::klib::memory::memset(vaddr as *mut u8, 0, PAGE_SIZE as usize);

        page += 1;
    }

    unsafe { VMALLOC_BRK = new_brk; }

    brk
}

// ──────────────────────────────────────────────
//  vmalloc — allocate virtual memory
// ──────────────────────────────────────────────

/// Allocate `size` bytes of virtually contiguous memory.
///
/// The physical pages backing this allocation may be non-contiguous.
/// Returns a pointer to the allocated virtual memory, or null on failure.
///
/// The allocation is page-aligned and rounded up to page size.
pub fn vmalloc(size: u32) -> *mut u8 {
    if size == 0 {
        return core::ptr::null_mut();
    }

    let alloc_size = align_up(size, PAGE_SIZE);

    // Find a free entry in the vmalloc table
    let entry_idx = find_free_entry();
    if entry_idx >= MAX_VMALLOC_ENTRIES {
        return core::ptr::null_mut(); // Too many allocations
    }

    // Allocate virtual space
    let vaddr = vbrk(alloc_size);
    if vaddr == 0 {
        return core::ptr::null_mut();
    }

    // Record the allocation
    unsafe {
        VMALLOC_TABLE[entry_idx] = VmallocEntry {
            vaddr,
            size: alloc_size,
            in_use: true,
        };
        VMALLOC_COUNT += 1;
    }

    vaddr as *mut u8
}

// ──────────────────────────────────────────────
//  vfree — free virtual memory
// ──────────────────────────────────────────────

/// Free memory previously allocated with vmalloc.
///
/// Unmaps the virtual pages and frees the underlying physical frames.
pub fn vfree(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }

    let vaddr = ptr as u32;

    // Find the allocation entry
    let mut i: usize = 0;
    while i < MAX_VMALLOC_ENTRIES {
        let entry = unsafe { &mut VMALLOC_TABLE[i] };
        if entry.in_use && entry.vaddr == vaddr {
            // Unmap pages and free physical frames
            let pages = entry.size / PAGE_SIZE;
            let mut page: u32 = 0;
            while page < pages {
                let page_vaddr = vaddr + page * PAGE_SIZE;
                let phys = paging::unmap_page(page_vaddr);
                if phys != 0 {
                    frame::free_frame(phys);
                }
                page += 1;
            }

            entry.in_use = false;
            entry.vaddr = 0;
            entry.size = 0;

            unsafe {
                if VMALLOC_COUNT > 0 {
                    VMALLOC_COUNT -= 1;
                }
            }
            return;
        }
        i += 1;
    }

    kernel_panic!("vfree: pointer not found in vmalloc table");
}

// ──────────────────────────────────────────────
//  vsize — get virtual allocation size
// ──────────────────────────────────────────────

/// Returns the size of the vmalloc allocation pointed to by `ptr`.
pub fn vsize(ptr: *const u8) -> u32 {
    if ptr.is_null() {
        return 0;
    }

    let vaddr = ptr as u32;

    let mut i: usize = 0;
    while i < MAX_VMALLOC_ENTRIES {
        let entry = unsafe { &VMALLOC_TABLE[i] };
        if entry.in_use && entry.vaddr == vaddr {
            return entry.size;
        }
        i += 1;
    }

    0
}

// ──────────────────────────────────────────────
//  Internal helpers
// ──────────────────────────────────────────────

/// Find a free entry in the vmalloc table.
fn find_free_entry() -> usize {
    let mut i: usize = 0;
    while i < MAX_VMALLOC_ENTRIES {
        if !unsafe { VMALLOC_TABLE[i].in_use } {
            return i;
        }
        i += 1;
    }
    MAX_VMALLOC_ENTRIES // no free entry
}

// ──────────────────────────────────────────────
//  Debug / info
// ──────────────────────────────────────────────

/// Print virtual memory allocator statistics (for shell command).
pub fn print_info(_args: &[u8]) {
    let brk = unsafe { VMALLOC_BRK };
    let count = unsafe { VMALLOC_COUNT };

    printkln!("=== Virtual Memory (vmalloc) ===");
    printkln!("  Region: {:#x} - {:#x}", VMALLOC_BASE, VMALLOC_BASE + VMALLOC_MAX_SIZE);
    printkln!("  Break:  {:#x}", brk);
    printkln!("  Used:   {} bytes ({} pages)",
        brk - VMALLOC_BASE, (brk - VMALLOC_BASE) / PAGE_SIZE);
    printkln!("  Active allocations: {}", count);

    if count > 0 {
        printkln!();
        printkln!("  Allocations:");
        let mut i: usize = 0;
        let mut shown: u32 = 0;
        while i < MAX_VMALLOC_ENTRIES && shown < count {
            let entry = unsafe { &VMALLOC_TABLE[i] };
            if entry.in_use {
                printkln!("    [{:>3}] {:#x} - {:#x} ({} pages)",
                    i as u32, entry.vaddr,
                    entry.vaddr + entry.size,
                    entry.size / PAGE_SIZE);
                shown += 1;
            }
            i += 1;
        }
    }
}
