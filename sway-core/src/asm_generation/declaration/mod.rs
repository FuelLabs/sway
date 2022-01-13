use super::{AsmNamespace, RegisterSequencer};
use crate::{asm_lang::Op, error::*, TypedDeclaration};

mod const_decl;
mod fn_decl;
mod reassignment;
mod var_decl;

pub(crate) use const_decl::convert_constant_decl_to_asm;
pub(crate) use fn_decl::convert_fn_decl_to_asm;
pub(crate) use reassignment::convert_reassignment_to_asm;
pub(crate) use var_decl::convert_variable_decl_to_asm;

pub(crate) fn convert_decl_to_asm(
    decl: &TypedDeclaration,
    namespace: &mut AsmNamespace,
    register_sequencer: &mut RegisterSequencer,
) -> CompileResult<Vec<Op>> {
    match decl {
        // For an enum declaration, we don't generate any asm.
        TypedDeclaration::EnumDeclaration(_) => ok(vec![], vec![], vec![]),
        TypedDeclaration::FunctionDeclaration(typed_fn_decl) => {
            convert_fn_decl_to_asm(typed_fn_decl, namespace, register_sequencer)
        }
        // a trait declaration also does not have any asm directly generated from it
        TypedDeclaration::TraitDeclaration(_) => ok(vec![], vec![], vec![]),
        // since all functions are inlined (for now -- shortcut), we also don't need to do anything
        // for this.
        TypedDeclaration::ImplTrait { .. } => ok(vec![], vec![], vec![]),
        // once again the declaration of a type has no inherent asm, only instantiations
        TypedDeclaration::StructDeclaration(_) => ok(vec![], vec![], vec![]),
        TypedDeclaration::VariableDeclaration(var_decl) => {
            convert_variable_decl_to_asm(var_decl, namespace, register_sequencer)
        }
        TypedDeclaration::ConstantDeclaration(const_decl) => {
            convert_constant_decl_to_asm(const_decl, namespace, register_sequencer)
        }
        TypedDeclaration::Reassignment(reassignment) => {
            convert_reassignment_to_asm(reassignment, namespace, register_sequencer)
        }
        _ => err(
            vec![],
            vec![CompileError::Unimplemented(
                "ASM generation has not yet been implemented for this declaration variant.",
                decl.span(),
            )],
        ),
    }
}
