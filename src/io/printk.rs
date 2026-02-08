/// printk — kernel formatted print, implemented from scratch.
///
/// In C/C++, printk uses variadic functions (va_list, va_arg) to handle
/// format strings like "%s", "%d", "%x". Rust has no variadic functions
/// in no_std, so we use a different strategy:
///
/// 1. A `printk!` macro collects all arguments into an array of `PrintArg`
///    (a type-erased enum), similar to how va_list packs args on the stack.
/// 2. A runtime format parser walks the format string, finds `{}` / `{:x}`
///    specifiers, and pulls the next `PrintArg` to format and display.
/// 3. Number-to-string conversions (itoa) are done on a stack buffer —
///    zero allocation, no heap, no core::fmt.
///
/// All array accesses use unsafe pointer arithmetic to avoid pulling in
/// `core::panicking::panic_bounds_check` (which doesn't exist in our kernel).
///
/// Supported format specifiers:
///   {}    — default (decimal for integers, as-is for strings/chars/bools)
///   {:x}  — hexadecimal lowercase
///   {:X}  — hexadecimal uppercase
///   {:b}  — binary
///   {:o}  — octal
///   {:#x} — hex with "0x" prefix
///   {:#X} — hex with "0X" prefix
///   {:#b} — binary with "0b" prefix
///   {:#o} — octal with "0o" prefix
///   {{    — literal '{'
///   }}    — literal '}'
///
/// See this another interesting approach (compile time macro expansion): https://github.com/kennystrawnmusic/printk/blob/master/src/lib.rs
/// Ours works at runtime (like C's printf/va_list)
use crate::io::display;

// ──────────────────────────────────────────────
//  PrintArg — type-erased argument (like va_arg)
// ──────────────────────────────────────────────

/// Represents a single argument passed to printk.
/// This is our Rust-safe replacement for C's va_list entries.
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

// From impls — so the macro can do `PrintArg::from(expr)` for each arg
impl<'a> From<&'a str> for PrintArg<'a> {
    fn from(v: &'a str) -> Self {
        PrintArg::Str(v)
    }
}
impl<'a> From<&'a [u8]> for PrintArg<'a> {
    fn from(v: &'a [u8]) -> Self {
        PrintArg::Bytes(v)
    }
}
impl<'a> From<u8> for PrintArg<'a> {
    fn from(v: u8) -> Self {
        PrintArg::U32(v as u32)
    }
}
impl<'a> From<u16> for PrintArg<'a> {
    fn from(v: u16) -> Self {
        PrintArg::U32(v as u32)
    }
}
impl<'a> From<u32> for PrintArg<'a> {
    fn from(v: u32) -> Self {
        PrintArg::U32(v)
    }
}
impl<'a> From<i8> for PrintArg<'a> {
    fn from(v: i8) -> Self {
        PrintArg::I32(v as i32)
    }
}
impl<'a> From<i16> for PrintArg<'a> {
    fn from(v: i16) -> Self {
        PrintArg::I32(v as i32)
    }
}
impl<'a> From<i32> for PrintArg<'a> {
    fn from(v: i32) -> Self {
        PrintArg::I32(v)
    }
}
impl<'a> From<usize> for PrintArg<'a> {
    fn from(v: usize) -> Self {
        PrintArg::Usize(v)
    }
}
impl<'a> From<bool> for PrintArg<'a> {
    fn from(v: bool) -> Self {
        PrintArg::Bool(v)
    }
}
impl<'a> From<char> for PrintArg<'a> {
    fn from(v: char) -> Self {
        PrintArg::Char(v as u8)
    }
}

// ──────────────────────────────────────────────
//  Number → string conversion (itoa), on stack
// ──────────────────────────────────────────────

const ITOA_BUF_SIZE: usize = 34; // enough for 32-bit binary + sign

/// Format specifier parsed from the format string.
#[derive(Copy, Clone)]
enum FmtSpec {
    Default,     // {}
    Hex,         // {:x}
    HexUpper,    // {:X}
    Binary,      // {:b}
    Octal,       // {:o}
    HexAlt,      // {:#x}
    HexUpperAlt, // {:#X}
    BinaryAlt,   // {:#b}
    OctalAlt,    // {:#o}
}

