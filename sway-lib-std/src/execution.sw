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
