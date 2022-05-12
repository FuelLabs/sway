use crate::{
    error::*,
    parse_tree::*,
    semantic_analysis::{ast_node::TypedStructDeclaration, monomorphization::*, namespace},
    span::Span,
    type_engine::*,
    Ident, TypeParameter,
};
use std::slice::IterMut;

impl<'a> Monomorphizable<'a, IterMut<'a, TypeParameter>> for TypedStructDeclaration {
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
        self.fields
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }

    fn as_type(&self) -> TypeInfo {
        TypeInfo::Struct {
            name: self.name.clone(),
            fields: self.fields.clone(),
            type_parameters: self.type_parameters.clone(),
        }
    }
}
