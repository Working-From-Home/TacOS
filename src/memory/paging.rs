/// Paging — x86 two-level page table implementation.
///
/// Linear address translation:
///   [31..22] = Page Directory index (10 bits, 0-1023)
///   [21..12] = Page Table index    (10 bits, 0-1023)
///   [11..0]  = Offset within page  (12 bits, 0-4095)
///
/// - Page Directory: 1024 entries × 4 bytes = 4KB
/// - Page Table:     1024 entries × 4 bytes = 4KB
/// - Each Page Table maps 1024 × 4KB = 4MB
/// - Full directory maps 1024 × 4MB = 4GB

use core::arch::asm;
use crate::{printkln, kernel_panic};
use super::{PAGE_SIZE, align_up, frame};

// ──────────────────────────────────────────────
//  Page entry flags
// ──────────────────────────────────────────────

pub const PAGE_PRESENT:       u32 = 1 << 0;  // Page is present in memory
pub const PAGE_WRITABLE:      u32 = 1 << 1;  // Page is read/write (vs read-only)
pub const PAGE_USER:          u32 = 1 << 2;  // Page is accessible from user mode
pub const PAGE_WRITE_THROUGH: u32 = 1 << 3;  // Write-through caching
pub const PAGE_CACHE_DISABLE: u32 = 1 << 4;  // Disable caching
pub const PAGE_ACCESSED:      u32 = 1 << 5;  // CPU sets this on access
pub const PAGE_DIRTY:         u32 = 1 << 6;  // CPU sets this on write (PTE only)
pub const PAGE_SIZE_4MB:      u32 = 1 << 7;  // 4MB pages (PDE only)
pub const PAGE_GLOBAL:        u32 = 1 << 8;  // Global page (PTE only)

/// Mask to extract the physical address from a page entry (upper 20 bits)
const ADDR_MASK: u32 = 0xFFFFF000;

/// Number of entries in a page directory or page table
const ENTRIES_PER_TABLE: usize = 1024;

// ──────────────────────────────────────────────
//  Page Directory
// ──────────────────────────────────────────────

/// Physical address of the current page directory
static mut PAGE_DIRECTORY_ADDR: u32 = 0;

/// Whether paging is currently enabled
static mut PAGING_ENABLED: bool = false;

/// Returns whether paging is enabled.
pub fn is_enabled() -> bool {
    unsafe { PAGING_ENABLED }
}

// ──────────────────────────────────────────────
//  CR register access
// ──────────────────────────────────────────────

/// Read CR0 register
fn get_cr0() -> u32 {
    let val: u32;
    unsafe { asm!("mov {}, cr0", out(reg) val); }
    val
}

/// Write CR0 register
fn set_cr0(val: u32) {
    unsafe { asm!("mov cr0, {}", in(reg) val); }
}

/// Read CR3 register (page directory physical address)
fn get_cr3() -> u32 {
    let val: u32;
    unsafe { asm!("mov {}, cr3", out(reg) val); }
    val
}

/// Write CR3 register (load page directory)
fn set_cr3(val: u32) {
    unsafe { asm!("mov cr3, {}", in(reg) val); }
}

/// Flush TLB by reloading CR3
pub fn flush_tlb() {
    let cr3 = get_cr3();
    set_cr3(cr3);
}

/// Invalidate a single page in the TLB
pub fn invlpg(addr: u32) {
    unsafe {
        asm!("invlpg [{}]", in(reg) addr, options(nostack));
    }
}

// ──────────────────────────────────────────────
//  Page table entry helpers
// ──────────────────────────────────────────────

/// Create a page directory entry pointing to a page table.
fn make_pde(pt_phys_addr: u32, flags: u32) -> u32 {
    (pt_phys_addr & ADDR_MASK) | (flags & 0xFFF)
}

/// Create a page table entry pointing to a physical frame.
fn make_pte(frame_phys_addr: u32, flags: u32) -> u32 {
    (frame_phys_addr & ADDR_MASK) | (flags & 0xFFF)
}

/// Get the physical address from a page entry.
fn entry_addr(entry: u32) -> u32 {
    entry & ADDR_MASK
}

/// Check if a page entry is present.
fn entry_present(entry: u32) -> bool {
    entry & PAGE_PRESENT != 0
}

// ──────────────────────────────────────────────
//  Address decomposition
// ──────────────────────────────────────────────

/// Extract the page directory index from a virtual address (bits 31..22)
fn pd_index(vaddr: u32) -> usize {
    ((vaddr >> 22) & 0x3FF) as usize
}

/// Extract the page table index from a virtual address (bits 21..12)
fn pt_index(vaddr: u32) -> usize {
    ((vaddr >> 12) & 0x3FF) as usize
}

