/// Shared format engine for `print!` and `printk!`.
///
/// Both macro families share the same runtime format parser and
/// number-to-string (itoa) routines.  The only difference is the
/// output *sink*: VGA display vs. kernel log ring buffer.
///
/// Supported format specifiers:
///   {}    — default (decimal for integers, as-is for strings/chars/bools)
///   {:x}  — hexadecimal lowercase       {:#x} — with "0x" prefix
///   {:X}  — hexadecimal uppercase       {:#X} — with "0X" prefix
///   {:b}  — binary                      {:#b} — with "0b" prefix
///   {:o}  — octal                       {:#o} — with "0o" prefix
///   {{    — literal '{'
///   }}    — literal '}'
///
use crate::io::{display, klog};

// ──────────────────────────────────────────────
//  Output sink
// ──────────────────────────────────────────────

#[derive(Copy, Clone, PartialEq)]
enum Sink {
    Display,
    Klog,
}

#[inline]
fn sink_char(c: u8, sink: Sink) {
    match sink {
        Sink::Display => display::put_char(c),
        Sink::Klog => klog::log_byte(c),
    }
}

#[inline]
fn sink_str(s: &str, sink: Sink) {
    match sink {
        Sink::Display => display::put_str(s),
        Sink::Klog => klog::log_str(s),
    }
}

#[inline]
fn sink_bytes(bytes: &[u8], sink: Sink) {
    match sink {
        Sink::Display => display::put_bytes(bytes),
        Sink::Klog => klog::log_bytes(bytes),
    }
}

// ──────────────────────────────────────────────
//  PrintArg — type-erased argument (like va_arg)
// ──────────────────────────────────────────────

