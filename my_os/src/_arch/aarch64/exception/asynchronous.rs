//! Architectural asynchronous exception handling.

use aarch64_cpu::registers::*;
use tock_registers::interfaces::Readable;

/// Trait to retrieve a specific field of the `DAIF` register.
trait DaifField {
    /// Returns the corresponding field from the `DAIF` register.
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register>;
}

/// Structs representing different exception masks.
struct Debug; // Debug exceptions
struct SError; // SError exceptions (System Error)
struct IRQ; // Interrupt Request (IRQ) exceptions
struct FIQ; // Fast Interrupt Request (FIQ) exceptions

/// Implement `DaifField` for Debug exceptions.
impl DaifField for Debug {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::D // Debug mask bit in DAIF register
    }
}

/// Implement `DaifField` for SError exceptions.
impl DaifField for SError {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::A // Asynchronous abort mask bit in DAIF register
    }
}

/// Implement `DaifField` for IRQ exceptions.
impl DaifField for IRQ {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::I // IRQ mask bit in DAIF register
    }
}

/// Implement `DaifField` for FIQ exceptions.
impl DaifField for FIQ {
    fn daif_field() -> tock_registers::fields::Field<u64, DAIF::Register> {
        DAIF::F // FIQ mask bit in DAIF register
    }
}

/// Check if a specific exception type is masked (disabled).
fn is_masked<T>() -> bool
where
    T: DaifField,
{
    DAIF.is_set(T::daif_field()) // Check if the corresponding bit is set in DAIF
}

/// Print the current state of the exception masks.
#[rustfmt::skip]
pub fn print_state() {
    use crate::info;

    // Convert boolean mask state to a readable string.
    let to_mask_str = |x| -> _ {
        if x { "Masked" } else { "Unmasked" }
    };

    // Print the exception mask states
    info!("      Debug:  {}", to_mask_str(is_masked::<Debug>()));
    info!("      SError: {}", to_mask_str(is_masked::<SError>()));
    info!("      IRQ:    {}", to_mask_str(is_masked::<IRQ>()));
    info!("      FIQ:    {}", to_mask_str(is_masked::<FIQ>()));
}
