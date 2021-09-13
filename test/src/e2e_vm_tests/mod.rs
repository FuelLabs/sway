mod harness;
use fuel_vm::prelude::*;

pub fn run() {
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
    ];
    project_names.into_iter().for_each(|(name, res)| {
        assert_eq!(crate::e2e_vm_tests::harness::runs_in_vm(name), res);
    });

    // source code that should _not_ compile
    let project_names = vec!["recursive_calls"];
    project_names
        .into_iter()
        .for_each(|name| does_not_compile(name));
}
