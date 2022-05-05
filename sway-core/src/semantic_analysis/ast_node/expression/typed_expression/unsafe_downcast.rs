use sway_types::{Ident, Span};

use crate::{
    error::{err, ok},
    semantic_analysis::TypedEnumVariant,
    CallPath, CompileError, CompileResult, NamespaceRef, NamespaceWrapper, TypedDeclaration,
};

use super::TypedExpression;

// currently the unsafe downcast expr is only used for enums, so this method is specialized for enums
pub(crate) fn instantiate_unsafe_downcast(
    call_path: CallPath,
    parent: TypedExpression,
    field_to_access: Ident,
    span: Span,
    namespace: NamespaceRef,
) -> CompileResult<TypedExpression> {
    //let mut warnings = vec!();
    //let mut errors = vec!();
    unimplemented!()
}
