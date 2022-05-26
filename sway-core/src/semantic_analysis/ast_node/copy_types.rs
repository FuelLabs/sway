use crate::{
    type_engine::{insert_type, look_up_type_id, look_up_type_id_raw, TypeId},
    TypeInfo, TypeParameter,
};
use sway_types::Span;

pub(crate) type TypeMapping = Vec<(TypeParameter, TypeId)>;

pub(crate) fn insert_type_parameters(type_parameters: &[TypeParameter]) -> TypeMapping {
    type_parameters
        .iter()
        .map(|x| {
            (
                x.clone(),
                insert_type(TypeInfo::UnknownGeneric {
                    name: x.name_ident.clone(),
                }),
            )
        })
        .collect()
}

pub(crate) trait CopyTypes {
    fn copy_types(&mut self, type_mapping: &TypeMapping);
}

impl TypeId {
    pub(crate) fn update_type(&mut self, type_mapping: &TypeMapping, span: &Span) {
        *self = match look_up_type_id(*self).matches_type_parameter(type_mapping) {
            Some(matching_id) => insert_type(TypeInfo::Ref(matching_id, span.clone())),
            None => {
                let ty = TypeInfo::Ref(insert_type(look_up_type_id_raw(*self)), span.clone());
                insert_type(ty)
            }
        };
    }
}
