use crate::semantic_analysis::TypedExpression;
use crate::type_engine::*;
use crate::Ident;
use crate::{type_engine::TypeId, TypeParameter};

#[derive(Clone, Debug)]
pub struct TypedVariableDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) body: TypedExpression<'sc>, // will be codeblock variant
    pub(crate) is_mutable: bool,
    pub(crate) type_ascription: TypeId,
}

impl TypedVariableDeclaration<'_> {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        if let Some(matching_id) =
            look_up_type_id(self.type_ascription).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.type_ascription))
        };

        self.body.copy_types(type_mapping)
    }
}
