//! The `kernel` binary.

//! # Boot flow
//! The kernel's entry point is the function `cpu::boot::arch_boot::_start()`.
//!     - It is implemented in `src/_arch/aarch64/cpu/boot.s`.

#![allow(clippy::upper_case_acronyms)]
#![feature(asm_const)]
#![feature(const_option)]
#![feature(format_args_nl)]
#![feature(nonzero_min_max)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![feature(unchecked_math)]
#![no_main]
#![no_std]

mod bsp;
mod console;
mod cpu;
mod drivers;
mod exception;
mod panic_wait;
mod print;
mod synchronization;
mod timer;

const CLEAR_SCREEN: &str = "\x1B[2J";
const RESET_CURSOR: &str = "\x1B[H";
const BOLD_TEXT: &str = "\x1B[1m";
const RESET_TEXT: &str = "\x1B[0m";
const BOOT_SCREEN: &str = r#"
    ____  __  _____________   ____  _____
   / __ \/ / / / ___/_  __/  / __ \/ ___/
  / /_/ / / / /\__ \ / /    / / / /\__ \ 
 / _, _/ /_/ /___/ // /    / /_/ /___/ / 
/_/ |_|\____//____//_/     \____//____/  
"#;

// boot.s calls this
unsafe fn kernel_init() -> ! {
    // init the driver subsystem.
    if let Err(x) = bsp::drivers::init() {
        panic!("Error initializing BSP driver subsystem: {}", x);
    }

    // init all the drivers
    drivers::driver_manager().init_drivers();

    kernel_main()
}

fn kernel_main() -> ! {
    use console::console;
    use core::time::Duration;

    print_boot_screen();

    for run in 1..=5 {
        info!("Run {} - Spinning for 1 second", run);
        timer::time_manager().spin_for(Duration::from_secs(1));
    }

    // echo mode.
    console().clear_rx();
    loop {
        let c = console().read_char();
        console().write_char(c);
    }
}

fn print_boot_screen() {
    use core::time::Duration;

    print!("{}", CLEAR_SCREEN);
    print!("{}", RESET_CURSOR);
    print!("{}", BOLD_TEXT);
    println!("{}", BOOT_SCREEN);

    info!("Booting on: {}", bsp::board_name());

    let (_, priv_level) = exception::current_privilege_level();
    info!("Current Privilge Level: {}", priv_level);

    info!("Exception handling state:");
    exception::asynchronous::print_state();

    info!(
        "Architectural timer resolution: {} ns",
        timer::time_manager().resolution().as_nanos()
    );

    println!("Drivers loaded:");
    drivers::driver_manager().enumerate();

    print!("{}", RESET_TEXT);

    // Test a failing timer case.
    timer::time_manager().spin_for(Duration::from_nanos(1));

    return;
}
