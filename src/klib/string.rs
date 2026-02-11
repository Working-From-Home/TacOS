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
                return *s1.add(i) as i32 - *s2.add(i) as i32;
            }
            i += 1;
        }
        *s1.add(i) as i32 - *s2.add(i) as i32
    }
}

/// Compares the first n bytes of two null-terminated strings (C-style).
pub fn strncmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    unsafe {
        let mut i = 0;
        while i < n && *s1.add(i) != 0 && *s2.add(i) != 0 {
            if *s1.add(i) != *s2.add(i) {
                return *s1.add(i) as i32 - *s2.add(i) as i32;
            }
            i += 1;
        }
        if i == n {
            return 0;
        }
        *s1.add(i) as i32 - *s2.add(i) as i32
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── strlen ──

    #[test]
    fn strlen_empty() {
        assert_eq!(strlen(b"\0".as_ptr()), 0);
    }

    #[test]
    fn strlen_basic() {
        assert_eq!(strlen(b"hello\0".as_ptr()), 5);
    }

    #[test]
    fn strlen_single_char() {
        assert_eq!(strlen(b"a\0".as_ptr()), 1);
    }

    // ── strcmp ──

    #[test]
    fn strcmp_equal() {
        assert_eq!(strcmp(b"abc\0".as_ptr(), b"abc\0".as_ptr()), 0);
    }

    #[test]
    fn strcmp_less() {
        assert!(strcmp(b"abc\0".as_ptr(), b"abd\0".as_ptr()) < 0);
    }

    #[test]
    fn strcmp_greater() {
        assert!(strcmp(b"abd\0".as_ptr(), b"abc\0".as_ptr()) > 0);
    }

    #[test]
    fn strcmp_prefix() {
        assert!(strcmp(b"ab\0".as_ptr(), b"abc\0".as_ptr()) < 0);
    }

    #[test]
    fn strcmp_empty_both() {
        assert_eq!(strcmp(b"\0".as_ptr(), b"\0".as_ptr()), 0);
    }

    // ── strncmp ──

    #[test]
    fn strncmp_equal_within_n() {
        assert_eq!(strncmp(b"abcdef\0".as_ptr(), b"abcxyz\0".as_ptr(), 3), 0);
    }

    #[test]
    fn strncmp_differ_within_n() {
        assert!(strncmp(b"abcdef\0".as_ptr(), b"abxdef\0".as_ptr(), 3) < 0);
    }

    #[test]
    fn strncmp_zero_n() {
        assert_eq!(strncmp(b"abc\0".as_ptr(), b"xyz\0".as_ptr(), 0), 0);
    }

    // ── strcpy ──

    #[test]
    fn strcpy_basic() {
        let mut dest = [0u8; 16];
        strcpy(dest.as_mut_ptr(), b"hello\0".as_ptr());
        assert_eq!(&dest[..6], b"hello\0");
    }

    #[test]
    fn strcpy_empty() {
        let mut dest = [0xFFu8; 8];
        strcpy(dest.as_mut_ptr(), b"\0".as_ptr());
        assert_eq!(dest[0], 0);
    }

    // ── strncpy ──

    #[test]
    fn strncpy_exact() {
        let mut dest = [0xFFu8; 8];
        strncpy(dest.as_mut_ptr(), b"hi\0".as_ptr(), 5);
        assert_eq!(&dest[..5], b"hi\0\0\0");
    }

    #[test]
    fn strncpy_truncate() {
        let mut dest = [0xFFu8; 8];
        strncpy(dest.as_mut_ptr(), b"hello world\0".as_ptr(), 5);
        assert_eq!(&dest[..5], b"hello");
    }

    // ── strcat ──

    #[test]
    fn strcat_basic() {
        let mut buf = [0u8; 16];
        strcpy(buf.as_mut_ptr(), b"hello\0".as_ptr());
        strcat(buf.as_mut_ptr(), b" world\0".as_ptr());
        assert_eq!(&buf[..12], b"hello world\0");
    }

    // ── strncat ──

    #[test]
    fn strncat_partial() {
        let mut buf = [0u8; 16];
        strcpy(buf.as_mut_ptr(), b"hi\0".as_ptr());
        strncat(buf.as_mut_ptr(), b"there\0".as_ptr(), 3);
        assert_eq!(&buf[..6], b"hithe\0");
    }

    // ── strchr ──

    #[test]
    fn strchr_found() {
        let s = b"hello\0";
        let p = strchr(s.as_ptr(), b'l');
        assert_eq!(unsafe { p.offset_from(s.as_ptr()) }, 2);
    }

    #[test]
    fn strchr_not_found() {
        assert!(strchr(b"hello\0".as_ptr(), b'z').is_null());
    }

    // ── strstr ──

    #[test]
    fn strstr_found() {
        let h = b"hello world\0";
        let p = strstr(h.as_ptr(), b"world\0".as_ptr());
        assert_eq!(unsafe { p.offset_from(h.as_ptr()) }, 6);
    }

    #[test]
    fn strstr_not_found() {
        assert!(strstr(b"hello\0".as_ptr(), b"xyz\0".as_ptr()).is_null());
    }

    #[test]
    fn strstr_empty_needle() {
        let h = b"hello\0";
        let p = strstr(h.as_ptr(), b"\0".as_ptr());
        assert_eq!(unsafe { p.offset_from(h.as_ptr()) }, 0);
    }
}