/// Extract the page offset from a virtual address (bits 11..0)
#[allow(dead_code)]
fn page_offset(vaddr: u32) -> usize {
    (vaddr & 0xFFF) as usize
}

// ──────────────────────────────────────────────
//  Initialization — Identity mapping
// ──────────────────────────────────────────────

/// Initialize paging with identity mapping.
///
/// Identity maps all available physical memory so that virtual address = physical address.
/// This is the simplest paging setup and preserves existing code behavior.
///
/// Kernel pages (below KERNEL_SPACE_START) are mapped as supervisor-only.
/// All mapped pages are marked read/write.
pub fn init() {
    printkln!("  Paging: setting up identity mapping...");

    // Allocate a frame for the page directory
    let pd_addr = frame::alloc_frame();
    if pd_addr == 0 {
        kernel_panic!("Failed to allocate page directory");
    }

    // Zero out the page directory
    let pd = pd_addr as *mut u32;
    let mut i: usize = 0;
    while i < ENTRIES_PER_TABLE {
        unsafe { *pd.add(i) = 0; }
        i += 1;
    }

    // Calculate how many page directory entries we need
    // Each PDE maps 4MB (1024 pages × 4KB)
    let total_mem = super::total_memory();
    let num_pdes = if total_mem > 0 {
        let n = align_up(total_mem, 4 * 1024 * 1024) / (4 * 1024 * 1024);
        if n > 1024 { 1024 } else { n as usize }
    } else {
        32 // Default: map 128MB
    };

    printkln!("  Paging: mapping {} × 4MB = {} MB", num_pdes as u32, (num_pdes * 4) as u32);

    // For each 4MB region, allocate a page table and fill identity mapping
    let mut pde_idx: usize = 0;
    while pde_idx < num_pdes {
        let base = (pde_idx as u32) * 4 * 1024 * 1024; // Base physical address for this 4MB region

        // Allocate a frame for the page table
        let pt_addr = frame::alloc_frame();
        if pt_addr == 0 {
            kernel_panic!("Failed to allocate page table");
        }

        // Fill the page table with identity-mapped entries
        let pt = pt_addr as *mut u32;
        let mut pte_idx: usize = 0;
        while pte_idx < ENTRIES_PER_TABLE {
            let phys_addr = base + (pte_idx as u32) * PAGE_SIZE;
            let flags = if phys_addr < super::KERNEL_SPACE_START {
                // Below kernel space boundary: supervisor-only, read/write
                PAGE_PRESENT | PAGE_WRITABLE
            } else {
                // Kernel space: supervisor-only, read/write
                PAGE_PRESENT | PAGE_WRITABLE
            };
            unsafe { *pt.add(pte_idx) = make_pte(phys_addr, flags); }
            pte_idx += 1;
        }

        // Set the page directory entry
        let pde_flags = PAGE_PRESENT | PAGE_WRITABLE;
        unsafe { *pd.add(pde_idx) = make_pde(pt_addr, pde_flags); }

        pde_idx += 1;
    }

    // Save page directory address
    unsafe { PAGE_DIRECTORY_ADDR = pd_addr; }

    // Load the page directory into CR3
    set_cr3(pd_addr);

    // Enable paging by setting bit 31 of CR0
    let cr0 = get_cr0();
    set_cr0(cr0 | 0x80000000);

    unsafe { PAGING_ENABLED = true; }

    printkln!("  Paging: enabled (CR3={:#x})", pd_addr);
}

// ──────────────────────────────────────────────
//  Page mapping / unmapping
// ──────────────────────────────────────────────

/// Map a single virtual page to a physical frame with the given flags.
///
/// If the page table for this region doesn't exist, it will be allocated.
/// If the page is already mapped, it will be remapped (the old mapping is overwritten).
pub fn map_page(vaddr: u32, paddr: u32, flags: u32) {
    let pd_addr = unsafe { PAGE_DIRECTORY_ADDR };
    if pd_addr == 0 {
        kernel_panic!("map_page: paging not initialized");
    }

    let pdidx = pd_index(vaddr);
    let ptidx = pt_index(vaddr);

    let pd = pd_addr as *mut u32;
    let pde = unsafe { *pd.add(pdidx) };

    let pt_addr: u32;

    if entry_present(pde) {
        // Page table already exists
        pt_addr = entry_addr(pde);
    } else {
        // Allocate a new page table
        pt_addr = frame::alloc_frame();
        if pt_addr == 0 {
            kernel_panic!("map_page: failed to allocate page table");
        }

        // Zero the new page table
        let pt = pt_addr as *mut u32;
        let mut i: usize = 0;
        while i < ENTRIES_PER_TABLE {
            unsafe { *pt.add(i) = 0; }
            i += 1;
        }

        // Install in page directory
        // PDE flags include USER if any page in the table might be user-accessible
        let pde_flags = PAGE_PRESENT | PAGE_WRITABLE | (flags & PAGE_USER);
        unsafe { *pd.add(pdidx) = make_pde(pt_addr, pde_flags); }
    }

    // Set the page table entry
    let pt = pt_addr as *mut u32;
    unsafe { *pt.add(ptidx) = make_pte(paddr, flags | PAGE_PRESENT); }

    // Invalidate TLB for this page
    if unsafe { PAGING_ENABLED } {
        invlpg(vaddr);
    }
}

