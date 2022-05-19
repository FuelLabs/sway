use crate::{
    parse_tree::{declaration::TypeParameter, Visibility},
    type_engine::{look_up_type_id, TypeId, TypeInfo},
    CodeBlock,
};

use fuels_types::{Function, Property};
use sway_types::{ident::Ident, span::Span};

mod purity;
pub use purity::{promote_purity, Purity};

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub purity: Purity,
    pub name: Ident,
    pub visibility: Visibility,
    pub body: CodeBlock,
    pub parameters: Vec<FunctionParameter>,
    pub span: Span,
    pub return_type: TypeInfo,
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) return_type_span: Span,
}

impl FunctionDeclaration {
    pub fn parse_json_abi(&self) -> Function {
        Function {
            name: self.name.as_str().to_string(),
            type_field: "function".to_string(),
            inputs: self
                .parameters
                .iter()
                .map(|x| Property {
                    name: x.name.as_str().to_string(),
                    type_field: look_up_type_id(x.type_id).friendly_type_str(),
                    components: None,
                })
                .collect(),
            outputs: vec![Property {
                name: "".to_string(),
                type_field: self.return_type.friendly_type_str(),
                components: None,
            }],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionParameter {
    pub name: Ident,
    pub(crate) type_id: TypeId,
    pub(crate) type_span: Span,
}
