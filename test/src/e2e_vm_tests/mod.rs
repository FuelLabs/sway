mod harness;

pub fn run() {
    let project_names = vec![
        "asm_expr_basic",
        "basic_func_decl",
        "contract_abi_impl",
        "dependencies",
        "if_elseif_enum",
        "out_of_order_decl",
        "struct_field_reassignment",
        "contract_call",
        "enum_in_fn_decl",
        "empty_impl",
    ];
    project_names.into_iter().for_each(|name| {
        crate::e2e_vm_tests::harness::runs_in_vm(name);
    });
}
