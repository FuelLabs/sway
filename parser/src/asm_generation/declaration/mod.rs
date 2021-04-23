use crate::{vendored_vm::Op, TypedDeclaration};

use super::{AsmNamespace, RegisterSequencer};
mod fn_decl;
mod var_decl;
pub(crate) use fn_decl::convert_fn_decl_to_asm;
pub(crate) use var_decl::convert_variable_decl_to_asm;

pub(crate) fn convert_decl_to_asm<'sc>(
    decl: &TypedDeclaration<'sc>,
    namespace: &mut AsmNamespace<'sc>,
    register_sequencer: &mut RegisterSequencer,
) -> Vec<Op<'sc>> {
    match decl {
        // For an enum declaration, we don't generate any asm.
        TypedDeclaration::EnumDeclaration(_) => vec![],
        TypedDeclaration::FunctionDeclaration(typed_fn_decl) => {
            convert_fn_decl_to_asm(typed_fn_decl, namespace, register_sequencer)
        }
        // a trait declaration also does not have any asm directly generated from it
        TypedDeclaration::TraitDeclaration(_) => vec![],
        // since all functions are inlined (for now -- shortcut), we also don't need to do anything for this.
        TypedDeclaration::ImplTrait { .. } => vec![],
        // once again the declaration of a type has no inherent asm, only instantiations
        TypedDeclaration::StructDeclaration(_) => vec![],
        TypedDeclaration::VariableDeclaration(var_decl) => {
            convert_variable_decl_to_asm(var_decl, namespace, register_sequencer)
        }
        a => todo!("{:?}", a),
    }
}
