//! Helper functions to load and run external contract code.
library;

use ::contract_id::ContractId;

/// Load and run the contract with the provided `ContractId`.
///
/// Contract code will be loaded using `LDC` and jumped into.
/// Unlike a normal contract call, the context of the contract running
/// `run_external` is retained for the loaded code.
///
/// As this function never returns to the original code that calls it, it returns `!`.
#[inline(never)]
pub fn run_external(load_target: ContractId) -> ! {
    asm(
        load_target: load_target,
        word,
        length,
        ssp_saved,
        cur_stack_size,
    ) {
        csiz length load_target;
        move ssp_saved ssp;
        sub cur_stack_size sp ssp;
        cfs cur_stack_size;
        ldc load_target zero length i0;
        addi word zero i64;
        aloc word;
        sw hp ssp_saved i0;
    }
    __jmp_mem()
}

pub fn run_external2(load_target1: ContractId, load_target2: ContractId) -> ! {
    asm(
        load_target1: load_target1,
        load_target2: load_target2,
        load_target2_heap,
        heap_alloc_size,
        length1,
        length2,
        ssp_saved,
        cur_stack_size,
    ) {
        // Get lengths of both chunks
        csiz length1 load_target1;
        csiz length2 load_target2;

        // Store load_target2 on the heap as it'll be overwritten with the first LDC we do.
        addi heap_alloc_size zero i32;
        aloc heap_alloc_size;
        mcp hp load_target2 heap_alloc_size;
        move load_target2_heap hp;

        // Save the old $ssp value as that's were the contract will be loaded.
        move ssp_saved ssp;
        // Shrink the stack since LDC wants $ssp == $sp
        sub cur_stack_size sp ssp;
        cfs cur_stack_size;

        // Do the loads
        ldc load_target1 zero length1 i0;
        ldc load_target2_heap zero length2 i0;

        // __jmp_mem jumps to $MEM[$hp], so set that up.
        addi heap_alloc_size zero i64;
        aloc heap_alloc_size;
        sw hp ssp_saved i0;
    }
    __jmp_mem()
}