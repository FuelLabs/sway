use crate::{
    error::*,
    parse_tree::*,
    semantic_analysis::{ast_node::TypedStructDeclaration, namespace},
    span::Span,
    type_engine::*,
    Ident, TypeParameter,
};

mod r#enum;
mod r#struct;

pub fn monomorphize_implemented_traits(
    ty: TypeInfo,

    namespace: &mut namespace::Items,
    type_mapping: &[(TypeParameter, TypeId)],
) {
    todo!()
}

/// Insert all type parameters as unknown types. Return a mapping of type parameter to
/// [TypeId]
pub(crate) fn insert_type_parameters(
    type_parameters: &[TypeParameter],
) -> Vec<(TypeParameter, TypeId)> {
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
