/// Shared print engine for `print!` and `printk!`.
///
/// Supports format specifiers:
///   {}    — default (decimal for integers, as-is for strings/chars/bools)
///   {:x}  — hexadecimal lowercase       {:#x} — with "0x" prefix
///   {:X}  — hexadecimal uppercase       {:#X} — with "0X" prefix
///   {:b}  — binary                      {:#b} — with "0b" prefix
///   {:o}  — octal                       {:#o} — with "0o" prefix
///   {{    — literal '{'
///   }}    — literal '}'

use crate::io::{display, klog};

// ──────────────────────────────────────────────
//  Output sink — controls where output is sent
// ──────────────────────────────────────────────

/// Selects which output backends receive the formatted text.
///
/// To add a new backend (e.g. serial), add a variant here,
/// update `to_display()` / `to_klog()` (and add `to_serial()`),
/// then update the three `emit_*` helpers below. Nothing else changes.
#[derive(Copy, Clone, PartialEq)]
pub enum Sink {
    Display,    // VGA display only (user-facing output: echo, dmesg dump, …)
    Klog,       // Kernel log ring buffer only (silent logging, no screen output)
    Kernel,     // VGA display AND kernel log ring buffer (kernel messages: printk)
}

impl Sink {
    /// Should this sink write to the VGA display?
    #[inline(always)]
    fn to_display(self) -> bool {
        match self {
            Sink::Display | Sink::Kernel => true,
            Sink::Klog => false,
        }
    }

