#![allow(dead_code)]

/// Returns the length of a null-terminated string (C-style).
pub fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    unsafe {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}

/// Compares two null-terminated strings (C-style).
pub fn strcmp(s1: *const u8, s2: *const u8) -> i32 {
    unsafe {
        let mut i = 0;
        while *s1.add(i) != 0 && *s2.add(i) != 0 {
            if *s1.add(i) != *s2.add(i) {
                return (*s1.add(i) - *s2.add(i)) as i32;
            }
            i += 1;
        }
        (*s1.add(i) - *s2.add(i)) as i32
    }
}

/// Compares the first n bytes of two null-terminated strings (C-style).
pub fn strncmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    unsafe {
        let mut i = 0;
        while i < n && *s1.add(i) != 0 && *s2.add(i) != 0 {
            if *s1.add(i) != *s2.add(i) {
                return (*s1.add(i) - *s2.add(i)) as i32;
            }
            i += 1;
        }
        if i == n {
            return 0;
        }
        (*s1.add(i) - *s2.add(i)) as i32
    }
}

/// Copies a null-terminated string (C-style) from src to dest.
pub fn strcpy(dest: *mut u8, src: *const u8) -> *mut u8 {
    unsafe {
        let mut i = 0;
        while *src.add(i) != 0 {
            *dest.add(i) = *src.add(i);
            i += 1;
        }
        *dest.add(i) = 0;
    }
    dest
}

/// Copies the first n bytes of a null-terminated string (C-style) from src to dest.
pub fn strncpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        let mut i = 0;
        while i < n && *src.add(i) != 0 {
            *dest.add(i) = *src.add(i);
            i += 1;
        }
        while i < n {
            *dest.add(i) = 0;
            i += 1;
        }
    }
    dest
}

/// Concatenates two null-terminated strings (C-style) and returns the result.
pub fn strcat(dest: *mut u8, src: *const u8) -> *mut u8 {
    unsafe {
        let mut i = 0;
        while *dest.add(i) != 0 {
            i += 1;
        }
        let mut j = 0;
        while *src.add(j) != 0 {
            *dest.add(i + j) = *src.add(j);
            j += 1;
        }
        *dest.add(i + j) = 0;
    }
    dest
}

/// Concatenates the first n bytes of two null-terminated strings (C-style) and returns the result.
pub fn strncat(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    unsafe {
        let mut i = 0;
        while *dest.add(i) != 0 {
            i += 1;
        }
        let mut j = 0;
        while j < n && *src.add(j) != 0 {
            *dest.add(i + j) = *src.add(j);
            j += 1;
        }
        *dest.add(i + j) = 0;
    }
    dest
}

/// Searches for the first occurrence of a character in a null-terminated string (C-style).
pub fn strchr(s: *const u8, c: u8) -> *const u8 {
    unsafe {
        let mut i = 0;
        while *s.add(i) != 0 {
            if *s.add(i) == c {
                return s.add(i);
            }
            i += 1;
        }
    }
    core::ptr::null()
}

/// Searches needle in haystack and returns a pointer to the first occurrence of needle in haystack.
pub fn strstr(haystack: *const u8, needle: *const u8) -> *const u8 {
    unsafe {
        let mut i = 0;
        let needle_len = strlen(needle);
        while *haystack.add(i) != 0 {
            if strncmp(haystack.add(i), needle, needle_len) == 0 {
                return haystack.add(i);
            }
            i += 1;
        }
    }
    core::ptr::null()
}