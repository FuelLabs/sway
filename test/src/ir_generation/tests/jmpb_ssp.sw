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
    fn test_function(code_id_p: ContractId) {
        let length = asm(code_id, length, word, ssp_saved) {
            // Allocate 32 bytes on the heap (we can't use the stack)
            addi word zero i32;
            aloc word;

            lw code_id fp i74;
           
            // Log the ContractID for debugging
            logd zero zero code_id word;

            // Load the entire contract with LDC
            csiz length code_id;
            // Save the old ssp
            move ssp_saved ssp;
            ldc code_id zero length;
            // return the ssp difference, to feed __jmpb_ssp.
            // This need not always be equal to `length` as `ldc` pads the `length`.
            sub length ssp ssp_saved;
            length: u64
        };
        __jmpb_ssp(length)
    }
}

// ::check-ir::

// check: pub entry fn test_function<72a09f5b>

// ::check-ir-optimized::
// pass: o1

// check: pub entry fn test_function
// not: local
// check: csiz   length code_id, !7
// check: ldc    code_id zero length,
// check: jmpb_ssp

// ::check-asm::

// regex: REG=.r\d+\b

// check: csiz $(len=$REG) $REG
// check: ldc  $REG $$zero $len
// check: sub  $(old_ssp=$REG) $$ssp $REG             ; jmpb_ssp: Compute $$ssp - offset
// sub  $(jmp_target_4=$REG) $old_ssp $$is              ; jmpb_ssp: Subtract $$is since $$jmp adds it back
// divi $(jmp_target=$REG) $jmp_target_4 i4               ; jmpb_ssp: Divide by 4 since Jmp multiplies by 4
// jmp $jmp_target                       ; jmpb_ssp: Jump to computed value
