mod harness;
use fuel_vm::prelude::*;

pub fn run(filter_regex: Option<regex::Regex>) {
    let filter = |name| {
        filter_regex
            .as_ref()
            .map(|regex| regex.is_match(name))
            .unwrap_or(true)
    };

    // programs that should successfully compile and terminate
    // with some known state
    let project_names = vec![
        ("asm_expr_basic", ProgramState::Return(6)),
        ("basic_func_decl", ProgramState::Return(1)), // 1 == true
        ("contract_abi_impl", ProgramState::Return(0)),
        ("dependencies", ProgramState::Return(0)), // 0 == false
        ("if_elseif_enum", ProgramState::Return(10)),
        ("out_of_order_decl", ProgramState::Return(1)),
        ("struct_field_reassignment", ProgramState::Return(0)),
        ("contract_call", ProgramState::Return(0)),
        ("enum_in_fn_decl", ProgramState::Return(255)),
        ("empty_impl", ProgramState::Return(0)),
        ("main_returns_unit", ProgramState::Return(0)),
        ("unary_not_basic", ProgramState::Return(1)), // 1 == true
        ("unary_not_basic_2", ProgramState::Return(1)), // 1 == true
        ("fix_opcode_bug", ProgramState::Return(30)),
        (
            "retd_b256",
            ProgramState::ReturnData(Bytes32::from([
                102, 104, 122, 173, 248, 98, 189, 119, 108, 143, 193, 139, 142, 159, 142, 32, 8,
                151, 20, 133, 110, 226, 51, 179, 144, 42, 89, 29, 13, 95, 41, 37,
            ])),
        ),
        (
            "retd_struct",
            ProgramState::ReturnData(Bytes32::from([
                2, 23, 32, 21, 62, 98, 71, 190, 175, 43, 135, 133, 106, 105, 116, 64, 126, 40, 204,
                235, 151, 159, 245, 170, 112, 203, 40, 158, 9, 238, 188, 213,
            ])),
        ),
        ("op_precedence", ProgramState::Return(0)),
        ("asm_without_return", ProgramState::Return(0)),
        ("op_precedence", ProgramState::Return(0)), // 1 == false
        ("b256_bad_jumps", ProgramState::Return(1)),
        ("b256_ops", ProgramState::Return(100)),
        ("bool_and_or", ProgramState::Return(42)),
        ("neq_4_test", ProgramState::Return(0)),
        ("eq_4_test", ProgramState::Return(1)),
    ];
    project_names.into_iter().for_each(|(name, res)| {
        if filter(name) {
            assert_eq!(crate::e2e_vm_tests::harness::runs_in_vm(name), res);
        }
    });

    // source code that should _not_ compile
    let project_names = vec![
        "recursive_calls",
        "asm_missing_return",
        "asm_should_not_have_return",
        "missing_fn_arguments",
        "excess_fn_arguments",
    ];
    project_names.into_iter().for_each(|name| {
        if filter(name) {
            crate::e2e_vm_tests::harness::does_not_compile(name)
        }
    });

    println!("_________________________________\nTests passed.");
}
