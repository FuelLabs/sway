// target-fuelvm

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

// check: local b256 $(contract_id_0_const=$ID) = const b256 0x0c1c50c2bf5ba4bb351b4249a2f5e7d86556fcb4a6ae90465ff6c86126eeb3c0
// check: local b256 $(asset_id_0_const=$ID) = const b256 0x0000000000000000000000000000000000000000000000000000000000000000
// check: local b256 $(threes_const=$ID) = const b256 0x3333333333333333333333333333333333333333333333333333333333333333
// check: local b256 $(contract_id_1_const=$ID) = const b256 0x0c1c50c2bf5ba4bb351b4249a2f5e7d86556fcb4a6ae90465ff6c86126eeb3c0
// check: local b256 $(asset_id_1_const=$ID) = const b256 0x0000000000000000000000000000000000000000000000000000000000000000
// check: local b256 $(big_fives_const=$ID) = const b256 0x5555555555555555555555555555555555555555555555555555555555555555
// check: local b256 $(contract_id_2_const=$ID) = const b256 0x0c1c50c2bf5ba4bb351b4249a2f5e7d86556fcb4a6ae90465ff6c86126eeb3c0
// check: local b256 $(asset_id_2_const=$ID) = const b256 0x0000000000000000000000000000000000000000000000000000000000000000
// check: local b256 $(arg_for_get_b256=$ID)

// check: $(contract_id_0_ptr=$VAL) = get_local __ptr b256, $contract_id_0_const
// check: $(threes_const_ptr=$VAL) = get_local __ptr b256, $threes_const
// check: $(contract_id_1_ptr=$VAL) = get_local __ptr b256, $contract_id_1_const
// check: $(big_fives_ptr=$VAL) = get_local __ptr b256, $big_fives_const
// check: $(contract_id_2_ptr=$VAL) = get_local __ptr b256, $contract_id_2_const

// --- call get_u64() ---
// check: $(oneone=$VAL) = const u64 1111
// check: $(user_arg=$VAL) = bitcast $oneone to u64

// check: $(args_ptr=$VAL) = get_local __ptr { b256, u64, u64 }, $ID

// check: $(idx_0=$VAL) = const u64 0
// check: $(arg_contract_id=$VAL) = get_elem_ptr $args_ptr, __ptr b256, $idx_0
// check: mem_copy_val $arg_contract_id, $contract_id_0_ptr

// check: $(idx_1=$VAL) = const u64 1
// check: $(arg_sel_ptr=$VAL) = get_elem_ptr $args_ptr, __ptr u64, $idx_1
// check: $(get_u64_sel=$VAL) = const u64 2559618804
// check: store $get_u64_sel to $arg_sel_ptr

// check: $(idx_2=$VAL) = const u64 2
// check: $(arg_user_ptr=$VAL) = get_elem_ptr $args_ptr, __ptr u64, $idx_2
// check: store $user_arg to $arg_user_ptr

// check: $(asset_id_ptr=$VAL) = get_local __ptr b256, $ID
// check: $(coins=$VAL) = const u64 0
// check: $(gas=$VAL) = const u64 10000
// check: $(call_res=$VAL) = contract_call u64 get_u64 $args_ptr, $coins, $asset_id_ptr, $gas

// --- call get_b256() ---
// check: $(user_arg_ptr=$VAL) = get_local __ptr b256, $arg_for_get_b256
// check: mem_copy_val $user_arg_ptr, $threes_const_ptr
// check: $(user_arg=$VAL) = ptr_to_int $user_arg_ptr to u64

// check: $(args_ptr=$VAL) = get_local __ptr { b256, u64, u64 }, $ID

// check: $(idx_0=$VAL) = const u64 0
// check: $(arg_contract_id=$VAL) = get_elem_ptr $args_ptr, __ptr b256, $idx_0
// check: mem_copy_val $arg_contract_id, $contract_id_1_ptr

// check: $(idx_1=$VAL) = const u64 1
// check: $(arg_sel_ptr=$VAL) = get_elem_ptr $args_ptr, __ptr u64, $idx_1
// check: $(get_b256_sel=$VAL) = const u64 1108491158
// check: store $get_b256_sel to $arg_sel_ptr

// check: $(idx_2=$VAL) = const u64 2
// check: $(args_user_ptr=$VAL) = get_elem_ptr $args_ptr, __ptr u64, $idx_2
// check: store $user_arg to $args_user_ptr

// check: $(asset_id_ptr=$VAL) = get_local __ptr b256, $asset_id_1_const
// check: $(coins=$VAL) = const u64 0
// check: $(gas=$VAL) = const u64 20000
// check: $(call_res=$VAL) = contract_call __ptr b256 get_b256 $args_ptr, $coins, $asset_id_ptr, $gas

// --- call get_s() --
// check: $(user_arg_ptr=$VAL) = get_local __ptr { u64, b256 }, args_struct_for_get_s

// check: $(idx_0=$VAL) = const u64 0
// check: $(user_arg_field0=$VAL) = get_elem_ptr $user_arg_ptr, __ptr u64, $idx_0
// check: $(small_fives=$VAL) = const u64 5555
// check: store $small_fives to $user_arg_field0

// check: $(idx_1=$VAL) = const u64 1
// check: $(user_arg_field1=$VAL) = get_elem_ptr $user_arg_ptr, __ptr b256, $idx_1
// check: mem_copy_val $user_arg_field1, $big_fives_ptr

// check: $(user_arg=$VAL) = ptr_to_int $user_arg_ptr to u64

// check: $(args_ptr=$VAL) = get_local __ptr { b256, u64, u64 }, $ID

// check: $(idx_0=$VAL) = const u64 0
// check: $(arg_contract_id=$VAL) = get_elem_ptr $args_ptr, __ptr b256, $idx_0
// check: mem_copy_val $arg_contract_id, $contract_id_2_ptr

// check: $(idx_1=$VAL) = const u64 1
// check: $(arg_sel_ptr=$VAL) = get_elem_ptr $args_ptr, __ptr u64, $idx_1
// check: $(get_s_sel=$VAL) = const u64 4234334249
// check: store $get_s_sel to $arg_sel_ptr

// check: $(idx_2=$VAL) = const u64 2
// check: $(args_user_ptr=$VAL) = get_elem_ptr $args_ptr, __ptr u64, $idx_2
// check: store $user_arg to $args_user_ptr

// check: $(asset_id_ptr=$VAL) = get_local __ptr b256, $asset_id_2_const
// check: $(gas=$VAL) = read_register cgas
// check: $(coins=$VAL) = const u64 0
// check: $(call_res=$VAL) = contract_call __ptr { u64, b256 } get_s $args_ptr, $coins, $asset_id_ptr, $gas
