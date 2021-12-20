use crate::span::Span;
use crate::type_engine::*;

use crate::Ident;
use crate::TypeParameter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedFunctionParameter {
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
    pub(crate) type_span: Span,
}

impl TypedFunctionParameter {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.r#type = if let Some(matching_id) =
            look_up_type_id(self.r#type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.r#type))
        }
    }
}
