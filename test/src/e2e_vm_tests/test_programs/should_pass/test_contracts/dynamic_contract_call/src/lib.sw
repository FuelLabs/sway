library;

#[cfg(experimental_new_encoding = false)]
pub fn dynamic_contract_call(contract_id: b256) -> u64 {
    // Call the fallback fn
    let call_params = (contract_id, 0, 0);
    let coins = 0;
    let asset_id = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let gas = std::registers::global_gas();
    asm(a: __addr_of(call_params), b: coins, c: __addr_of(asset_id), d: gas) {
        call a b c d;
    };
    let v = asm() {
        ret: u64
    };
    v
}

#[cfg(experimental_new_encoding = true)]
pub fn dynamic_contract_call(contract_id: b256) -> u64 {
    contract_call::<u64, (u64, u64, u64)>(contract_id,
        encode("some_method_name"),
        (1, 2, 3),
        0,
        0x0000000000000000000000000000000000000000000000000000000000000000,
        std::registers::global_gas()
    )
}