#[derive(Copy, Clone)]
pub enum PrintArg<'a> {
    Str(&'a str),
    Bytes(&'a [u8]),
    Char(u8),
    I32(i32),
    U32(u32),
    Usize(usize),
    Bool(bool),
}

impl<'a> From<&'a str> for PrintArg<'a> {
    fn from(v: &'a str) -> Self { PrintArg::Str(v) }
}
impl<'a> From<&'a [u8]> for PrintArg<'a> {
    fn from(v: &'a [u8]) -> Self { PrintArg::Bytes(v) }
}
impl<'a> From<u8> for PrintArg<'a> {
    fn from(v: u8) -> Self { PrintArg::U32(v as u32) }
}
impl<'a> From<u16> for PrintArg<'a> {
    fn from(v: u16) -> Self { PrintArg::U32(v as u32) }
}
impl<'a> From<u32> for PrintArg<'a> {
    fn from(v: u32) -> Self { PrintArg::U32(v) }
}
impl<'a> From<i8> for PrintArg<'a> {
    fn from(v: i8) -> Self { PrintArg::I32(v as i32) }
}
impl<'a> From<i16> for PrintArg<'a> {
    fn from(v: i16) -> Self { PrintArg::I32(v as i32) }
}
impl<'a> From<i32> for PrintArg<'a> {
    fn from(v: i32) -> Self { PrintArg::I32(v) }
}
impl<'a> From<usize> for PrintArg<'a> {
    fn from(v: usize) -> Self { PrintArg::Usize(v) }
}
impl<'a> From<bool> for PrintArg<'a> {
    fn from(v: bool) -> Self { PrintArg::Bool(v) }
}
impl<'a> From<char> for PrintArg<'a> {
    fn from(v: char) -> Self { PrintArg::Char(v as u8) }
}

// ──────────────────────────────────────────────
//  itoa — number → string on a stack buffer
// ──────────────────────────────────────────────

const ITOA_BUF_SIZE: usize = 34; // 32-bit binary + sign

#[derive(Copy, Clone)]
enum FmtSpec {
    Default, Hex, HexUpper, Binary, Octal,
    HexAlt, HexUpperAlt, BinaryAlt, OctalAlt,
}

fn u32_to_base(mut val: u32, base: u32, uppercase: bool, buf: &mut [u8; ITOA_BUF_SIZE]) -> usize {
    let ptr = buf.as_mut_ptr();
    if val == 0 {
        unsafe { *ptr.add(ITOA_BUF_SIZE - 1) = b'0'; }
        return ITOA_BUF_SIZE - 1;
    }
    let mut i = ITOA_BUF_SIZE;
    while val > 0 {
        i -= 1;
        let digit = (val % base) as u8;
        let c = if digit < 10 {
            b'0' + digit
        } else if uppercase {
            b'A' + (digit - 10)
        } else {
            b'a' + (digit - 10)
        };
        unsafe { *ptr.add(i) = c; }
        val /= base;
    }
    i
}

fn put_buf_range(buf: &[u8; ITOA_BUF_SIZE], start: usize, sink: Sink) {
    let ptr = buf.as_ptr();
    let mut i = start;
    while i < ITOA_BUF_SIZE {
        unsafe { sink_char(*ptr.add(i), sink); }
        i += 1;
    }
}

fn fmt_u32(val: u32, spec: FmtSpec, sink: Sink) {
    let mut buf = [0u8; ITOA_BUF_SIZE];
    match spec {
        FmtSpec::Default => {
            let s = u32_to_base(val, 10, false, &mut buf);
            put_buf_range(&buf, s, sink);
        }
        FmtSpec::Hex => {
            let s = u32_to_base(val, 16, false, &mut buf);
            put_buf_range(&buf, s, sink);
        }
        FmtSpec::HexUpper => {
            let s = u32_to_base(val, 16, true, &mut buf);
            put_buf_range(&buf, s, sink);
        }
        FmtSpec::Binary => {
            let s = u32_to_base(val, 2, false, &mut buf);
            put_buf_range(&buf, s, sink);
        }
        FmtSpec::Octal => {
            let s = u32_to_base(val, 8, false, &mut buf);
            put_buf_range(&buf, s, sink);
        }
        FmtSpec::HexAlt => {
            sink_str("0x", sink);
            let s = u32_to_base(val, 16, false, &mut buf);
            put_buf_range(&buf, s, sink);
        }
        FmtSpec::HexUpperAlt => {
            sink_str("0X", sink);
            let s = u32_to_base(val, 16, true, &mut buf);
            put_buf_range(&buf, s, sink);
        }
        FmtSpec::BinaryAlt => {
            sink_str("0b", sink);
            let s = u32_to_base(val, 2, false, &mut buf);
            put_buf_range(&buf, s, sink);
        }
        FmtSpec::OctalAlt => {
            sink_str("0o", sink);
            let s = u32_to_base(val, 8, false, &mut buf);
            put_buf_range(&buf, s, sink);
        }
    }
}

fn fmt_i32(val: i32, spec: FmtSpec, sink: Sink) {
    if val < 0 {
        sink_char(b'-', sink);
        fmt_u32((!(val as u32)).wrapping_add(1), spec, sink);
    } else {
        fmt_u32(val as u32, spec, sink);
    }
}

// ──────────────────────────────────────────────
//  Format string parser
// ──────────────────────────────────────────────

fn fmt_arg(arg: &PrintArg, spec: FmtSpec, sink: Sink) {
    match arg {
        PrintArg::Str(s)    => sink_str(s, sink),
        PrintArg::Bytes(b)  => sink_bytes(b, sink),
        PrintArg::Char(c)   => sink_char(*c, sink),
        PrintArg::I32(v)    => fmt_i32(*v, spec, sink),
        PrintArg::U32(v)    => fmt_u32(*v, spec, sink),
        PrintArg::Usize(v)  => fmt_u32(*v as u32, spec, sink),
        PrintArg::Bool(v)   => sink_str(if *v { "true" } else { "false" }, sink),
    }
}

fn parse_spec(bytes: *const u8, start: usize, end: usize) -> FmtSpec {
    let len = end - start;
    if len == 0 { return FmtSpec::Default; }
    unsafe {
        if *bytes.add(start) != b':' { return FmtSpec::Default; }
        if len == 2 {
            return match *bytes.add(start + 1) {
                b'x' => FmtSpec::Hex,    b'X' => FmtSpec::HexUpper,
                b'b' => FmtSpec::Binary, b'o' => FmtSpec::Octal,
                _ => FmtSpec::Default,
            };
        }
        if len == 3 && *bytes.add(start + 1) == b'#' {
            return match *bytes.add(start + 2) {
                b'x' => FmtSpec::HexAlt,    b'X' => FmtSpec::HexUpperAlt,
                b'b' => FmtSpec::BinaryAlt, b'o' => FmtSpec::OctalAlt,
                _ => FmtSpec::Default,
            };
        }
    }
    FmtSpec::Default
}

/// Core format engine — parses format string at runtime like C's printf.
fn _format(fmt: &str, args: &[PrintArg], sink: Sink) {
    let bytes = fmt.as_ptr();
    let len = fmt.len();
    let args_ptr = args.as_ptr();
    let args_len = args.len();
    let mut i: usize = 0;
    let mut arg_idx: usize = 0;

    while i < len {
        let ch = unsafe { *bytes.add(i) };

        if ch == b'{' {
            if i + 1 < len && unsafe { *bytes.add(i + 1) } == b'{' {
                sink_char(b'{', sink); i += 2; continue;
            }
            let spec_start = i + 1;
            let mut j = spec_start;
            while j < len && unsafe { *bytes.add(j) } != b'}' { j += 1; }
            let spec = parse_spec(bytes, spec_start, j);
            if arg_idx < args_len {
                fmt_arg(unsafe { &*args_ptr.add(arg_idx) }, spec, sink);
                arg_idx += 1;
            }
            i = j + 1; continue;
        }
        if ch == b'}' && i + 1 < len && unsafe { *bytes.add(i + 1) } == b'}' {
            sink_char(b'}', sink); i += 2; continue;
        }
        sink_char(ch, sink);
        i += 1;
    }
}

// ──────────────────────────────────────────────
//  Public entry points
// ──────────────────────────────────────────────

/// Writes formatted output to the VGA display.
pub fn _print(fmt: &str, args: &[PrintArg]) {
    _format(fmt, args, Sink::Display);
}

/// Writes formatted output to the kernel log ring buffer.
pub fn _printk(fmt: &str, args: &[PrintArg]) {
    _format(fmt, args, Sink::Klog);
}
