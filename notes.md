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
- Generic Kernel Code
    - descriptor types for different parts of memory
        - R/W, no-exec, un/cached
    - indepentant of the actual MMU's descriptors
- BSP Code
    - Contains `KernelVirtualLayout`, stores the high-level descriptors
    - This is here since the BSP has knows the board's memory map
    - Only describe regions that are **not** ordinary, normal cacheable DRAM
    - `KernelVirtualLayout` implements a property's function
        - used by the arch code to get properties for translation, and returns the phyiscal addr
        - The function scans for a descriptor that contains the queried address, and returns the respective findings for the first entry that is a hit. 
        - If no entry is found, it returns default attributes for normal cacheable DRAM and the input address, hence telling the MMU code that the requested address should be identity mapped
- Arch Specific Code
    - contains the aarch64 MMU driver
    - defines the page granule size (64 KB)
        - 16b offset for addressing inside a page.
        - 13b for indexing into L3 PT
        - 13b for indexing into L2 PT
        -  only a small amount of bits will actually be used
        - 9b for indexing in L1 PT
            - wont be used
        - 48-bit virtual address space for indexing.
    - Page Table (PT)
        - 8B (64-bit) per entry
        - each PT contains 8K entries 
    - LVL3 Page Table (L3 PT)
        - each entry maps 64 KB
        - Full PT covers 512MB
            - 8K * 64KB
        - L3 PT points directly to a page in physical mem
    - LVL2 Page Table (L2 PT)
        - each entry points to an L3 PT.
        - Full PT covers 4TB
            - 8K * 512MB

- Static Translation Table


