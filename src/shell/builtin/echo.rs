use crate::klib::cmd::{Cmd, Flag};
use crate::{printk, printkln};

pub fn echo(args: &[u8]) {
    let mut flags = [
        Flag::new(b'n'),
        Flag::new(b'e'),
        Flag::new(b'E'),
    ];
    let cmd = Cmd::parse(args, &mut flags);

    let newline = !cmd.flag(b'n');
    let escapes = cmd.flag(b'e') && !cmd.flag(b'E');
    let rest = cmd.args();

    if !rest.is_empty() {
        if escapes {
            print_with_escapes(rest);
        } else {
            printk!("{}", rest);
        }
    }

    if newline {
        printkln!();
    }
}

fn print_with_escapes(s: &[u8]) {
    let mut i = 0;
    while i < s.len() {
        let c = unsafe { *s.get_unchecked(i) };
        if c == b'\\' && i + 1 < s.len() {
            let next = unsafe { *s.get_unchecked(i + 1) };
            match next {
                b'n' => { printk!("\n"); i += 2; }
                b't' => { printk!("\t"); i += 2; }
                b'r' => { printk!("\r"); i += 2; }
                b'\\' => { printk!("\\"); i += 2; }
                b'a' => { printk!("\x07"); i += 2; }
                b'b' => { printk!("\x08"); i += 2; }
                b'f' => { printk!("\x0C"); i += 2; }
                b'v' => { printk!("\x0B"); i += 2; }
                b'0' => {
                    i += 2;
                    let mut val: u8 = 0;
                    let mut count = 0;
                    while count < 3 && i < s.len() {
                        let d = unsafe { *s.get_unchecked(i) };
                        if d >= b'0' && d <= b'7' {
                            val = val * 8 + (d - b'0');
                            i += 1;
                            count += 1;
                        } else {
                            break;
                        }
                    }
                    crate::io::display::put_char(val);
                }
                b'x' => {
                    i += 2;
                    let mut val: u8 = 0;
                    let mut count = 0;
                    while count < 2 && i < s.len() {
                        let d = unsafe { *s.get_unchecked(i) };
                        let nibble = hex_digit(d);
                        if nibble <= 0x0F {
                            val = val * 16 + nibble;
                            i += 1;
                            count += 1;
                        } else {
                            break;
                        }
                    }
                    if count > 0 {
                        crate::io::display::put_char(val);
                    }
                }
                b'c' => { return; }
                _ => {
                    printk!("\\");
                    crate::io::display::put_char(next);
                    i += 2;
                }
            }
        } else {
            crate::io::display::put_char(c);
            i += 1;
        }
    }
}

fn hex_digit(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0xFF,
    }
}