/// Writes an unsigned 32-bit integer in the given base into a stack buffer.
/// Returns the start index of the formatted digits within `buf`.
/// The digits occupy buf[start..ITOA_BUF_SIZE].
///
/// All buffer writes use raw pointer arithmetic — no bounds checks.
fn u32_to_base(mut val: u32, base: u32, uppercase: bool, buf: &mut [u8; ITOA_BUF_SIZE]) -> usize {
    let ptr = buf.as_mut_ptr();
    if val == 0 {
        unsafe {
            *ptr.add(ITOA_BUF_SIZE - 1) = b'0';
        }
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
        unsafe {
            *ptr.add(i) = c;
        }
        val /= base;
    }
    i
}

/// Prints the digits from buf[start..ITOA_BUF_SIZE] using raw pointer access.
fn put_buf_range(buf: &[u8; ITOA_BUF_SIZE], start: usize) {
    let ptr = buf.as_ptr();
    let mut i = start;
    while i < ITOA_BUF_SIZE {
        unsafe {
            display::put_char(*ptr.add(i));
        }
        i += 1;
    }
}

fn print_u32(val: u32, spec: FmtSpec) {
    let mut buf = [0u8; ITOA_BUF_SIZE];

    match spec {
        FmtSpec::Default => {
            let start = u32_to_base(val, 10, false, &mut buf);
            put_buf_range(&buf, start);
        }
        FmtSpec::Hex => {
            let start = u32_to_base(val, 16, false, &mut buf);
            put_buf_range(&buf, start);
        }
        FmtSpec::HexUpper => {
            let start = u32_to_base(val, 16, true, &mut buf);
            put_buf_range(&buf, start);
        }
        FmtSpec::Binary => {
            let start = u32_to_base(val, 2, false, &mut buf);
            put_buf_range(&buf, start);
        }
        FmtSpec::Octal => {
            let start = u32_to_base(val, 8, false, &mut buf);
            put_buf_range(&buf, start);
        }
        FmtSpec::HexAlt => {
            display::put_str("0x");
            let start = u32_to_base(val, 16, false, &mut buf);
            put_buf_range(&buf, start);
        }
        FmtSpec::HexUpperAlt => {
            display::put_str("0x");
            let start = u32_to_base(val, 16, true, &mut buf);
            put_buf_range(&buf, start);
        }
        FmtSpec::BinaryAlt => {
            display::put_str("0b");
            let start = u32_to_base(val, 2, false, &mut buf);
            put_buf_range(&buf, start);
        }
        FmtSpec::OctalAlt => {
            display::put_str("0o");
            let start = u32_to_base(val, 8, false, &mut buf);
            put_buf_range(&buf, start);
        }
    }
}

fn print_i32(val: i32, spec: FmtSpec) {
    if val < 0 {
        display::put_char(b'-');
        // Handle i32::MIN carefully: -(i32::MIN) overflows in signed.
        // Two's complement: negate via bitwise NOT + 1, in u32 space.
        let abs = (!(val as u32)).wrapping_add(1);
        print_u32(abs, spec);
    } else {
        print_u32(val as u32, spec);
    }
}

fn print_usize(val: usize, spec: FmtSpec) {
    // On i686 target, usize is 32 bits
    print_u32(val as u32, spec);
}

// ──────────────────────────────────────────────
//  Format string parser (runtime, like C printf)
// ──────────────────────────────────────────────

fn print_arg(arg: &PrintArg, spec: FmtSpec) {
    match arg {
        PrintArg::Str(s) => display::put_str(s),
        PrintArg::Bytes(b) => display::put_bytes(b),
        PrintArg::Char(c) => display::put_char(*c),
        PrintArg::I32(v) => print_i32(*v, spec),
        PrintArg::U32(v) => print_u32(*v, spec),
        PrintArg::Usize(v) => print_usize(*v, spec),
        PrintArg::Bool(v) => {
            if *v {
                display::put_str("true")
            } else {
                display::put_str("false")
            }
        }
    }
}

