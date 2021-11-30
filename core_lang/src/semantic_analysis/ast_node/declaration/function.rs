use crate::{type_engine::*, TypeParameter};

/// Insert all type parameters as unknown types. Return a mapping of type parameter to
/// [TypeId]
pub(crate) fn insert_type_parameters<'sc>(
    params: &[TypeParameter<'sc>],
) -> Vec<(TypeParameter<'sc>, TypeId)> {
    params
        .into_iter()
        .map(|x| {
            (
                x.clone(),
                insert_type(TypeInfo::UnknownGeneric {
                    name: x.name_ident.primary_name.to_string(),
                }),
            )
        })
        .collect()
}
