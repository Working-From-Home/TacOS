#[no_mangle]
pub extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    unsafe {
        while i < n {
            *s.add(i) = c as u8;
            i += 1;
        }
    }
    s
}

#[no_mangle]
pub extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    unsafe {
        while i < n {
            *dest.add(i) = *src.add(i);
            i += 1;
        }
    }
    dest
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── memset ───────────────────────────────────────────

    #[test]
    fn memset_fill_zero() {
        let mut buf = [0xFFu8; 16];
        memset(buf.as_mut_ptr(), 0, buf.len());
        assert!(buf.iter().all(|&b| b == 0));
    }

    #[test]
    fn memset_fill_value() {
        let mut buf = [0u8; 16];
        memset(buf.as_mut_ptr(), 0xAB, buf.len());
        assert!(buf.iter().all(|&b| b == 0xAB));
    }

    #[test]
    fn memset_partial() {
        let mut buf = [0u8; 16];
        memset(buf.as_mut_ptr(), 0xFF, 4);
        assert_eq!(&buf[..4], &[0xFF; 4]);
        assert_eq!(&buf[4..], &[0u8; 12]);
    }

    #[test]
    fn memset_zero_len() {
        let mut buf = [0xAA; 8];
        memset(buf.as_mut_ptr(), 0, 0);
        assert!(buf.iter().all(|&b| b == 0xAA));
    }

    #[test]
    fn memset_truncates_c_to_u8() {
        let mut buf = [0u8; 4];
        // 0x1FF should be truncated to 0xFF
        memset(buf.as_mut_ptr(), 0x1FF, buf.len());
        assert!(buf.iter().all(|&b| b == 0xFF));
    }

    // ── memcpy ───────────────────────────────────────────

    #[test]
    fn memcpy_basic() {
        let src = [1u8, 2, 3, 4, 5];
        let mut dst = [0u8; 5];
        memcpy(dst.as_mut_ptr(), src.as_ptr(), 5);
        assert_eq!(dst, src);
    }

    #[test]
    fn memcpy_partial() {
        let src = [10u8, 20, 30, 40, 50];
        let mut dst = [0u8; 5];
        memcpy(dst.as_mut_ptr(), src.as_ptr(), 3);
        assert_eq!(&dst[..3], &[10, 20, 30]);
        assert_eq!(&dst[3..], &[0, 0]);
    }

    #[test]
    fn memcpy_zero_len() {
        let src = [1u8, 2, 3];
        let mut dst = [0xFFu8; 3];
        memcpy(dst.as_mut_ptr(), src.as_ptr(), 0);
        assert!(dst.iter().all(|&b| b == 0xFF));
    }

    #[test]
    fn memcpy_large() {
        let src: [u8; 256] = core::array::from_fn(|i| i as u8);
        let mut dst = [0u8; 256];
        memcpy(dst.as_mut_ptr(), src.as_ptr(), 256);
        assert_eq!(dst, src);
    }

    #[test]
    fn memcpy_returns_dest() {
        let src = [1u8, 2];
        let mut dst = [0u8; 2];
        let ret = memcpy(dst.as_mut_ptr(), src.as_ptr(), 2);
        assert_eq!(ret, dst.as_mut_ptr());
    }

    #[test]
    fn memset_returns_s() {
        let mut buf = [0u8; 4];
        let ret = memset(buf.as_mut_ptr(), 0, 4);
        assert_eq!(ret, buf.as_mut_ptr());
    }
}