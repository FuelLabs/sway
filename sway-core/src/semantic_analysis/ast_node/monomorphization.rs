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

pub trait Monomorphizable<'a, I: Iterator<Item = &'a mut TypeParameter>>: Clone + Sized {
    fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]);

    fn type_parameters(&self) -> &[TypeParameter];

    fn type_parameters_iter_mut(&'a mut self) -> I;

    fn span(&self) -> &Span;

    fn as_type(&self) -> TypeInfo;

    fn monomorphize(
        &self,
        namespace: &mut namespace::Items,
        type_arguments: &[TypeArgument],
        self_type: Option<TypeId>,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_parameters = self.type_parameters();
        let type_mapping = insert_type_parameters(type_parameters);
        let mut new_decl = Self::monomorphize_inner(self, namespace, &type_mapping);
        let type_arguments_span = type_arguments
            .iter()
            .map(|x| x.span.clone())
            .reduce(Span::join)
            .unwrap_or_else(|| self.span().clone());
        if !type_arguments.is_empty() {
            if type_mapping.len() != type_arguments.len() {
                errors.push(CompileError::IncorrectNumberOfTypeArguments {
                    given: type_arguments.len(),
                    expected: type_mapping.len(),
                    span: type_arguments_span,
                });
                return err(warnings, errors);
            }
            for ((_, interim_type), type_argument) in type_mapping.iter().zip(type_arguments.iter())
            {
                match self_type {
                    Some(self_type) => {
                        let (mut new_warnings, new_errors) = unify_with_self(
                            *interim_type,
                            type_argument.type_id,
                            self_type,
                            &type_argument.span,
                            "Type argument is not assignable to generic type parameter.",
                        );
                        warnings.append(&mut new_warnings);
                        errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
                    }
                    None => {
                        let (mut new_warnings, new_errors) = unify(
                            *interim_type,
                            type_argument.type_id,
                            &type_argument.span,
                            "Type argument is not assignable to generic type parameter.",
                        );
                        warnings.append(&mut new_warnings);
                        errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
                    }
                }
            }
            // associate the type arguments with the parameters in the struct decl
            new_decl
                .type_parameters_iter_mut()
                .zip(type_arguments.iter())
                .for_each(
                    |(
                        TypeParameter {
                            ref mut type_id, ..
                        },
                        arg,
                    )| {
                        *type_id = arg.type_id;
                    },
                );
        }
        ok(new_decl, warnings, errors)
    }

    fn type_id(&self) -> TypeId {
        insert_type(self.as_type())
    }

    fn monomorphize_inner(
        &self,
        namespace: &mut namespace::Items,
        type_mapping: &[(TypeParameter, TypeId)],
    ) -> Self {
        let old_type_id = self.type_id();
        let mut new_decl = self.clone();
        todo!("diff between `monomorphize_impl_traits` and `copy_methods_to_type`?");
        monomorphize_implemented_traits(self.as_type(), namespace, type_mapping);
        new_decl.copy_types(type_mapping);
        namespace.copy_methods_to_type(
            look_up_type_id(old_type_id),
            look_up_type_id(new_decl.type_id()),
            type_mapping,
        );
        new_decl
    }
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
