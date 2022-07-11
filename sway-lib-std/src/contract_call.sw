library contract_call;

use ::contract_id::ContractId;

// Wrapper around the low-level CALL opcode specified here (https://github.com/FuelLabs/fuel-specs/blob/1be31f70c757d8390f74b9e1b3beb096620553eb/specs/vm/instruction_set.md#call-call-contract)

pub struct CallData {
    /// Data to pass into the called function
    arguments: u64,
    /// Encoded representation of a function to be called on the specified contract
    function_selector: u64,
    /// The Id of the contract to be caled using the provided function selector and arguments
    id: ContractId,
}

pub fn call(call_data: CallData, amount: u64, asset: ContractId, gas: u64) {
    asm(call_data, amount, asset, gas) {
        call call_data amount asset gas;
    }
}