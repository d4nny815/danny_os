//! Processor code.

// export the arch code
#[path = "_arch/aarch64/cpu.rs"]
mod arch_cpu;

// export boot code
mod boot;

// export the arch spin techinque
pub use arch_cpu::{nop, wait_forever};
