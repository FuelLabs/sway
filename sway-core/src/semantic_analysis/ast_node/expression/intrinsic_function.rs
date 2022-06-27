use std::fmt;

use sway_types::Span;

use crate::{
    error::{err, ok},
    semantic_analysis::{EnforceTypeArguments, TypeCheckContext},
    type_engine::*,
    types::DeterministicallyAborts,
    CompileError, CompileResult, IntrinsicFunctionKind,
};

use super::TypedExpression;

#[derive(Debug, Clone)]
pub enum TypedIntrinsicFunctionKind {
    SizeOfVal { exp: Box<TypedExpression> },
    SizeOfType { type_id: TypeId, type_span: Span },
    IsRefType { type_id: TypeId, type_span: Span },
    GetStorageKey,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedIntrinsicFunctionKind {
    fn eq(&self, other: &Self) -> bool {
        use TypedIntrinsicFunctionKind::*;
        match (self, other) {
            (SizeOfVal { exp: l_exp }, SizeOfVal { exp: r_exp }) => *l_exp == *r_exp,
            (
                SizeOfType {
                    type_id: l_type_id, ..
                },
                SizeOfType {
                    type_id: r_type_id, ..
                },
            ) => look_up_type_id(*l_type_id) == look_up_type_id(*r_type_id),
            (
                IsRefType {
                    type_id: l_type_id, ..
                },
                IsRefType {
                    type_id: r_type_id, ..
                },
            ) => look_up_type_id(*l_type_id) == look_up_type_id(*r_type_id),
            (GetStorageKey, GetStorageKey) => true,
            _ => false,
        }
    }
}

impl CopyTypes for TypedIntrinsicFunctionKind {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        use TypedIntrinsicFunctionKind::*;
        match self {
            SizeOfVal { exp } => {
                exp.copy_types(type_mapping);
            }
            SizeOfType { type_id, type_span } => {
                type_id.update_type(type_mapping, type_span);
            }
            IsRefType { type_id, type_span } => {
                type_id.update_type(type_mapping, type_span);
            }
            GetStorageKey => {}
        }
    }
}

impl fmt::Display for TypedIntrinsicFunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TypedIntrinsicFunctionKind::*;
        let s = match self {
            SizeOfVal { exp } => format!("size_of_val({})", exp),
            SizeOfType { type_id, .. } => format!("size_of({})", look_up_type_id(*type_id)),
            IsRefType { type_id, .. } => format!("is_ref_type({})", look_up_type_id(*type_id)),
            GetStorageKey => "get_storage_key".to_string(),
        };
        write!(f, "{}", s)
    }
}

impl DeterministicallyAborts for TypedIntrinsicFunctionKind {
    fn deterministically_aborts(&self) -> bool {
        use TypedIntrinsicFunctionKind::*;
        match self {
            SizeOfVal { exp } => exp.deterministically_aborts(),
            SizeOfType { .. } | GetStorageKey | IsRefType { .. } => false,
        }
    }
}

impl UnresolvedTypeCheck for TypedIntrinsicFunctionKind {
    fn check_for_unresolved_types(&self) -> Vec<CompileError> {
        use TypedIntrinsicFunctionKind::*;
        match self {
            SizeOfVal { exp } => exp.check_for_unresolved_types(),
            SizeOfType { type_id, .. } => type_id.check_for_unresolved_types(),
            IsRefType { type_id, .. } => type_id.check_for_unresolved_types(),
            GetStorageKey => vec![],
        }
    }
}

impl TypedIntrinsicFunctionKind {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        kind: IntrinsicFunctionKind,
    ) -> CompileResult<(Self, TypeId)> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let (intrinsic_function, return_type) = match kind {
            IntrinsicFunctionKind::SizeOfVal { exp } => {
                let ctx = ctx
                    .with_help_text("")
                    .with_type_annotation(insert_type(TypeInfo::Unknown));
                let exp = check!(
                    TypedExpression::type_check(ctx, *exp),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let intrinsic_function =
                    TypedIntrinsicFunctionKind::SizeOfVal { exp: Box::new(exp) };
                let return_type = insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour));
                (intrinsic_function, return_type)
            }
            IntrinsicFunctionKind::SizeOfType {
                type_name,
                type_span,
            } => {
                let type_id = check!(
                    ctx.resolve_type_with_self(
                        insert_type(type_name),
                        &type_span,
                        EnforceTypeArguments::Yes
                    ),
                    insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors,
                );
                let intrinsic_function =
                    TypedIntrinsicFunctionKind::SizeOfType { type_id, type_span };
                let return_type = insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour));
                (intrinsic_function, return_type)
            }
            IntrinsicFunctionKind::IsRefType {
                type_name,
                type_span,
            } => {
                let type_id = check!(
                    ctx.resolve_type_with_self(
                        insert_type(type_name),
                        &type_span,
                        EnforceTypeArguments::Yes
                    ),
                    insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors,
                );
                let intrinsic_function =
                    TypedIntrinsicFunctionKind::IsRefType { type_id, type_span };
                (intrinsic_function, insert_type(TypeInfo::Boolean))
            }
            IntrinsicFunctionKind::GetStorageKey => (
                TypedIntrinsicFunctionKind::GetStorageKey,
                insert_type(TypeInfo::B256),
            ),
        };
        ok((intrinsic_function, return_type), warnings, errors)
    }
}
