/// `printk!` / `printkln!` — formatted output to kernel log buffer.
///
/// Output is stored in a ring buffer and can be retrieved with `dmesg`.

/// Print to kernel log buffer only.
#[macro_export]
macro_rules! printk {
    ($fmt:expr) => {
        $crate::io::print_engine::write_klog($fmt, &[])
    };
    ($fmt:expr, $($arg:expr),* $(,)?) => {
        $crate::io::print_engine::write_klog($fmt, &[
            $($crate::io::print_engine::PrintArg::from($arg)),*
        ])
    };
}

/// Print to kernel log buffer with trailing newline.
#[macro_export]
macro_rules! printkln {
    () => { $crate::printk!("\n") };
    ($fmt:expr) => {
        $crate::io::print_engine::write_klog(concat!($fmt, "\n"), &[])
    };
    ($fmt:expr, $($arg:expr),* $(,)?) => {
        $crate::io::print_engine::write_klog(concat!($fmt, "\n"), &[
            $($crate::io::print_engine::PrintArg::from($arg)),*
        ])
    };
}
