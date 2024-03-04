library;

use ::contract_id::ContractId;

/// Load and run the contract with the provided `ContractId`
///
/// Contract code will be loaded using `LDC` and jumped into.
/// Unlike a normal contract call, the context of the contract running
/// `run_external` is retained for the loaded code.
///
/// As this function never returns to the original code that calls it, it returns `!`.
pub fn run_external(load_target: ContractId) -> ! {
    asm(load_target, word, length, ssp_saved) {
        lw load_target fp i74;
        csiz length load_target;
        move ssp_saved ssp;
        ldc load_target zero length;
        addi word zero i64;
        aloc word;
        sw hp ssp_saved i0;
    }
    __jmp_mem()
}
