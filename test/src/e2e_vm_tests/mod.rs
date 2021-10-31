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
                139, 216, 67, 1, 172, 74, 189, 183, 82, 11, 99, 241, 23, 111, 195, 89, 208, 127,
                16, 95, 247, 254, 168, 151, 227, 225, 199, 179, 50, 80, 63, 175,
            ])),
        ),
        ("op_precedence", ProgramState::Return(0)),
        ("asm_without_return", ProgramState::Return(0)),
        ("op_precedence", ProgramState::Return(0)), // 1 == false
        ("b256_bad_jumps", ProgramState::Return(1)),
        ("b256_ops", ProgramState::Return(100)),
        ("struct_field_access", ProgramState::Return(43)),
        ("bool_and_or", ProgramState::Return(42)),
        ("doc_strings", ProgramState::Return(20)),
        ("neq_4_test", ProgramState::Return(0)),
        ("eq_4_test", ProgramState::Return(1)),
        ("local_impl_for_ord", ProgramState::Return(1)), // true
        ("const_decl", ProgramState::Return(100)),
        ("const_decl_in_library", ProgramState::Return(1)), // true
        ("aliased_imports", ProgramState::Return(42)),
        ("caller_auth_test", ProgramState::Return(1)), // true
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
        "infinite_dependencies",
        "top_level_vars",
        "dependencies_parsing_error",
        "mut_error_message",
        "reassignment_to_non_variable_message",
    ];
    project_names.into_iter().for_each(|name| {
        if filter(name) {
            crate::e2e_vm_tests::harness::does_not_compile(name)
        }
    });

    // ---- Contract Deployments
    // contracts that should be deployed for the following tests to work
    let contract_names = vec!["basic_storage", "increment_contract"];

    for name in contract_names {
        harness::deploy_contract(name)
    }

    // ---- Tests that need the above contracts deployed to work
    // TODO validate that call output is correct
    let project_names = &["call_basic_storage", "call_increment_contract"];

    project_names
        .into_iter()
        .for_each(|name| harness::runs_on_node(name));

    println!("_________________________________\nTests passed.");
}
