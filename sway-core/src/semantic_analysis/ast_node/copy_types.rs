use crate::{
    type_engine::{insert_type, TypeId},
    TypeInfo, TypeParameter,
};

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
