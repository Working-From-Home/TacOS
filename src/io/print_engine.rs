/// Shared print engine for `print!` and `printk!`.
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
fn emit_char(c: u8, sink: Sink) {
    match sink {
        Sink::Display => display::put_char(c),
        Sink::Klog => klog::log_byte(c),
    }
}

#[inline]
fn emit_str(s: &str, sink: Sink) {
    match sink {
        Sink::Display => display::put_str(s),
        Sink::Klog => klog::log_str(s),
    }
}

#[inline]
fn emit_bytes(bytes: &[u8], sink: Sink) {
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
enum Spec {
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

fn write_buf(buf: &[u8; ITOA_BUF_SIZE], start: usize, sink: Sink) {
    let ptr = buf.as_ptr();
    let mut i = start;
    while i < ITOA_BUF_SIZE {
        unsafe { emit_char(*ptr.add(i), sink); }
        i += 1;
    }
}

fn write_u32(val: u32, spec: Spec, sink: Sink) {
    let mut buf = [0u8; ITOA_BUF_SIZE];
    match spec {
        Spec::Default => {
            let s = u32_to_base(val, 10, false, &mut buf);
            write_buf(&buf, s, sink);
        }
        Spec::Hex => {
            let s = u32_to_base(val, 16, false, &mut buf);
            write_buf(&buf, s, sink);
        }
        Spec::HexUpper => {
            let s = u32_to_base(val, 16, true, &mut buf);
            write_buf(&buf, s, sink);
        }
        Spec::Binary => {
            let s = u32_to_base(val, 2, false, &mut buf);
            write_buf(&buf, s, sink);
        }
        Spec::Octal => {
            let s = u32_to_base(val, 8, false, &mut buf);
            write_buf(&buf, s, sink);
        }
        Spec::HexAlt => {
            emit_str("0x", sink);
            let s = u32_to_base(val, 16, false, &mut buf);
            write_buf(&buf, s, sink);
        }
        Spec::HexUpperAlt => {
            emit_str("0X", sink);
            let s = u32_to_base(val, 16, true, &mut buf);
            write_buf(&buf, s, sink);
        }
        Spec::BinaryAlt => {
            emit_str("0b", sink);
            let s = u32_to_base(val, 2, false, &mut buf);
            write_buf(&buf, s, sink);
        }
        Spec::OctalAlt => {
            emit_str("0o", sink);
            let s = u32_to_base(val, 8, false, &mut buf);
            write_buf(&buf, s, sink);
        }
    }
}

fn write_i32(val: i32, spec: Spec, sink: Sink) {
    if val < 0 {
        emit_char(b'-', sink);
        write_u32((!(val as u32)).wrapping_add(1), spec, sink);
    } else {
        write_u32(val as u32, spec, sink);
    }
}

// ──────────────────────────────────────────────
//  Format string parser
// ──────────────────────────────────────────────

fn write_arg(arg: &PrintArg, spec: Spec, sink: Sink) {
    match arg {
        PrintArg::Str(s)    => emit_str(s, sink),
        PrintArg::Bytes(b)  => emit_bytes(b, sink),
        PrintArg::Char(c)   => emit_char(*c, sink),
        PrintArg::I32(v)    => write_i32(*v, spec, sink),
        PrintArg::U32(v)    => write_u32(*v, spec, sink),
        PrintArg::Usize(v)  => write_u32(*v as u32, spec, sink),
        PrintArg::Bool(v)   => emit_str(if *v { "true" } else { "false" }, sink),
    }
}

fn parse_spec(bytes: *const u8, start: usize, end: usize) -> Spec {
    let len = end - start;
    if len == 0 { return Spec::Default; }
    unsafe {
        if *bytes.add(start) != b':' { return Spec::Default; }
        if len == 2 {
            return match *bytes.add(start + 1) {
                b'x' => Spec::Hex,    b'X' => Spec::HexUpper,
                b'b' => Spec::Binary, b'o' => Spec::Octal,
                _ => Spec::Default,
            };
        }
        if len == 3 && *bytes.add(start + 1) == b'#' {
            return match *bytes.add(start + 2) {
                b'x' => Spec::HexAlt,    b'X' => Spec::HexUpperAlt,
                b'b' => Spec::BinaryAlt, b'o' => Spec::OctalAlt,
                _ => Spec::Default,
            };
        }
    }
    Spec::Default
}

/// Core print engine — parses format string at runtime like C's printf.
fn format(fmt: &str, args: &[PrintArg], sink: Sink) {
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
                emit_char(b'{', sink); i += 2; continue;
            }
            let spec_start = i + 1;
            let mut j = spec_start;
            while j < len && unsafe { *bytes.add(j) } != b'}' { j += 1; }
            let spec = parse_spec(bytes, spec_start, j);
            if arg_idx < args_len {
                write_arg(unsafe { &*args_ptr.add(arg_idx) }, spec, sink);
                arg_idx += 1;
            }
            i = j + 1; continue;
        }
        if ch == b'}' && i + 1 < len && unsafe { *bytes.add(i + 1) } == b'}' {
            emit_char(b'}', sink); i += 2; continue;
        }
        emit_char(ch, sink);
        i += 1;
    }
}

// ──────────────────────────────────────────────
//  Public entry points
// ──────────────────────────────────────────────

/// Writes formatted output to the VGA display.
pub fn write_display(fmt: &str, args: &[PrintArg]) {
    format(fmt, args, Sink::Display);
}

/// Writes formatted output to the kernel log ring buffer.
pub fn write_klog(fmt: &str, args: &[PrintArg]) {
    format(fmt, args, Sink::Klog);
}
