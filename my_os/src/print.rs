//! Printing

use crate::console;
use core::fmt;

pub fn _print(args: fmt::Arguments) {
    console::console().write_fmt(args).unwrap(); // use to interface to print to console
}

/// Prints without a newline.
/// Carbon copy from <https://doc.rust-lang.org/src/std/macros.rs.html>
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

/// Prints with a newline.
/// Carbon copy from <https://doc.rust-lang.org/src/std/macros.rs.html>
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_print(format_args_nl!($($arg)*));
    })
}

/// Prints an info, with a newline.
#[macro_export]
macro_rules! info {
    ($string:expr) => ({
        let timestamp = $crate::timer::time_manager().uptime();

        $crate::print::_print(format_args_nl!(
            concat!("[  {:>3}.{:06}] ", $string),
            timestamp.as_secs(),
            timestamp.subsec_micros(),
        ));
    });
    ($format_string:expr, $($arg:tt)*) => ({
        let timestamp = $crate::timer::time_manager().uptime();

        $crate::print::_print(format_args_nl!(
            concat!("[  {:>3}.{:06}] ", $format_string),
            timestamp.as_secs(),
            timestamp.subsec_micros(),
            $($arg)*
        ));
    })
}

/// Prints a warning, with a newline.
#[macro_export]
macro_rules! warn {
    ($string:expr) => ({
        let timestamp = $crate::timer::time_manager().uptime();

        $crate::print::_print(format_args_nl!(
            concat!("[W {:>3}.{:06}] ", $string),
            timestamp.as_secs(),
            timestamp.subsec_micros(),
        ));
    });
    ($format_string:expr, $($arg:tt)*) => ({
        let timestamp = $crate::timer::time_manager().uptime();

        $crate::print::_print(format_args_nl!(
            concat!("[W {:>3}.{:06}] ", $format_string),
            timestamp.as_secs(),
            timestamp.subsec_micros(),
            $($arg)*
        ));
    })
}
