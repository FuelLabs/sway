use super::{
    IsConstant, TypedCodeBlock, TypedExpression, TypedExpressionVariant, TypedReturnStatement,
};
use crate::parse_tree::*;
use crate::semantics::Namespace;
use crate::{error::*, types::ResolvedType, Ident};
use pest::Span;

#[derive(Clone, Debug)]
pub enum TypedDeclaration<'sc> {
    VariableDeclaration(TypedVariableDeclaration<'sc>),
    FunctionDeclaration(TypedFunctionDeclaration<'sc>),
    TraitDeclaration(TypedTraitDeclaration<'sc>),
    StructDeclaration(TypedStructDeclaration<'sc>),
    EnumDeclaration(TypedEnumDeclaration<'sc>),
    Reassignment(TypedReassignment<'sc>),
    ImplTrait {
        trait_name: Ident<'sc>,
        span: Span<'sc>,
        methods: Vec<TypedFunctionDeclaration<'sc>>,
    },
    // no contents since it is a side-effectful declaration, i.e it populates a namespace
    SideEffect,
    ErrorRecovery,
}

impl<'sc> TypedDeclaration<'sc> {
    /// friendly name string used for error reporting.
    pub(crate) fn friendly_name(&self) -> &'static str {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(_) => "variable",
            FunctionDeclaration(_) => "function",
            TraitDeclaration(_) => "trait",
            StructDeclaration(_) => "struct",
            EnumDeclaration(_) => "enum",
            Reassignment(_) => "reassignment",
            ImplTrait { .. } => "impl trait",
            SideEffect => "",
            ErrorRecovery => "error",
        }
    }
    pub(crate) fn return_type(&self) -> CompileResult<'sc, ResolvedType<'sc>> {
        ok(
            match self {
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    body, ..
                }) => body.return_type.clone(),
                TypedDeclaration::FunctionDeclaration { .. } => {
                    return err(
                        vec![],
                        vec![CompileError::Unimplemented(
                            "Function pointers have not yet been implemented.",
                            self.span(),
                        )],
                    )
                }
                TypedDeclaration::StructDeclaration(TypedStructDeclaration {
                    name,
                    fields,
                    ..
                }) => ResolvedType::Struct {
                    name: name.clone(),
                    fields: fields.clone(),
                },
                TypedDeclaration::Reassignment(TypedReassignment { rhs, .. }) => {
                    rhs.return_type.clone()
                }
                decl => {
                    return err(
                        vec![],
                        vec![CompileError::NotAType {
                            span: decl.span(),
                            name: decl.pretty_print(),
                            actually_is: decl.friendly_name().to_string(),
                        }],
                    )
                }
            },
            vec![],
            vec![],
        )
    }

    pub(crate) fn span(&self) -> Span<'sc> {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(TypedVariableDeclaration { name, .. }) => name.span.clone(),
            FunctionDeclaration(TypedFunctionDeclaration { span, .. }) => span.clone(),
            TraitDeclaration(TypedTraitDeclaration { name, .. }) => name.span.clone(),
            StructDeclaration(TypedStructDeclaration { name, .. }) => name.span.clone(),
            EnumDeclaration(TypedEnumDeclaration { span, .. }) => span.clone(),
            Reassignment(TypedReassignment { lhs, .. }) => lhs.span.clone(),
            ImplTrait { span, .. } => span.clone(),
            SideEffect | ErrorRecovery => unreachable!("No span exists for these ast node types"),
        }
    }

    pub(crate) fn pretty_print(&self) -> String {
        format!(
            "{} declaration ({})",
            self.friendly_name(),
            match self {
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    is_mutable,
                    name,
                    ..
                }) => format!(
                    "{} {}",
                    if *is_mutable { "mut" } else { "" },
                    name.primary_name
                ),
                TypedDeclaration::FunctionDeclaration(TypedFunctionDeclaration {
                    name, ..
                }) => {
                    name.primary_name.into()
                }
                TypedDeclaration::TraitDeclaration(TypedTraitDeclaration { name, .. }) =>
                    name.primary_name.into(),
                TypedDeclaration::StructDeclaration(TypedStructDeclaration { name, .. }) =>
                    name.primary_name.into(),
                TypedDeclaration::EnumDeclaration(TypedEnumDeclaration { name, .. }) =>
                    name.primary_name.into(),
                TypedDeclaration::Reassignment(TypedReassignment { lhs, .. }) =>
                    lhs.primary_name.into(),
                _ => String::new(),
            }
        )
    }
}

