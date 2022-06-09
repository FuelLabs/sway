use crate::{error::*, parse_tree::*, semantic_analysis::*, type_engine::*};

use sway_types::{ident::Ident, span::Span, Spanned};

use std::{
    convert::From,
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone, Eq)]
pub struct TypeParameter {
    pub(crate) type_id: TypeId,
    pub(crate) name_ident: Ident,
    pub(crate) trait_constraints: Vec<TraitConstraint>,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl Hash for TypeParameter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        look_up_type_id(self.type_id).hash(state);
        self.name_ident.hash(state);
        self.trait_constraints.hash(state);
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypeParameter {
    fn eq(&self, other: &Self) -> bool {
        look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
            && self.name_ident == other.name_ident
            && self.trait_constraints == other.trait_constraints
    }
}

impl From<&TypeParameter> for TypedDeclaration {
    fn from(n: &TypeParameter) -> Self {
        TypedDeclaration::GenericTypeForFunctionScope {
            name: n.name_ident.clone(),
            type_id: n.type_id,
        }
    }
}

impl CopyTypes for TypeParameter {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.type_id = match look_up_type_id(self.type_id).matches_type_parameter(type_mapping) {
            Some(matching_id) => insert_type(TypeInfo::Ref(matching_id, self.name_ident.span())),
            None => {
                let ty = TypeInfo::Ref(insert_type(look_up_type_id_raw(self.type_id)), self.span());
                insert_type(ty)
            }
        };
    }
}

impl Spanned for TypeParameter {
    fn span(&self) -> Span {
        self.name_ident.span()
    }
}

impl TypeParameter {
    pub(crate) fn type_check(
        type_parameter: &mut TypeParameter,
        namespace: &mut Namespace,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];
        // TODO: add check here to see if the type parameter has a valid name
        type_parameter.type_id = insert_type(TypeInfo::UnknownGeneric {
            name: type_parameter.name_ident.clone(),
        });
        let type_parameter_decl = TypedDeclaration::GenericTypeForFunctionScope {
            name: type_parameter.name_ident.clone(),
            type_id: type_parameter.type_id,
        };
        check!(
            namespace.insert_symbol(type_parameter.name_ident.clone(), type_parameter_decl),
            return err(warnings, errors),
            warnings,
            errors
        );
        ok((), warnings, errors)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct TraitConstraint {
    pub(crate) call_path: CallPath,
}
