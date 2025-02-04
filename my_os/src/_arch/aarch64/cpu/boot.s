#! Actual Boot code

// 	Load the address of a symbol into a register, PC-relative.
// 		The symbol must lie within +/- 4 GiB of the Program Counter.
// 	- https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
// same as #define ADR_REL(reg, sym) (do_work)
.macro ADR_REL register, symbol
	adrp	\register, \symbol					// load high part of the address
	add	\register, \register, #:lo12:\symbol	// load low part into reg 
.endm

// define entry
.section .text._start

// entry point
_start:
	// continue if in HyperVisor priv level
	mrs 	x0, CurrentEL						// load priv level into x0 
	cmp		x0, {CONST_EL2_MASK}				
	b.ne	.L_parking_loop						// if not priv level 2 then spin


	// stop all the other cores except 0	
	// MPIDR_EL1 has info about the current CPU core(core ID, cluster ID)
	mrs		x0, MPIDR_EL1						// load core info from sys reg into x0
	and		x0, x0, {CONST_CORE_ID_MASK}		// mask for core id 
	ldr		x1, BOOT_CORE_ID      				// load core_id in x1 for comparison
	cmp		x0, x1								
	b.ne	.L_parking_loop						// if not core 0 park

	// Initialize DRAM start and end
	ADR_REL	x0, __bss_start					
	ADR_REL x1, __bss_end_exclusive
	
.L_bss_init_loop:
	cmp		x0, x1
	b.eq	.L_prepare_rust						// continue if done
	stp		xzr, xzr, [x0], #16					// zero out 16 bytes at a time
	b		.L_bss_init_loop						// keep zeroing out bss

.L_prepare_rust:
	// Set the stack pointer.
	ADR_REL	x0, __boot_core_stack_end_exclusive	// x0 has top addr
	mov	sp, x0									// copy x0 to sp reg

	// Read cpu's timer info
	ADR_REL x1, ARCH_TIMER_COUNTER_FREQUENCY 	// load addr for timer info (inside kernel) into x1
	mrs 	x2, CNTFRQ_EL0						// load the timer info into x2
	cmp		x2, xzr								
	b.eq	.L_parking_loop						// if CNT FREQ is 0 sumting wong
	str		w2, [x1]							// save the info in the var

	b	_start_rust								// start the rust code
	
.L_parking_loop:
	wfe											// sleepy time
	b	.L_parking_loop

.size	_start, . - _start
.type	_start, function
.global	_start
