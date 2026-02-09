/// Command history — stores the last N commands for recall with ArrowUp/ArrowDown.
///
/// All array accesses use raw pointer arithmetic to avoid pulling in
/// `core::panicking::panic_bounds_check` (which doesn't exist in our kernel).

const MAX_HISTORY: usize = 5;
const MAX_CMD_LEN: usize = 78;

struct HistoryEntry {
    buf: [u8; MAX_CMD_LEN],
    len: usize,
}

impl HistoryEntry {
    const fn empty() -> Self {
        HistoryEntry {
            buf: [0; MAX_CMD_LEN],
            len: 0,
        }
    }
}

static mut ENTRIES: [HistoryEntry; MAX_HISTORY] = [
    HistoryEntry::empty(),
    HistoryEntry::empty(),
    HistoryEntry::empty(),
    HistoryEntry::empty(),
    HistoryEntry::empty(),
];

/// Number of entries stored so far (max MAX_HISTORY).
static mut COUNT: usize = 0;

/// Ring buffer head: points to the next slot to write.
static mut HEAD: usize = 0;

/// Current browsing index: 0 = not browsing, 1 = most recent, etc.
static mut BROWSE_INDEX: usize = 0;

/// Raw pointer helper to access ENTRIES[idx].
#[inline(always)]
unsafe fn entry_ptr(idx: usize) -> *mut HistoryEntry {
    ENTRIES.as_mut_ptr().add(idx)
}

/// Pushes a command into the history ring buffer.
/// Ignores empty commands and duplicates of the most recent entry.
pub fn push(cmd: &[u8]) {
    if cmd.is_empty() {
        return;
    }

    unsafe {
        // Skip if identical to the most recent entry
        if COUNT > 0 {
            let last = (HEAD + MAX_HISTORY - 1) % MAX_HISTORY;
            let last_entry = entry_ptr(last);
            if (*last_entry).len == cmd.len() {
                let mut same = true;
                let mut i = 0;
                while i < cmd.len() {
                    if *(*last_entry).buf.as_ptr().add(i) != *cmd.as_ptr().add(i) {
                        same = false;
                        break;
                    }
                    i += 1;
                }
                if same {
                    BROWSE_INDEX = 0;
                    return;
                }
            }
        }

        let len = if cmd.len() > MAX_CMD_LEN { MAX_CMD_LEN } else { cmd.len() };
        let entry = entry_ptr(HEAD);
        let mut i = 0;
        while i < len {
            *(*entry).buf.as_mut_ptr().add(i) = *cmd.as_ptr().add(i);
            i += 1;
        }
        (*entry).len = len;

        HEAD = (HEAD + 1) % MAX_HISTORY;
        if COUNT < MAX_HISTORY {
            COUNT += 1;
        }
        BROWSE_INDEX = 0;
    }
}

/// Move up in history (older). Returns the command bytes or None if at the end.
pub fn up() -> Option<&'static [u8]> {
    unsafe {
        if COUNT == 0 || BROWSE_INDEX >= COUNT {
            return None;
        }
        BROWSE_INDEX += 1;
        let idx = (HEAD + MAX_HISTORY - BROWSE_INDEX) % MAX_HISTORY;
        let entry = entry_ptr(idx);
        let len = (*entry).len;
        Some(core::slice::from_raw_parts((*entry).buf.as_ptr(), len))
    }
}

/// Move down in history (newer). Returns the command bytes, or empty slice if back to live input.
pub fn down() -> Option<&'static [u8]> {
    unsafe {
        if BROWSE_INDEX == 0 {
            return None;
        }
        BROWSE_INDEX -= 1;
        if BROWSE_INDEX == 0 {
            // Back to live (empty) input
            static EMPTY: [u8; 0] = [];
            return Some(&EMPTY);
        }
        let idx = (HEAD + MAX_HISTORY - BROWSE_INDEX) % MAX_HISTORY;
        let entry = entry_ptr(idx);
        let len = (*entry).len;
        Some(core::slice::from_raw_parts((*entry).buf.as_ptr(), len))
    }
}

/// Resets browse position (called when a new command is entered).
pub fn reset_browse() {
    unsafe {
        BROWSE_INDEX = 0;
    }
}
