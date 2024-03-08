library;

pub fn dynamic_contract_call(contract_id: b256) -> u64 {
    // Call the fallback fn
    let call_params = (contract_id, 0, 0);
    let coins = 0;
    let asset_id = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let gas = std::registers::global_gas();
    let result = asm(a: __addr_of(call_params), b: coins, c: __addr_of(asset_id), d: gas) {
        call a b c d;
        ret: u64
    };
    result
}