    /// Should this sink write to the kernel log ring buffer?
    #[inline(always)]
    fn to_klog(self) -> bool {
        match self {
            Sink::Klog | Sink::Kernel => true,
            Sink::Display => false,
        }
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
//  Sink-aware emit helpers
// ──────────────────────────────────────────────

/// Emit a single byte (raw glyph, no control-char interpretation).
#[inline]
fn emit_raw(c: u8, sink: Sink) {
    if sink.to_display() { display::put_char(c); }
    if sink.to_klog()    { klog::log_byte(c); }
}

/// Emit a string (with control-char interpretation on the display side).
#[inline]
fn emit_str(s: &str, sink: Sink) {
    if sink.to_display() { display::put_str(s); }
    if sink.to_klog()    { klog::log_str(s); }
}

/// Emit a byte slice (with control-char interpretation on the display side).
#[inline]
fn emit_bytes(b: &[u8], sink: Sink) {
    if sink.to_display() { display::put_bytes(b); }
    if sink.to_klog()    { klog::log_bytes(b); }
}

/// Emit a single byte with control-char interpretation (\n, \t, etc.).
/// Used for literal characters from the format string.
#[inline]
fn emit_byte(c: u8, sink: Sink) {
    if sink.to_display() { display::write_byte(c, crate::drivers::vga::DEFAULT_COLOR); }
    if sink.to_klog()    { klog::log_byte(c); }
}

// ──────────────────────────────────────────────
//  itoa — number → string on a stack buffer
// ──────────────────────────────────────────────

const ITOA_BUF_SIZE: usize = 34; // 32-bit binary + sign

#[derive(Copy, Clone)]
enum Spec {
    Default,
    Hex,
    HexUpper,
    Binary,
    Octal,
    HexAlt,
    HexUpperAlt,
    BinaryAlt,
    OctalAlt,
}

impl Spec {
    fn params(self) -> (u32, bool, &'static str) {
        match self {
            Spec::Default      => (10, false, ""),
            Spec::Hex          => (16, false, ""),
            Spec::HexUpper     => (16, true,  ""),
            Spec::Binary       => ( 2, false, ""),
            Spec::Octal        => ( 8, false, ""),
            Spec::HexAlt       => (16, false, "0x"),
            Spec::HexUpperAlt  => (16, true,  "0X"),
            Spec::BinaryAlt    => ( 2, false, "0b"),
            Spec::OctalAlt     => ( 8, false, "0o"),
        }
    }
}

fn u32_to_base(mut val: u32, base: u32, uppercase: bool, buf: &mut [u8; ITOA_BUF_SIZE]) -> usize {
    if val == 0 {
        unsafe { *buf.get_unchecked_mut(ITOA_BUF_SIZE - 1) = b'0'; }
        return ITOA_BUF_SIZE - 1;
    }
    let mut i = ITOA_BUF_SIZE;
    while val > 0 {
        i -= 1;
        let digit = (val % base) as u8;
        unsafe {
            *buf.get_unchecked_mut(i) = if digit < 10 {
                b'0' + digit
            } else if uppercase {
                b'A' + (digit - 10)
            } else {
                b'a' + (digit - 10)
            };
        }
        val /= base;
    }
    i
}

fn emit_buf(buf: &[u8; ITOA_BUF_SIZE], start: usize, sink: Sink) {
    let mut i = start;
    while i < ITOA_BUF_SIZE {
        emit_raw(unsafe { *buf.get_unchecked(i) }, sink);
        i += 1;
    }
}

fn write_u32(val: u32, spec: Spec, sink: Sink) {
    let (base, uppercase, prefix) = spec.params();
    if !prefix.is_empty() {
        emit_str(prefix, sink);
    }
    let mut buf = [0u8; ITOA_BUF_SIZE];
    let start = u32_to_base(val, base, uppercase, &mut buf);
    emit_buf(&buf, start, sink);
}

fn write_i32(val: i32, spec: Spec, sink: Sink) {
    if val < 0 {
        emit_raw(b'-', sink);
        write_u32(val.wrapping_neg() as u32, spec, sink);
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
        PrintArg::Char(c)   => emit_raw(*c, sink),
        PrintArg::I32(v)    => write_i32(*v, spec, sink),
        PrintArg::U32(v)    => write_u32(*v, spec, sink),
        PrintArg::Usize(v)  => write_u32(*v as u32, spec, sink),
        PrintArg::Bool(v)   => emit_str(if *v { "true" } else { "false" }, sink),
    }
}

fn parse_spec(fmt: &[u8], start: usize, end: usize) -> Spec {
    let len = end - start;
    if len == 0 {
        return Spec::Default;
    }
    if unsafe { *fmt.get_unchecked(start) } != b':' {
        return Spec::Default;
    }
    if len == 2 {
        return match unsafe { *fmt.get_unchecked(start + 1) } {
            b'x' => Spec::Hex,
            b'X' => Spec::HexUpper,
            b'b' => Spec::Binary,
            b'o' => Spec::Octal,
            _ => Spec::Default,
        };
    }
    if len == 3 && unsafe { *fmt.get_unchecked(start + 1) } == b'#' {
        return match unsafe { *fmt.get_unchecked(start + 2) } {
            b'x' => Spec::HexAlt,
            b'X' => Spec::HexUpperAlt,
            b'b' => Spec::BinaryAlt,
            b'o' => Spec::OctalAlt,
            _ => Spec::Default,
        };
    }
    Spec::Default
}

/// Core print engine — parses format string and emits to the selected sink(s).
fn format(fmt: &str, args: &[PrintArg], sink: Sink) {
    let bytes = fmt.as_bytes();
    let len = bytes.len();
    let mut i: usize = 0;
    let mut arg_idx: usize = 0;

    while i < len {
        let ch = unsafe { *bytes.get_unchecked(i) };

        if ch == b'{' {
            if i + 1 < len && unsafe { *bytes.get_unchecked(i + 1) } == b'{' {
                emit_raw(b'{', sink);
                i += 2;
                continue;
            }
            let spec_start = i + 1;
            let mut j = spec_start;
            while j < len && unsafe { *bytes.get_unchecked(j) } != b'}' {
                j += 1;
            }
            let spec = parse_spec(bytes, spec_start, j);
            if arg_idx < args.len() {
                write_arg(unsafe { args.get_unchecked(arg_idx) }, spec, sink);
                arg_idx += 1;
            }
            i = j + 1;
            continue;
        }
        if ch == b'}' && i + 1 < len && unsafe { *bytes.get_unchecked(i + 1) } == b'}' {
            emit_raw(b'}', sink);
            i += 2;
            continue;
        }
        // Regular character — needs control-char interpretation (\n, \t, etc.)
        emit_byte(ch, sink);
        i += 1;
    }
}

// ──────────────────────────────────────────────
//  Public entry points
// ──────────────────────────────────────────────

/// Writes formatted output to VGA display only (user-facing output).
pub fn write_display(fmt: &str, args: &[PrintArg]) {
    format(fmt, args, Sink::Display);
}

/// Writes formatted output to kernel log ring buffer only (no screen output).
pub fn write_klog(fmt: &str, args: &[PrintArg]) {
    format(fmt, args, Sink::Klog);
}

/// Writes formatted output to both VGA display and kernel log ring buffer.
pub fn write_kernel(fmt: &str, args: &[PrintArg]) {
    format(fmt, args, Sink::Kernel);
}
