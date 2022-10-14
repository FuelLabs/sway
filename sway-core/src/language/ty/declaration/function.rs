use std::fmt;

use sway_types::{Ident, Span, Spanned};

use crate::{
    declaration_engine::*,
    language::{ty::*, Purity, Visibility},
    type_system::*,
    AttributesMap,
};

#[derive(Clone, Debug, Eq)]
pub struct TyFunctionDeclaration {
    pub name: Ident,
    pub body: TyCodeBlock,
    pub parameters: Vec<TyFunctionParameter>,
    pub span: Span,
    pub attributes: AttributesMap,
    pub return_type: TypeId,
    pub initial_return_type: TypeId,
    pub type_parameters: Vec<TypeParameter>,
    /// Used for error messages -- the span pointing to the return type
    /// annotation of the function
    pub return_type_span: Span,
    pub(crate) visibility: Visibility,
    /// whether this function exists in another contract and requires a call to it or not
    pub(crate) is_contract_call: bool,
    pub(crate) purity: Purity,
}

impl fmt::Display for TyFunctionDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "fn {}{}({}) -> {} {{ .. }}",
            self.name,
            if self.type_parameters.is_empty() {
                String::new()
            } else {
                format!(
                    "<{}>",
                    self.type_parameters
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
            self.parameters
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            self.return_type
        )
    }
}

impl From<&TyFunctionDeclaration> for TyAstNode {
    fn from(o: &TyFunctionDeclaration) -> Self {
        let span = o.span.clone();
        TyAstNode {
            content: TyAstNodeContent::Declaration(TyDeclaration::FunctionDeclaration(
                de_insert_function(o.clone()),
            )),
            span,
        }
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyFunctionDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.body == other.body
            && self.parameters == other.parameters
            && look_up_type_id(self.return_type) == look_up_type_id(other.return_type)
            && self.type_parameters == other.type_parameters
            && self.visibility == other.visibility
            && self.is_contract_call == other.is_contract_call
            && self.purity == other.purity
    }
}

impl CopyTypes for TyFunctionDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.type_parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
        self.parameters
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
        self.return_type.copy_types(type_mapping);
        self.body.copy_types(type_mapping);
    }
}

impl Spanned for TyFunctionDeclaration {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl MonomorphizeHelper for TyFunctionDeclaration {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }

    fn name(&self) -> &Ident {
        &self.name
    }
}

impl UnconstrainedTypeParameters for TyFunctionDeclaration {
    fn type_parameter_is_unconstrained(&self, type_parameter: &TypeParameter) -> bool {
        let type_parameter_info = look_up_type_id(type_parameter.type_id);
        if self
            .type_parameters
            .iter()
            .map(|type_param| look_up_type_id(type_param.type_id))
            .any(|x| x == type_parameter_info)
        {
            return false;
        }
        if self
            .parameters
            .iter()
            .map(|param| look_up_type_id(param.type_id))
            .any(|x| x == type_parameter_info)
        {
            return true;
        }
        if look_up_type_id(self.return_type) == type_parameter_info {
            return true;
        }

        false
    }
}

#[derive(Debug, Clone, Eq)]
pub struct TyFunctionParameter {
    pub name: Ident,
    pub is_reference: bool,
    pub is_mutable: bool,
    pub mutability_span: Span,
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    pub type_span: Span,
}

impl fmt::Display for TyFunctionParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ref_str = if self.is_reference { "" } else { "ref " };
        let mut_str = if self.is_mutable { "" } else { "mut " };
        if self.is_self() {
            write!(f, "{}{}self", ref_str, mut_str,)
        } else {
            write!(f, "{}: {}{}{}", self.name, ref_str, mut_str, self.type_id)
        }
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyFunctionParameter {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
            && self.is_mutable == other.is_mutable
    }
}

impl CopyTypes for TyFunctionParameter {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.type_id.copy_types(type_mapping);
    }
}
