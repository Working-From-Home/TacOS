/// Kernel heap allocator — physical memory helpers.
///
/// Provides: kmalloc, kfree, ksize, kbrk
///
/// Uses an implicit free-list allocator in identity-mapped physical memory.
/// The heap occupies contiguous physical memory starting right after the kernel.
/// Since paging identity-maps all available RAM, the heap's physical addresses
/// equal its virtual addresses — no page table manipulation needed.
///
/// kbrk grows the heap by reserving frames in the frame allocator's bitmap,
/// preventing them from being handed out by alloc_frame().
///
/// Memory layout:
///   [BlockHeader (8 bytes)] [user data (N bytes)] [BlockHeader] [data] ...

use crate::{printkln, kernel_panic};
use super::{PAGE_SIZE, align_up, frame};

// ──────────────────────────────────────────────
//  Block header
// ──────────────────────────────────────────────

/// Header prepended to each allocation.
/// size   = number of user-data bytes (not including this header)
/// flags  = bit 0: 1=free, 0=allocated
///
/// Total header size: 8 bytes.
/// Next block (implicit): at address (self + 8 + size)
#[repr(C)]
struct BlockHeader {
    size: u32,
    flags: u32,
}

const HEADER_SIZE: u32 = 8; // size_of::<BlockHeader>()
const FLAG_FREE: u32 = 1;
const MIN_ALLOC: u32 = 8;   // Minimum allocation size (for alignment)

impl BlockHeader {
    fn is_free(&self) -> bool {
        self.flags & FLAG_FREE != 0
    }

    fn set_free(&mut self) {
        self.flags |= FLAG_FREE;
    }

    fn set_used(&mut self) {
        self.flags &= !FLAG_FREE;
    }

    /// Pointer to the user data area
    fn data_ptr(&self) -> *mut u8 {
        unsafe { (self as *const BlockHeader as *mut u8).add(HEADER_SIZE as usize) }
    }

    /// Pointer to the next block in the implicit list
    fn next(&self) -> *mut BlockHeader {
        unsafe {
            (self as *const BlockHeader as *mut u8)
                .add((HEADER_SIZE + self.size) as usize) as *mut BlockHeader
        }
    }
}

// ──────────────────────────────────────────────
//  Heap state
// ──────────────────────────────────────────────

/// Start of the kernel heap (page-aligned, after kernel end)
static mut HEAP_START: u32 = 0;

/// Current heap break — end of used heap memory (where next alloc goes)
static mut HEAP_BRK: u32 = 0;

/// End of reserved heap pages (page-aligned, >= HEAP_BRK)
static mut HEAP_END: u32 = 0;

/// Number of active allocations (for statistics)
static mut ALLOC_COUNT: u32 = 0;

// ──────────────────────────────────────────────
//  Initialization
// ──────────────────────────────────────────────

/// Initialize the kernel heap.
///
/// The heap sits in identity-mapped memory right after the kernel's BSS.
/// No page table manipulation needed — we just reserve frames in the bitmap.
pub fn init() {
    let heap_start = align_up(super::kernel_end(), PAGE_SIZE);

    unsafe {
        HEAP_START = heap_start;
        HEAP_BRK = heap_start;
        HEAP_END = heap_start;
    }

    // Reserve one initial page
    kbrk(PAGE_SIZE);

    printkln!("  Heap: {:#x} (initial {} bytes)",
        heap_start, PAGE_SIZE);
}

// ──────────────────────────────────────────────
//  kbrk — extend the kernel heap
// ──────────────────────────────────────────────

/// Extend the kernel heap by `increment` bytes.
///
/// Reserves frames in the frame allocator bitmap so they won't be
/// given out by alloc_frame(). Since paging identity-maps all RAM,
/// the newly reserved frames are already accessible.
///
/// Returns the previous break value (start of new memory), or 0 on failure.
pub fn kbrk(increment: u32) -> u32 {
    if increment == 0 {
        return unsafe { HEAP_BRK };
    }

    let old_brk = unsafe { HEAP_BRK };
    let new_brk = old_brk + increment;

    // Reserve new page frames as needed
    let mut current_end = unsafe { HEAP_END };
    while new_brk > current_end {
        // Reserve this frame in the bitmap (prevents alloc_frame from giving it out)
        frame::reserve_frame(current_end);
        current_end += PAGE_SIZE;
    }

    unsafe {
        HEAP_BRK = new_brk;
        HEAP_END = current_end;
    }

    old_brk
}

// ──────────────────────────────────────────────
//  kmalloc — allocate physical memory
// ──────────────────────────────────────────────