#[derive(Clone, Debug)]
pub struct TypedStructDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) fields: Vec<TypedStructField<'sc>>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) visibility: Visibility,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TypedStructField<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) r#type: ResolvedType<'sc>,
    pub(crate) span: Span<'sc>,
}

#[derive(Clone, Debug)]
pub struct TypedEnumDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) variants: Vec<TypedEnumVariant<'sc>>,
    pub(crate) span: Span<'sc>,
}
impl<'sc> TypedEnumDeclaration<'sc> {
    /// Given type arguments, match them up with the type parameters and return the result.
    /// Currently unimplemented as we don't support generic enums yet, but when we do, this will be
    /// the place to resolve those typed.
    pub(crate) fn resolve_generic_types(
        &self,
        _type_arguments: Vec<ResolvedType<'sc>>,
    ) -> CompileResult<'sc, Self> {
        ok(self.clone(), vec![], vec![])
    }
    /// Returns the [ResolvedType] corresponding to this enum's type.
    pub(crate) fn as_type(&self) -> ResolvedType<'sc> {
        ResolvedType::Enum {
            name: self.name.clone(),
            variant_types: self.variants.iter().map(|x| x.r#type.clone()).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypedEnumVariant<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) r#type: ResolvedType<'sc>,
    pub(crate) tag: usize,
    pub(crate) span: Span<'sc>,
}

#[derive(Clone, Debug)]
pub struct TypedVariableDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) body: TypedExpression<'sc>, // will be codeblock variant
    pub(crate) is_mutable: bool,
}

// TODO: type check generic type args and their usage
#[derive(Clone, Debug)]
pub struct TypedFunctionDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) body: TypedCodeBlock<'sc>,
    pub(crate) parameters: Vec<TypedFunctionParameter<'sc>>,
    pub(crate) span: pest::Span<'sc>,
    pub(crate) return_type: ResolvedType<'sc>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    /// Used for error messages -- the span pointing to the return type
    /// annotation of the function
    pub(crate) return_type_span: Span<'sc>,
    pub(crate) visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedFunctionParameter<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) r#type: ResolvedType<'sc>,
    pub(crate) type_span: Span<'sc>,
}

#[derive(Clone, Debug)]
pub struct TypedTraitDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) interface_surface: Vec<TypedTraitFn<'sc>>,
    pub(crate) methods: Vec<TypedFunctionDeclaration<'sc>>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
    pub(crate) visibility: Visibility,
}
#[derive(Clone, Debug)]
pub struct TypedTraitFn<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) parameters: Vec<TypedFunctionParameter<'sc>>,
    pub(crate) return_type: ResolvedType<'sc>,
}

#[derive(Clone, Debug)]
pub struct TypedReassignment<'sc> {
    pub(crate) lhs: Ident<'sc>,
    pub(crate) rhs: TypedExpression<'sc>,
}

