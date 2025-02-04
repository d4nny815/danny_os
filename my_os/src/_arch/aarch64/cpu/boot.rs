//! Architectural boot code.

use aarch64_cpu::{asm, registers::*};
use core::arch::global_asm;
use tock_registers::interfaces::Writeable;

global_asm!(
    include_str!("boot.s"),
    CONST_CORE_ID_MASK = const 0b11,     // mask for just getting CORE_ID, used in boot.s
    CONST_EL2_MASK = const 0x8
);

#[inline(always)]
unsafe fn prep_el2_to_el1_trans(phys_boot_core_stack_end_exclusive_addr: u64) {
    // allow timers for EL1
    CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

    // no offsets for reading cntrs
    CNTVOFF_EL2.set(0);

    // set EL1 to op in Aarch64 mode
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);

    SPSR_EL2.write(
        SPSR_EL2::D::Unmasked
            + SPSR_EL2::A::Unmasked
            + SPSR_EL2::I::Unmasked
            + SPSR_EL2::F::Unmasked
            + SPSR_EL2::M::EL1h,
    );

    // load kernel init addr here
    // when going down levels it goes to this addr
    ELR_EL2.set(crate::kernel_init as *const () as u64);

    // setup el1 stack
    SP_EL1.set(phys_boot_core_stack_end_exclusive_addr);
}

// The Rust entry of the `kernel` binary.
// function is called from the assembly `_start` function.
// x0 contains the top of stack addr from the asm code
// x0 is the 1st arg reg in ARM
#[no_mangle]
pub unsafe fn _start_rust(phys_boot_core_stack_end_exclusive_addr: u64) -> ! {
    prep_el2_to_el1_trans(phys_boot_core_stack_end_exclusive_addr);

    // Use `eret` to "return" to EL1. This results in execution of kernel_init() in EL1.
    asm::eret();
}
