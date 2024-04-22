// target-fuelvm

contract;

pub struct ContractId {
    /// The underlying raw `b256` data of the contract id.
    pub value: b256,
}

abi MyContract {
    fn test_function(code_id: ContractId);
}

impl MyContract for Contract {
    fn test_function(code_id: ContractId) {
        asm(code_id, word, length, ssp_saved) {
            lw code_id fp i74;
            // Load the entire contract with LDC
            csiz length code_id;
            // Save the old ssp
            move ssp_saved ssp;
            ldc code_id zero length;
            // Store the old ssp to MEM[$hp] so that we can jump to it.
            // allocate a word the stack
            addi word zero i64;
            aloc word;
            sw hp ssp_saved i0;
        }
        __jmp_mem()
    }
}

// ::check-ir::

// check: pub entry fn test_function<72a09f5b>

// ::check-ir-optimized::
// pass: o1

// check: pub entry fn test_function
// not: local
// check: csiz   length code_id
// check: ldc    code_id zero length,
// check: jmp_mem

// ::check-asm::

// regex: REG=.r\d+\b

// check: csiz $(len=$REG) $REG
// check: ldc  $REG $$zero $len
// check: lw   $(target=$REG) $$hp i0               ; jmp_mem: Load MEM[$$hp]
// check: sub  $(jmp_target_4=$REG) $target $$is              ; jmp_mem: Subtract $$is since Jmp adds it back
// check: divi $(jmp_target=$REG) $jmp_target_4 i4               ; jmp_mem: Divide by 4 since Jmp multiplies by 4
// check: jmp $jmp_target                       ; jmp_mem: Jump to computed value
