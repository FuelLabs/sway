use derivative::Derivative;
use sway_types::{Ident, Span};

use crate::{
    language::{ty::*, Purity, Visibility},
    type_system::*,
};

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TyTraitFn {
    pub name: Ident,
    pub(crate) purity: Purity,
    pub parameters: Vec<TyFunctionParameter>,
    pub return_type: TypeId,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub return_type_span: Span,
}

impl CopyTypes for TyTraitFn {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.return_type.copy_types(type_mapping);
    }
}

impl TyTraitFn {
    /// This function is used in trait declarations to insert "placeholder" functions
    /// in the methods. This allows the methods to use functions declared in the
    /// interface surface.
    pub(crate) fn to_dummy_func(&self, mode: Mode) -> TyFunctionDeclaration {
        TyFunctionDeclaration {
            purity: self.purity,
            name: self.name.clone(),
            body: TyCodeBlock { contents: vec![] },
            parameters: self.parameters.clone(),
            span: self.name.span(),
            return_type: self.return_type,
            initial_return_type: self.return_type,
            return_type_span: self.return_type_span.clone(),
            visibility: Visibility::Public,
            type_parameters: vec![],
            is_contract_call: mode == Mode::ImplAbiFn,
        }
    }
}
