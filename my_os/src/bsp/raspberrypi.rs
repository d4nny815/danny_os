//! BSP file for the Raspberry Pi 3

// export board specific implementations
pub mod cpu;
pub mod drivers;
pub mod memory;

pub fn board_name() -> &'static str {
    "Raspberry Pi 3"
}
