script;

struct S {
    x: u64,
    y: b256,
}

abi Test {
    fn get_u64(val: u64) -> u64;
    fn get_b256(val: b256) -> b256;
    fn get_s(val1: u64, val2: b256) -> S;
}

fn main() -> u64 {
    let caller = abi(Test, 0x0c1c50c2bf5ba4bb351b4249a2f5e7d86556fcb4a6ae90465ff6c86126eeb3c0);

    let a = caller.get_u64 {
        coins: 0,
        asset_id: 0x0000000000000000000000000000000000000000000000000000000000000000,
        gas: 10000,
    }
    (1111);

    let b = caller.get_b256 {
        coins: 0,
        asset_id: 0x0000000000000000000000000000000000000000000000000000000000000000,
        gas: 20000,
    }
    (0x3333333333333333333333333333333333333333333333333333333333333333);

    let s = caller.get_s {
        coins: 0,
        asset_id:0x0000000000000000000000000000000000000000000000000000000000000000,
    }
    (5555, 0x5555555555555555555555555555555555555555555555555555555555555555);
    0
}

// check: local ptr u64 a
// check: local ptr b256 arg_for_get_b256
// check: local mut ptr { u64, b256 } args_struct_for_get_s
// check: local ptr b256 b
// check: local ptr { u64, b256 } s

// BUILD THE PARAMS: contract id, selector and immediate argument.
// check: $(get_u64_arg=$VAL) = const u64 1111
// check: $(get_u64_arg_bitcast=$VAL) = bitcast $get_u64_arg to u64
// check: $(get_u64_params_undef=$VAL) = const { b256, u64, u64 } { b256 undef, u64 undef, u64 undef }
// check: $(contract_id=$VAL) = const b256 0x0c1c50c2bf5ba4bb351b4249a2f5e7d86556fcb4a6ae90465ff6c86126eeb3c0
// check: $(get_u64_params_0=$VAL) = insert_value $get_u64_params_undef, { b256, u64, u64 }, $contract_id, 0
// check: $(get_u64_selector=$VAL) = const u64 2559618804
// check: $(get_u64_params_1=$VAL) = insert_value $get_u64_params_0, { b256, u64, u64 }, $get_u64_selector, 1
// check: $(get_u64_params=$VAL) = insert_value $get_u64_params_1, { b256, u64, u64 }, $get_u64_arg_bitcast, 2

// MAKE THE CONTRACT CALL: params, coins, asset and gas.
// check: $(get_u64_coins=$VAL) = const u64 0
// check: $(get_u64_asset=$VAL) = const b256 0x0000000000000000000000000000000000000000000000000000000000000000
// check: $(get_u64_gas=$VAL) = const u64 10000
// check: $(get_u64_res=$VAL) = contract_call u64 get_u64 $get_u64_params, $get_u64_coins, $get_u64_asset, $get_u64_gas

// check: $(a_ptr=$VAL) = get_ptr ptr u64 a, ptr u64, 0
// check: store $get_u64_res, ptr $a_ptr

// BUILD THE PARAMS: contract id, selector and ptr to argument.
// check: $(get_b256_arg=$VAL) = get_ptr ptr b256 arg_for_get_b256, ptr b256, 0
// check: $(get_b256_arg_lit=$VAL) = const b256 0x3333333333333333333333333333333333333333333333333333333333333333
// check: store $get_b256_arg_lit, ptr $get_b256_arg
// check: $(get_b256_arg_ptr=$VAL) = get_ptr ptr b256 arg_for_get_b256, ptr u64, 0
// check: $(get_b256_params_undef=$VAL) = const { b256, u64, u64 } { b256 undef, u64 undef, u64 undef }
// check: $(contract_id=$VAL) = const b256 0x0c1c50c2bf5ba4bb351b4249a2f5e7d86556fcb4a6ae90465ff6c86126eeb3c0
// check: $(get_b256_params_0=$VAL) = insert_value $get_b256_params_undef, { b256, u64, u64 }, $contract_id, 0
// check: $(get_b256_selector=$VAL) = const u64 1108491158
// check: $(get_b256_params_1=$VAL) = insert_value $get_b256_params_0, { b256, u64, u64 }, $get_b256_selector, 1
// check: $(get_b256_params=$VAL) = insert_value $get_b256_params_1, { b256, u64, u64 }, $get_b256_arg_ptr, 2

// MAKE THE CONTRACT CALL: params, coins, asset and gas.
// check: $(get_b256_coins=$VAL) = const u64 0
// check: $(get_b256_asset=$VAL) = const b256 0x0000000000000000000000000000000000000000000000000000000000000000
// check: $(get_b256_gas=$VAL) = const u64 20000
// check: $(get_b256_res=$VAL) = contract_call b256 get_b256 $get_b256_params, $get_b256_coins, $get_b256_asset, $get_b256_gas

// check: $(b_ptr=$VAL) = get_ptr ptr b256 b, ptr b256, 0
// check: store $get_b256_res, ptr $b_ptr


// BUILD THE PARAMS: contract id, selector and ptr to struct argument.
// check: $(get_s_arg_undef=$VAL) = get_ptr mut ptr { u64, b256 } args_struct_for_get_s, ptr { u64, b256 }, 0
// check: $(get_s_arg_x=$VAL) = const u64 5555
// check: $(get_s_arg_0=$VAL) = insert_value $get_s_arg_undef, { u64, b256 }, $get_s_arg_x, 0
// check: $(get_s_arg_y=$VAL) = const b256 0x5555555555555555555555555555555555555555555555555555555555555555
// check: $VAL = insert_value $get_s_arg_0, { u64, b256 }, $get_s_arg_y, 1
// check: $(get_s_arg_ptr=$VAL) = get_ptr mut ptr { u64, b256 } args_struct_for_get_s, ptr u64, 0
// check: $(get_s_params_undef=$VAL) = const { b256, u64, u64 } { b256 undef, u64 undef, u64 undef }
// check: $(contract_id=$VAL) = const b256 0x0c1c50c2bf5ba4bb351b4249a2f5e7d86556fcb4a6ae90465ff6c86126eeb3c0
// check: $(get_s_params_0=$VAL) = insert_value $get_s_params_undef, { b256, u64, u64 }, $contract_id, 0
// check: $(get_s_selector=$VAL) = const u64 4234334249
// check: $(get_s_params_1=$VAL) = insert_value $get_s_params_0, { b256, u64, u64 }, $get_s_selector, 1
// check: $(get_s_params=$VAL) = insert_value $get_s_params_1, { b256, u64, u64 }, $get_s_arg_ptr, 2

// MAKE THE CONTRACT CALL: params, coins, asset and gas.
// check: $(get_s_gas=$VAL) = read_register cgas
// check: $(get_s_coins=$VAL) = const u64 0
// check: $(get_s_asset=$VAL) = const b256 0x0000000000000000000000000000000000000000000000000000000000000000
// check: $(get_s_res=$VAL) = contract_call { u64, b256 } get_s $get_s_params, $get_s_coins, $get_s_asset, $get_s_gas

// check: $(s_ptr=$VAL) = get_ptr ptr { u64, b256 } s, ptr { u64, b256 }, 0
// check: store $get_s_res, ptr $s_ptr
