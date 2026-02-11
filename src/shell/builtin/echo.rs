use crate::{print, println};

pub fn echo(argv: &'static [&'static [u8]]) {
    let mut start = 1;
    let mut newline = true;

    if argv.len() > 1 && unsafe { *argv.get_unchecked(1) } == b"-n" {
        start = 2;
        newline = false;
    }
    let mut i = start;
    while i < argv.len() {
        if i > start {
            print!(" ");
        }
        print!("{}", unsafe { *argv.get_unchecked(i) });
        i += 1;
    }
    if newline {
        println!();
    }
}