/// Allocate `size` bytes from the kernel heap.
///
/// Returns a pointer to the allocated memory, or null (0) on failure.
/// The returned pointer is aligned to at least 4 bytes.
///
/// Uses a first-fit implicit free-list allocator.
pub fn kmalloc(size: u32) -> *mut u8 {
    if size == 0 {
        return core::ptr::null_mut();
    }

    // Align size up to MIN_ALLOC for alignment
    let alloc_size = if size < MIN_ALLOC {
        MIN_ALLOC
    } else {
        align_up(size, 4)
    };

    let heap_start = unsafe { HEAP_START };
    let heap_brk = unsafe { HEAP_BRK };

    // Walk the implicit free list looking for a free block
    if heap_brk > heap_start {
        let mut current = heap_start as *mut BlockHeader;
        let end = heap_brk as *mut BlockHeader;

        while (current as u32) < (end as u32) {
            let block = unsafe { &mut *current };

            if block.is_free() && block.size >= alloc_size {
                // Found a free block large enough

                // Split if there's enough room for another block
                let remaining = block.size - alloc_size;
                if remaining > HEADER_SIZE + MIN_ALLOC {
                    // Split: shrink this block and create a new free block after it
                    let old_size = block.size;
                    block.size = alloc_size;
                    block.set_used();

                    let new_block = unsafe {
                        &mut *((current as *mut u8)
                            .add((HEADER_SIZE + alloc_size) as usize) as *mut BlockHeader)
                    };
                    new_block.size = old_size - alloc_size - HEADER_SIZE;
                    new_block.flags = FLAG_FREE;
                } else {
                    // Use the whole block
                    block.set_used();
                }

                unsafe { ALLOC_COUNT += 1; }
                return block.data_ptr();
            }

            // Move to next block
            current = block.next();
        }
    }

    // No suitable free block found — extend the heap
    let needed = HEADER_SIZE + alloc_size;
    let old_brk = kbrk(needed);
    if old_brk == 0 {
        return core::ptr::null_mut(); // Out of memory
    }

    // Create a new block header at the old break position
    let block = old_brk as *mut BlockHeader;
    unsafe {
        (*block).size = alloc_size;
        (*block).flags = 0; // allocated
        ALLOC_COUNT += 1;
    }

    unsafe { (*block).data_ptr() }
}

/// Allocate `size` bytes from the kernel heap, zeroed.
pub fn kmalloc_zeroed(size: u32) -> *mut u8 {
    let ptr = kmalloc(size);
    if !ptr.is_null() {
        crate::klib::memory::memset(ptr, 0, size as usize);
    }
    ptr
}

// ──────────────────────────────────────────────
//  kfree — free physical memory
// ──────────────────────────────────────────────

/// Free memory previously allocated with kmalloc.
///
/// Marks the block as free and coalesces adjacent free blocks.
pub fn kfree(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }

    // Find the block header (immediately before the user data)
    let block = unsafe { &mut *((ptr as u32 - HEADER_SIZE) as *mut BlockHeader) };

    if block.is_free() {
        kernel_panic!("kfree: double free detected");
    }

    block.set_free();
    unsafe {
        if ALLOC_COUNT > 0 {
            ALLOC_COUNT -= 1;
        }
    }

    // Coalesce with next block if it's free
    let heap_brk = unsafe { HEAP_BRK };
    let next = block.next();
    if (next as u32) < heap_brk {
        let next_block = unsafe { &*next };
        if next_block.is_free() {
            block.size += HEADER_SIZE + next_block.size;
        }
    }
}

// ──────────────────────────────────────────────
//  ksize — get allocation size
// ──────────────────────────────────────────────

/// Returns the size of the allocation pointed to by `ptr`.
///
/// This is the usable size (may be >= the originally requested size
/// due to alignment and minimum allocation constraints).
pub fn ksize(ptr: *const u8) -> u32 {
    if ptr.is_null() {
        return 0;
    }

    let block = unsafe { &*((ptr as u32 - HEADER_SIZE) as *const BlockHeader) };
    block.size
}

// ──────────────────────────────────────────────
//  Statistics / Debug
// ──────────────────────────────────────────────

/// Print kernel heap statistics (for shell command).
pub fn print_info(_args: &[u8]) {
    let heap_start = unsafe { HEAP_START };
    let heap_brk = unsafe { HEAP_BRK };
    let heap_end = unsafe { HEAP_END };
    let alloc_count = unsafe { ALLOC_COUNT };

    printkln!("=== Kernel Heap (kmalloc) ===");
    printkln!("  Heap start:  {:#x}", heap_start);
    printkln!("  Heap break:  {:#x}", heap_brk);
    printkln!("  Heap end:    {:#x}", heap_end);
    printkln!("  Heap used:   {} bytes", heap_brk - heap_start);
    printkln!("  Heap capacity: {} bytes ({} pages)",
        heap_end - heap_start, (heap_end - heap_start) / PAGE_SIZE);
    printkln!("  Active allocations: {}", alloc_count);

    // Walk the block list and show details
    if heap_brk > heap_start {
        printkln!();
        printkln!("  Block list:");
        let mut current = heap_start as *mut BlockHeader;
        let end = heap_brk as *const u8;
        let mut block_num: u32 = 0;
        let mut total_free: u32 = 0;
        let mut total_used: u32 = 0;

        while (current as u32) < (end as u32) {
            let block = unsafe { &*current };
            let status = if block.is_free() { "free" } else { "used" };
            printkln!("    [{}] {:#x}: {} bytes ({})",
                block_num, current as u32, block.size, status);

            if block.is_free() {
                total_free += block.size;
            } else {
                total_used += block.size;
            }

            current = block.next();
            block_num += 1;

            // Safety: avoid infinite loops
            if block_num > 1000 {
                printkln!("    ... (truncated)");
                break;
            }
        }

        printkln!("  Total used: {} bytes, Total free: {} bytes", total_used, total_free);
    }
}
