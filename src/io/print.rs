/// `print!` / `println!` — formatted output to VGA display.

/// Print to VGA display. Use for user-facing output.
#[macro_export]
macro_rules! print {
    ($fmt:expr) => {
        $crate::io::print_engine::write_display($fmt, &[])
    };
    ($fmt:expr, $($arg:expr),* $(,)?) => {
        $crate::io::print_engine::write_display($fmt, &[
            $($crate::io::print_engine::PrintArg::from($arg)),*
        ])
    };
}

/// Print to VGA display with trailing newline.
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($fmt:expr) => {
        $crate::io::print_engine::write_display(concat!($fmt, "\n"), &[])
    };
    ($fmt:expr, $($arg:expr),* $(,)?) => {
        $crate::io::print_engine::write_display(concat!($fmt, "\n"), &[
            $($crate::io::print_engine::PrintArg::from($arg)),*
        ])
    };
}
