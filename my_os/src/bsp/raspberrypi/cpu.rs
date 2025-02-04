//! Board code needed by the arch

/// Used by `arch` code to find the boot core.
#[no_mangle]
#[link_section = ".text._start_arguments"]
pub static BOOT_CORE_ID: u64 = 0; // change this if i want a main core to run on
