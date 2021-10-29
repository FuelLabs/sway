use core_lang::semantic_analysis::ast_node::TypedDeclaration;

pub fn generate_abi_spec<'sc>(declaration: TypedDeclaration<'sc>) {
    match declaration {
        TypedDeclaration::AbiDeclaration(typed_abi_declaration) => {
            println!("{:?}", typed_abi_declaration);
        }
        _ => {
            // todo
        }
    }
}
