use aarch64_cpu::asm; // aarch64_cpu crate for asm

pub use asm::nop;

#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe();
    }
}
