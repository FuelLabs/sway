mod harness;
use fuel_vm::prelude::*;

pub fn run() {
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
        ("unary_not_basic_2", ProgramState::Return(0)), // 0 == false
    ];
    project_names.into_iter().for_each(|(name, res)| {
        assert_eq!(crate::e2e_vm_tests::harness::runs_in_vm(name), res);
    });
}
