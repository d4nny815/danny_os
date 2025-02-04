//! Synchronous and asynchronous exception handling.

#[path = "_arch/aarch64/exception.rs"]
mod arch_exception;

pub mod asynchronous;

pub use arch_exception::current_privilege_level;

/// Kernel privilege levels.
#[allow(missing_docs)]
#[derive(Eq, PartialEq)]
pub enum PrivilegeLevel {
    User,
    Kernel,
    Hypervisor,
    Unknown,
}
