# Notes 
## Arm privilege levels: 
- EL stands for exception level
- User space, Kernel Level, Hypervisor, Low-Level Firmware -> (EL0, EL1, EL2, EL3)
- The Exception level can only change when any of the following occur:
- Taking an exception
    - Returning from an exception
    - Processor reset
    - During Debug state
    - Exiting from Debug state
- When taking one, level can only increase
- When returning from one, level can only decrease
- A-prof Arm arch implements virtual memory
    - uses MMU to apply attributes to memory regions
    - Like read/write perms for mem
    - when in EL0 it checks unpriv perms
    - when in EL1-3 it check priv perms
    - MMU config in sys regs
- Sys regs have the perms needed at the end with _ELx 
- SCTLR_EL1
    - Top-level system control for EL0 and EL1
- SCTLR_EL2
    - Top-level system control for EL2
- SCTLR_EL3
    - Top-level system control for EL3
- Pi3 boots into EL2 (Kernel Level)

## ARM virtualization
- A separate, isolated computing environment, which is indistinguishable from the real physical machine
- Type 2 hypervisor - hosted
    - Host OS has full control of the hardware
    - hosts guest OSs
    - Running QEMU, Virtual Box on pc to host another OS
- Type 1 hypervisor - standalone
    - No host OS
    - runs directly on the hw
- Not efficient to emualate a guest OS on arm
- Better to let to the OS know its being emulated, has virtual devices
    - paravirtualization
- Stage 2 translation
    - allows hypervisor control the view of mem that an VM has
    - hypervisor controls the MMIO and wheres its placed the the VM's mem space
- VA -> IPA -> PA
    - OS does the translation from virtual address(VA) to Intermediate Physical Address(IPA), stage 1 translation
    - OS thinks IPA is what the phyiscal mem looks like
    - Hypervisor translates IPA to Physical Address(PA) where the resources are actually located, stage 2 translation
- Virtual Machine ID (VMID)
    - used as the tag in the translation lookaside buffer (TLB) 
    - allows multiple VMs translation to be into the TLB together
    - EL2 and 3 are not subject to stage 2 translations
- Address Space ID (ASID)
    - app is assigned ASID by the OS
- Stage 1 and 2 translation have info about perms and type


# Virtual Memory
- Static Translation Table


