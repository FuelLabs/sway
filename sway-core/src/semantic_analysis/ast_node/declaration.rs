mod abi;
mod r#enum;
mod function;
mod impl_trait;
mod storage;
mod r#struct;
mod r#trait;

pub use abi::*;
pub use function::*;
pub use impl_trait::*;
pub use r#enum::*;
pub use r#struct::*;
pub use r#trait::*;
pub use storage::*;

use crate::{
    language::{ty, *},
    semantic_analysis::*,
};

use sway_types::Spanned;

impl ty::TyTraitFn {
    /// This function is used in trait declarations to insert "placeholder" functions
    /// in the methods. This allows the methods to use functions declared in the
    /// interface surface.
    pub(crate) fn to_dummy_func(&self, mode: Mode) -> ty::TyFunctionDeclaration {
        ty::TyFunctionDeclaration {
            purity: self.purity,
            inline: self.inline,
            name: self.name.clone(),
            body: ty::TyCodeBlock { contents: vec![] },
            parameters: self.parameters.clone(),
            span: self.name.span(),
            attributes: self.attributes.clone(),
            return_type: self.return_type,
            initial_return_type: self.return_type,
            return_type_span: self.return_type_span.clone(),
            visibility: Visibility::Public,
            type_parameters: vec![],
            is_contract_call: mode == Mode::ImplAbiFn,
        }
    }
}
