use super::{
    IsConstant, TypedCodeBlock, TypedExpression, TypedExpressionVariant, TypedReturnStatement,
};
use crate::error::*;
use crate::parse_tree::*;
use crate::semantics::Namespace;
use crate::types::TypeInfo;
use crate::TraitFn;

#[derive(Clone, Debug)]
pub enum TypedDeclaration<'sc> {
    VariableDeclaration(TypedVariableDeclaration<'sc>),
    FunctionDeclaration(TypedFunctionDeclaration<'sc>),
    TraitDeclaration(TypedTraitDeclaration<'sc>),
    StructDeclaration(StructDeclaration<'sc>),
    EnumDeclaration(EnumDeclaration<'sc>),
    Reassignment(TypedReassignment<'sc>),
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
            SideEffect => "",
            ErrorRecovery => "error",
        }
    }
    pub(crate) fn return_type(&self) -> CompileResult<'sc, TypeInfo<'sc>> {
        ok(
            match self {
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    body, ..
                }) => body.return_type.clone(),
                TypedDeclaration::FunctionDeclaration { .. } => todo!("fn pointer type"),
                TypedDeclaration::StructDeclaration(StructDeclaration { name, .. }) => {
                    TypeInfo::Struct { name: name.clone() }
                }
                TypedDeclaration::Reassignment(TypedReassignment { rhs, .. }) => {
                    rhs.return_type.clone()
                }
                _ => return err(vec![], vec![todo!("used typeless symbol as type err")]),
            },
            vec![],
            vec![],
        )
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
                TypedDeclaration::StructDeclaration(StructDeclaration { name, .. }) =>
                    name.primary_name.into(),
                TypedDeclaration::EnumDeclaration(EnumDeclaration { name, .. }) => name.to_string(),
                TypedDeclaration::Reassignment(TypedReassignment { lhs, .. }) =>
                    lhs.primary_name.into(),
                _ => String::new(),
            }
        )
    }
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
    pub(crate) parameters: Vec<FunctionParameter<'sc>>,
    pub(crate) span: pest::Span<'sc>,
    pub(crate) return_type: TypeInfo<'sc>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
}

#[derive(Clone, Debug)]
pub struct TypedTraitDeclaration<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) interface_surface: Vec<TraitFn<'sc>>, // TODO typed TraitFn which checks geneerics
    pub(crate) methods: Vec<TypedFunctionDeclaration<'sc>>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
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
        _return_type_annotation: Option<TypeInfo<'sc>>,
        _help_text: impl Into<String>,
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
            ..
        } = fn_decl.clone();
        // insert parameters into namespace
        let mut namespace = namespace.clone();
        parameters
            .clone()
            .into_iter()
            .for_each(|FunctionParameter { name, r#type, .. }| {
                namespace.insert(
                    name.clone(),
                    TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                        name: name.clone(),
                        body: TypedExpression {
                            expression: TypedExpressionVariant::FunctionParameter,
                            return_type: r#type,
                            is_constant: IsConstant::No,
                        },
                        is_mutable: false, // TODO allow mutable function params?
                    }),
                );
            });
        let (body, _implicit_block_return) = type_check!(
            TypedCodeBlock::type_check(
                body,
                &namespace,
                Some(return_type.clone()),
                "Function body's return type does not match up with its return type annotation."
            ),
            (TypedCodeBlock { contents: vec![] }, TypeInfo::ErrorRecovery),
            warnings,
            errors
        );

        // check the generic types in the arguments, make sure they are in the type
        // scope
        let mut generic_params_buf_for_error_message = Vec::new();
        for param in parameters.iter() {
            if let TypeInfo::Custom { ref name } = param.r#type {
                generic_params_buf_for_error_message.push(name.primary_name);
            }
        }
        let comma_separated_generic_params = generic_params_buf_for_error_message.join(", ");
        for FunctionParameter {
            ref r#type, name, ..
        } in parameters.iter()
        {
            let span = name.span.clone();
            if let TypeInfo::Custom { name, .. } = r#type {
                let args_span = parameters.iter().fold(
                    parameters[0].name.span.clone(),
                    |acc,
                     FunctionParameter {
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

        ok(
            TypedFunctionDeclaration {
                name,
                body,
                parameters,
                span: span.clone(),
                return_type,
                type_parameters,
            },
            warnings,
            errors,
        )
    }
}