impl<'sc> TypedFunctionDeclaration<'sc> {
    pub(crate) fn type_check(
        fn_decl: FunctionDeclaration<'sc>,
        namespace: &Namespace<'sc>,
        _return_type_annotation: Option<ResolvedType<'sc>>,
        _help_text: impl Into<String>,
        // If there are any `Self` types in this declaration,
        // resolve them to this type.
        self_type: Option<ResolvedType<'sc>>,
    ) -> CompileResult<'sc, TypedFunctionDeclaration<'sc>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let FunctionDeclaration {
            name,
            body,
            parameters,
            span,
            return_type,
            type_parameters,
            return_type_span,
            visibility,
            ..
        } = fn_decl.clone();
        let return_type = namespace.resolve_type(&return_type);
        // insert parameters into namespace
        let mut namespace = namespace.clone();
        for FunctionParameter {
            name, ref r#type, ..
        } in parameters.clone()
        {
            let r#type = namespace.resolve_type(r#type);
            namespace.insert(
                name.clone(),
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    name: name.clone(),
                    body: TypedExpression {
                        expression: TypedExpressionVariant::FunctionParameter,
                        return_type: r#type,
                        is_constant: IsConstant::No,
                        span: name.span.clone(),
                    },
                    is_mutable: false, // TODO allow mutable function params?
                }),
            );
        }
        // check return type for Self types
        let return_type = if return_type == ResolvedType::SelfType {
            match self_type {
                Some(ref ty) => ty.clone(),
                None => {
                    errors.push(CompileError::UnqualifiedSelfType {
                        span: return_type_span.clone(),
                    });
                    return_type
                }
            }
        } else {
            return_type
        };
        // If there are no implicit block returns, then we do not want to type check them, so we
        // stifle the errors. If there _are_ implicit block returns, we want to type_check them.
        let (body, _implicit_block_return) = type_check!(
            TypedCodeBlock::type_check(
                body.clone(),
                &namespace,
                Some(return_type.clone()),
                "Function body's return type does not match up with its return type annotation."
            ),
            (
                TypedCodeBlock {
                    contents: vec![],
                    whole_block_span: body.whole_block_span.clone()
                },
                Some(ResolvedType::ErrorRecovery)
            ),
            warnings,
            errors
        );

        // check the generic types in the arguments, make sure they are in the type
        // scope
        let mut parameters = parameters
            .into_iter()
            .map(
                |FunctionParameter {
                     name,
                     r#type,
                     type_span,
                 }| TypedFunctionParameter {
                    name,
                    r#type: namespace.resolve_type(&r#type),
                    type_span,
                },
            )
            .collect::<Vec<_>>();
        let mut generic_params_buf_for_error_message = Vec::new();
        for param in parameters.iter() {
            if let ResolvedType::Generic { ref name } = param.r#type {
                generic_params_buf_for_error_message.push(name.primary_name);
            }
        }
        let comma_separated_generic_params = generic_params_buf_for_error_message.join(", ");
        for TypedFunctionParameter {
            ref r#type, name, ..
        } in parameters.iter()
        {
            let span = name.span.clone();
            if let ResolvedType::Generic { name, .. } = r#type {
                let args_span = parameters.iter().fold(
                    parameters[0].name.span.clone(),
                    |acc,
                     TypedFunctionParameter {
                         name: Ident { span, .. },
                         ..
                     }| crate::utils::join_spans(acc, span.clone()),
                );
                if type_parameters
                    .iter()
                    .find(|x| x.name == name.primary_name)
                    .is_none()
                {
                    errors.push(CompileError::TypeParameterNotInTypeScope {
                        name: name.primary_name,
                        span: span.clone(),
                        comma_separated_generic_params: comma_separated_generic_params.clone(),
                        fn_name: fn_decl.name.primary_name,
                        args: args_span.as_str(),
                    });
                }
            }
        }
        // handle the return statement(s)
        let return_statements: Vec<(&TypedExpression, &pest::Span<'sc>)> =
            body.contents
                .iter()
                .filter_map(|x| {
                    if let crate::semantics::TypedAstNode {
                        content:
                            crate::semantics::TypedAstNodeContent::ReturnStatement(
                                TypedReturnStatement { ref expr },
                            ),
                        span,
                    } = x
                    {
                        Some((expr, span))
                    } else {
                        None
                    }
                })
                .collect();
        for (stmt, span) in return_statements {
            let convertability = stmt.return_type.is_convertable(
                &return_type,
                span.clone(),
                "Function body's return type does not match up with its return type annotation.",
            );
            match convertability {
                Ok(warning) => {
                    if let Some(warning) = warning {
                        warnings.push(CompileWarning {
                            warning_content: warning,
                            span: span.clone(),
                        });
                    }
                }
                Err(err) => {
                    errors.push(err.into());
                }
            }
        }
        for TypedFunctionParameter {
            ref mut r#type,
            type_span,
            ..
        } in parameters.iter_mut()
        {
            if *r#type == ResolvedType::SelfType {
                match self_type {
                    Some(ref ty) => *r#type = ty.clone(),
                    None => {
                        errors.push(CompileError::UnqualifiedSelfType {
                            span: type_span.clone(),
                        });
                        continue;
                    }
                }
            }
        }

        ok(
            TypedFunctionDeclaration {
                name,
                body,
                parameters,
                span: span.clone(),
                return_type,
                type_parameters,
                return_type_span,
                visibility,
            },
            warnings,
            errors,
        )
    }
}
