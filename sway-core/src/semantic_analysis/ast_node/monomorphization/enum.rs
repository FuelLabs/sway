use crate::{
    error::*,
    parse_tree::*,
    semantic_analysis::{
        ast_node::{TypedEnumDeclaration, TypedEnumVariant},
        monomorphization::*,
        namespace,
    },
    span::Span,
    type_engine::*,
    Ident, TypeParameter,
};
use std::slice::IterMut;
impl<'a> Monomorphizable<'a, IterMut<'a, TypeParameter>> for TypedEnumDeclaration {
    fn type_parameters(&self) -> &[TypeParameter] {
        &self.type_parameters
    }
    fn span(&self) -> &Span {
        &self.span
    }

    fn type_parameters_iter_mut(&'a mut self) -> IterMut<'a, TypeParameter> {
        self.type_parameters.iter_mut()
    }

    fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.variants
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
    fn as_type(&self) -> TypeInfo {
        TypeInfo::Enum {
            name: self.name.clone(),
            variant_types: self.variants.clone(),
            type_parameters: self.type_parameters.clone(),
        }
    }
}

impl TypedEnumVariant {
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.r#type = if let Some(matching_id) =
            look_up_type_id(self.r#type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.r#type))
        };
    }
}
