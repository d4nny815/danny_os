//! System Console

mod null_console;
use crate::synchronization::{self, SpinLock};
// bsp defines the implemention
pub mod interface {
    use core::fmt;

    /// Console write functions.
    #[allow(dead_code)]
    pub trait Write {
        fn write_char(&self, c: char);

        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;

        fn flush(&self);
    }

    /// Console read functions.
    pub trait Read {
        fn read_char(&self) -> char {
            ' '
        }

        fn clear_rx(&self);
    }

    /// Console statistics.
    #[allow(dead_code)]
    pub trait Stats {
        fn chars_written(&self) -> usize {
            0
        }

        fn chars_read(&self) -> usize {
            0
        }
    }

    pub trait All: Write + Read + Stats {}
}

static CUR_CONSOLE: SpinLock<&'static (dyn interface::All + Sync)> =
    SpinLock::new(&null_console::NULL_CONSOLE);

use synchronization::interface::Mutex;

/// Register a new console.
pub fn register_console(new_console: &'static (dyn interface::All + Sync)) {
    CUR_CONSOLE.lock(|con| *con = new_console);
}

/// Return a reference to the currently registered console.
pub fn console() -> &'static dyn interface::All {
    CUR_CONSOLE.lock(|con| *con)
}
