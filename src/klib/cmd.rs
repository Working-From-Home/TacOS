//  Minimal command-line flag parser for shell builtins.
//
//  # Usage
//
//  ```ignore
//  use crate::klib::cmd::{Cmd, Flag};
//
//  let mut flags = [
//      Flag::short(b'n'),                  // -n
//      Flag::short(b'e'),                  // -e
//      Flag::long(b"verbose"),             // --verbose
//      Flag::both(b'h', b"help"),          // -h / --help
//  ];
//  let cmd = Cmd::parse(args, &mut flags);
//
//  let no_newline = cmd.get(b'n');
//  let verbose    = cmd.get_long(b"verbose");
//  let help       = cmd.get(b'h');  // or cmd.get_long(b"help")
//  let rest       = cmd.args();
//  ```

/// A boolean flag with optional short (`-x`) and/or long (`--name`) form.
pub struct Flag {
    pub short: u8,            // 0 means no short form
    pub long: &'static [u8],  // empty means no long form
    pub set: bool,
}

impl Flag {
    /// Flag with short form only: `-x`
    pub const fn short(c: u8) -> Self {
        Flag { short: c, long: b"", set: false }
    }

    /// Flag with long form only: `--name`
    pub const fn long(name: &'static [u8]) -> Self {
        Flag { short: 0, long: name, set: false }
    }

    /// Flag with both short and long forms: `-x` / `--name`
    pub const fn both(c: u8, name: &'static [u8]) -> Self {
        Flag { short: c, long: name, set: false }
    }

    /// Backwards-compatible alias for `Flag::short`.
    pub const fn new(c: u8) -> Self {
        Self::short(c)
    }
}

/// Parsed command result: holds references to parsed flags and the
/// remaining positional arguments.
pub struct Cmd<'a> {
    flags: &'a [Flag],
    rest: &'a [u8],
}

impl<'a> Cmd<'a> {
    /// Parses `args` (full line including command name).
    /// Skips the first word (command name), then processes flag tokens.
    ///
    /// Supported forms:
    /// - `-abc`        short flags (each char must be registered)
    /// - `--name`      long flag (must match a registered long name)
    /// - `--`          stops flag parsing
    ///
    /// Unrecognized tokens stop flag parsing (treated as positional args).
    pub fn parse(args: &'a [u8], flags: &'a mut [Flag]) -> Self {
        let len = args.len();
        let mut i: usize = 0;

        // Skip command name (first word)
        while i < len && unsafe { *args.get_unchecked(i) } != b' ' {
            i += 1;
        }
        // Skip space after command
        if i < len {
            i += 1;
        }

        // Parse flag tokens
        while i < len {
            // Must start with '-'
            if unsafe { *args.get_unchecked(i) } != b'-' {
                break;
            }

            // Check for "--" prefix
            if i + 1 < len && unsafe { *args.get_unchecked(i + 1) } == b'-' {
                let after = i + 2;
                // Bare "--" ends flag parsing
                if after >= len || unsafe { *args.get_unchecked(after) } == b' ' {
                    i = after;
                    if i < len {
                        i += 1;
                    }
                    break;
                }

                // Long flag: --name
                let name_start = after;
                let mut name_end = after;
                while name_end < len && unsafe { *args.get_unchecked(name_end) } != b' ' {
                    name_end += 1;
                }

                let mut matched = false;
                let mut j = 0;
                while j < flags.len() {
                    if flags[j].long.len() > 0
                        && bytes_equal_range(args, name_start, name_end, flags[j].long)
                    {
                        flags[j].set = true;
                        matched = true;
                        break;
                    }
                    j += 1;
                }

                if !matched {
                    break; // unknown long flag, stop parsing
                }

                i = name_end;
                if i < len && unsafe { *args.get_unchecked(i) } == b' ' {
                    i += 1;
                }
                continue;
            }

            let tok_start = i;
            i += 1; // skip '-'

            // Bare "-" is not a flag
            if i >= len || unsafe { *args.get_unchecked(i) } == b' ' {
                i = tok_start;
                break;
            }

            // Short flags: all chars in this token must be known
            let flag_start = i;
            let mut valid = true;
            while i < len && unsafe { *args.get_unchecked(i) } != b' ' {
                let c = unsafe { *args.get_unchecked(i) };
                let mut found = false;
                let mut j = 0;
                while j < flags.len() {
                    if flags[j].short == c {
                        found = true;
                        break;
                    }
                    j += 1;
                }
                if !found {
                    valid = false;
                    break;
                }
                i += 1;
            }

            if !valid {
                i = tok_start;
                break;
            }

            // Apply short flags
            let mut k = flag_start;
            while k < i {
                let c = unsafe { *args.get_unchecked(k) };
                let mut j = 0;
                while j < flags.len() {
                    if flags[j].short == c {
                        flags[j].set = true;
                    }
                    j += 1;
                }
                k += 1;
            }

            // Skip space after flag token
            if i < len && unsafe { *args.get_unchecked(i) } == b' ' {
                i += 1;
            }
        }

        let rest = if i <= len {
            unsafe { args.get_unchecked(i..len) }
        } else {
            unsafe { args.get_unchecked(len..len) }
        };

        Cmd { flags, rest }
    }

    /// Check if a short flag was set.
    pub fn get(&self, short: u8) -> bool {
        let mut i = 0;
        while i < self.flags.len() {
            if self.flags[i].short == short {
                return self.flags[i].set;
            }
            i += 1;
        }
        false
    }

    /// Check if a long flag was set.
    pub fn get_long(&self, name: &[u8]) -> bool {
        let mut i = 0;
        while i < self.flags.len() {
            if self.flags[i].long.len() > 0
                && self.flags[i].long.len() == name.len()
            {
                let mut eq = true;
                let mut j = 0;
                while j < name.len() {
                    if self.flags[i].long[j] != name[j] {
                        eq = false;
                        break;
                    }
                    j += 1;
                }
                if eq && self.flags[i].set {
                    return true;
                }
            }
            i += 1;
        }
        false
    }

    /// Backwards-compatible alias for `get`.
    pub fn flag(&self, short: u8) -> bool {
        self.get(short)
    }

    /// Returns the remaining positional arguments after flags.
    pub fn args(&self) -> &'a [u8] {
        self.rest
    }
}

/// Compare a sub-range of `haystack[start..end]` to `needle`.
fn bytes_equal_range(haystack: &[u8], start: usize, end: usize, needle: &[u8]) -> bool {
    let range_len = end - start;
    if range_len != needle.len() {
        return false;
    }
    let mut i = 0;
    while i < range_len {
        if unsafe { *haystack.get_unchecked(start + i) } != unsafe { *needle.get_unchecked(i) } {
            return false;
        }
        i += 1;
    }
    true
}