/// Unmap a virtual page.
/// Returns the physical address of the frame that was mapped, or 0 if not mapped.
pub fn unmap_page(vaddr: u32) -> u32 {
    let pd_addr = unsafe { PAGE_DIRECTORY_ADDR };
    if pd_addr == 0 {
        return 0;
    }

    let pdidx = pd_index(vaddr);
    let ptidx = pt_index(vaddr);

    let pd = pd_addr as *mut u32;
    let pde = unsafe { *pd.add(pdidx) };

    if !entry_present(pde) {
        return 0;
    }

    let pt_addr = entry_addr(pde);
    let pt = pt_addr as *mut u32;
    let pte = unsafe { *pt.add(ptidx) };

    if !entry_present(pte) {
        return 0;
    }

    let phys = entry_addr(pte);

    // Clear the page table entry
    unsafe { *pt.add(ptidx) = 0; }

    // Invalidate TLB
    if unsafe { PAGING_ENABLED } {
        invlpg(vaddr);
    }

    phys
}

/// Get the physical address mapped to a virtual address.
/// Returns Some(phys_addr) or None if not mapped.
pub fn virt_to_phys(vaddr: u32) -> Option<u32> {
    let pd_addr = unsafe { PAGE_DIRECTORY_ADDR };
    if pd_addr == 0 {
        return None;
    }

    let pdidx = pd_index(vaddr);
    let ptidx = pt_index(vaddr);
    let offset = (vaddr & 0xFFF) as u32;

    let pd = pd_addr as *mut u32;
    let pde = unsafe { *pd.add(pdidx) };

    if !entry_present(pde) {
        return None;
    }

    let pt_addr = entry_addr(pde);
    let pt = pt_addr as *mut u32;
    let pte = unsafe { *pt.add(ptidx) };

    if !entry_present(pte) {
        return None;
    }

    Some(entry_addr(pte) + offset)
}

/// Check if a virtual address is mapped.
pub fn is_mapped(vaddr: u32) -> bool {
    virt_to_phys(vaddr).is_some()
}

// ──────────────────────────────────────────────
//  Debug / info
// ──────────────────────────────────────────────

/// Print page directory summary (for shell command).
pub fn print_info(_args: &[u8]) {
    let pd_addr = unsafe { PAGE_DIRECTORY_ADDR };

    printkln!("=== Paging Info ===");
    printkln!("  Status: {}", if unsafe { PAGING_ENABLED } { "enabled" } else { "disabled" });
    printkln!("  CR3 (Page Directory): {:#x}", pd_addr);

    if pd_addr == 0 {
        return;
    }

    let pd = pd_addr as *mut u32;
    let mut mapped_entries: u32 = 0;
    let mut total_pages: u32 = 0;
    let mut user_pages: u32 = 0;
    let mut rw_pages: u32 = 0;

    let mut pdidx: usize = 0;
    while pdidx < ENTRIES_PER_TABLE {
        let pde = unsafe { *pd.add(pdidx) };
        if entry_present(pde) {
            mapped_entries += 1;

            let pt_addr = entry_addr(pde);
            let pt = pt_addr as *mut u32;
            let mut ptidx: usize = 0;
            while ptidx < ENTRIES_PER_TABLE {
                let pte = unsafe { *pt.add(ptidx) };
                if entry_present(pte) {
                    total_pages += 1;
                    if pte & PAGE_USER != 0 {
                        user_pages += 1;
                    }
                    if pte & PAGE_WRITABLE != 0 {
                        rw_pages += 1;
                    }
                }
                ptidx += 1;
            }
        }
        pdidx += 1;
    }

    printkln!("  Page Directory entries in use: {}", mapped_entries);
    printkln!("  Mapped regions: {} × 4MB", mapped_entries);
    printkln!("  Total mapped pages: {} ({} MB)", total_pages, total_pages / 256);
    printkln!("  Read/Write pages:   {}", rw_pages);
    printkln!("  User-accessible:    {}", user_pages);
    printkln!("  Supervisor-only:    {}", total_pages - user_pages);
    printkln!("  Kernel space:       {:#x} - {:#x} (conceptual)",
        super::KERNEL_SPACE_START, 0xFFFFFFFF_u32);
    printkln!("  User space:         {:#x} - {:#x} (conceptual)",
        0_u32, super::USER_SPACE_END);
}
