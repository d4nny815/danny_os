//! A panic handler that infinitely waits.

use crate::{cpu, println};
use core::panic::PanicInfo;

fn panic_prevent_reenter() {
    use core::sync::atomic::{AtomicBool, Ordering};

    static CURRENTLY_PANIC: AtomicBool = AtomicBool::new(false); // aarch64 specific

    // if not panicing set panic flag
    if !CURRENTLY_PANIC.load(Ordering::Relaxed) {
        CURRENTLY_PANIC.store(true, Ordering::Relaxed);

        return;
    }

    cpu::wait_forever() // spin if there is a core already panic
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Protect against panic infinite loops if any of the following code panics itself.
    panic_prevent_reenter();

    let timestamp = crate::timer::time_manager().uptime();
    let (location, line, column) = match info.location() {
        Some(loc) => (loc.file(), loc.line(), loc.column()),
        _ => ("???", 0, 0),
    };

    println!(
        "[  {:>3}.{:06}] Kernel panic!\n\n\
        Panic location:\n      File '{}', line {}, column {}\n\n\
        {}",
        timestamp.as_secs(),
        timestamp.subsec_micros(),
        location,
        line,
        column,
        info.message().unwrap_or(&format_args!("")),
    );

    cpu::wait_forever()
}