/// Parses the content between '{' and '}' to determine the format spec.
/// `bytes` points to the full format string, `start..end` is the content
/// between '{' and '}'.
///
/// Uses raw pointer access throughout — no bounds checks.
fn parse_spec(bytes: *const u8, start: usize, end: usize) -> FmtSpec {
    let spec_len = end - start;

    if spec_len == 0 {
        return FmtSpec::Default; // {}
    }

    // Must start with ':'
    unsafe {
        if *bytes.add(start) != b':' {
            return FmtSpec::Default;
        }

        // {:x} {:X} {:b} {:o}
        if spec_len == 2 {
            return match *bytes.add(start + 1) {
                b'x' => FmtSpec::Hex,
                b'X' => FmtSpec::HexUpper,
                b'b' => FmtSpec::Binary,
                b'o' => FmtSpec::Octal,
                _ => FmtSpec::Default,
            };
        }

        // {:#x} {:#X} {:#b} {:#o}
        if spec_len == 3 && *bytes.add(start + 1) == b'#' {
            return match *bytes.add(start + 2) {
                b'x' => FmtSpec::HexAlt,
                b'X' => FmtSpec::HexUpperAlt,
                b'b' => FmtSpec::BinaryAlt,
                b'o' => FmtSpec::OctalAlt,
                _ => FmtSpec::Default,
            };
        }
    }

    FmtSpec::Default
}

/// Internal entry point called by the `printk!` macro.
/// Parses the format string at runtime and consumes args from the slice,
/// just like C's printf walks va_list.
///
/// All indexing uses raw pointer arithmetic to avoid bringing in
/// `core::panicking::panic_bounds_check`.
#[doc(hidden)]
pub fn _printk(fmt: &str, args: &[PrintArg]) {
    let bytes = fmt.as_ptr();
    let len = fmt.len();
    let args_ptr = args.as_ptr();
    let args_len = args.len();
    let mut i: usize = 0;
    let mut arg_idx: usize = 0;

    while i < len {
        let ch = unsafe { *bytes.add(i) };

        // Check for '{' — start of a format specifier
        if ch == b'{' {
            // Escaped '{{' → literal '{'
            if i + 1 < len && unsafe { *bytes.add(i + 1) } == b'{' {
                display::put_char(b'{');
                i += 2;
                continue;
            }

            // Find closing '}'
            let spec_start = i + 1;
            let mut j = spec_start;
            while j < len && unsafe { *bytes.add(j) } != b'}' {
                j += 1;
            }

            let spec = parse_spec(bytes, spec_start, j);

            // Consume one argument
            if arg_idx < args_len {
                let arg = unsafe { &*args_ptr.add(arg_idx) };
                print_arg(arg, spec);
                arg_idx += 1;
            }

            i = j + 1; // skip past '}'
            continue;
        }

        // Escaped '}}' → literal '}'
        if ch == b'}' && i + 1 < len && unsafe { *bytes.add(i + 1) } == b'}' {
            display::put_char(b'}');
            i += 2;
            continue;
        }

        // Regular character
        display::put_char(ch);
        i += 1;
    }
}

// ──────────────────────────────────────────────
//  Public macros
// ──────────────────────────────────────────────

/// Kernel print macro — replacement for C's printk.
///
/// Uses Rust macros (instead of va_list) to safely collect arguments
/// into a `PrintArg` array, then parses the format string at runtime.
///
/// # Format specifiers
/// - `{}`    — default (decimal for ints, string for &str)
/// - `{:x}`  — hex lowercase
/// - `{:X}`  — hex uppercase
/// - `{:b}`  — binary
/// - `{:o}`  — octal
/// - `{:#x}` — hex with 0x prefix
/// - `{:#b}` — binary with 0b prefix
/// - `{:#o}` — octal with 0o prefix
/// - `{{`    — literal '{'
/// - `}}`    — literal '}'
///
/// # Examples
/// ```
/// printk!("Hello, {}!\n", "TacOS");
/// printk!("Dec: {}, Hex: {:#x}, Bin: {:#b}\n", 42, 255, 10);
/// ```
#[macro_export]
macro_rules! printk {
    ($fmt:expr) => {
        $crate::io::printk::_printk($fmt, &[])
    };
    ($fmt:expr, $($arg:expr),* $(,)?) => {
        $crate::io::printk::_printk($fmt, &[
            $($crate::io::printk::PrintArg::from($arg)),*
        ])
    };
}

/// Like `printk!` but appends a newline.
#[macro_export]
macro_rules! printkln {
    () => {
        $crate::printk!("\n")
    };
    ($fmt:expr) => {
        $crate::io::printk::_printk(concat!($fmt, "\n"), &[])
    };
    ($fmt:expr, $($arg:expr),* $(,)?) => {
        $crate::io::printk::_printk(concat!($fmt, "\n"), &[
            $($crate::io::printk::PrintArg::from($arg)),*
        ])
    };
}
