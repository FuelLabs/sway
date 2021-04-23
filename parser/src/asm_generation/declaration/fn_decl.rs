use crate::{
    asm_generation::{AsmNamespace, RegisterSequencer},
    vendored_vm::Op,
    TypedFunctionDeclaration,
};

pub(crate) fn convert_fn_decl_to_asm<'sc>(
    _decl: &TypedFunctionDeclaration<'sc>,
    _namespace: &mut AsmNamespace,
    _register_sequencer: &mut RegisterSequencer,
) -> Vec<Op<'sc>> {
    // for now, we inline all functions as a shortcut.
    vec![]
}
