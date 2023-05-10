//! Utilities to help with loading code from a deployed contract into the current contract.
//! This allows for patterns like proxies and upgrades, executing the loaded code in the context 
//! of the contract which does the loading. Additionally, once loaded the new code is 
//! cheap to call as it can be accessed via jumps instead of contract calls.
library;


/// Load 'size' bytes of bytecode from contract at 'source' starting at 'offset'.
///
/// ### Arguments
///
/// * source - The contract from which to load code.
/// * offset - The number of bytes from the start of the bytecode at `source` from which to start loading code.
/// * size   - The total number of bytes to load, beginning at `offset`.
///
/// Ref: http://specs.fuel.network/master/vm/instruction_set.html?highlight=ldc#ldc-load-code-from-an-external-contract
fn load_code(source: ContractId, offset: u64, size: u64) {
    asm(src: source, start: offset, bytes: size) {
        ldc src start size;
    };
}

