use crate::io::klog;
use crate::println;

pub fn dmesg(argv: &'static [&'static [u8]]) {
    // dmesg -c : dump and clear
    let clear = argv.len() > 1 && unsafe { *argv.get_unchecked(1) } == b"-c";

    klog::dump();

    if clear {
        klog::clear();
        println!("[klog cleared]");
    }
}
