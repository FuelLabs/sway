use crate::{
    error::*,
    parse_tree::*,
    semantic_analysis::{ast_node::TypedStructDeclaration, monomorphization::*, namespace},
    span::Span,
    type_engine::*,
    Ident, TypeParameter,
};
impl TypedStructDeclaration {
    pub(crate) fn monomorphize(
        &self,
        namespace: &mut namespace::Items,
        type_arguments: &[TypeArgument],
        self_type: Option<TypeId>,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_mapping = insert_type_parameters(&self.type_parameters);
        let mut new_decl = Self::monomorphize_inner(self, namespace, &type_mapping);
        let type_arguments_span = type_arguments
            .iter()
            .map(|x| x.span.clone())
            .reduce(Span::join)
            .unwrap_or_else(|| self.span.clone());
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
                .type_parameters
                .iter_mut()
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
    fn monomorphize_inner(
        &self,
        namespace: &mut namespace::Items,
        type_mapping: &[(TypeParameter, TypeId)],
    ) -> Self {
        let old_type_id = self.type_id();
        let mut new_decl = self.clone();
        monomorphize_implemented_traits(self.as_type(), namespace, type_mapping);
        new_decl.copy_types(type_mapping);
        namespace.copy_methods_to_type(
            look_up_type_id(old_type_id),
            look_up_type_id(new_decl.type_id()),
            type_mapping,
        );
        new_decl
    }

    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.fields
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}
