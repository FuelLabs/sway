use super::{IsConstant, TypedCodeBlock, TypedExpression, TypedExpressionVariant};
use crate::error::*;
use crate::parse_tree::*;
use crate::types::{IntegerBits, TypeInfo};
use crate::{AstNode, AstNodeContent, CodeBlock, ParseTree, ReturnStatement, TraitFn};
use either::Either;
use pest::Span;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub(crate) enum TypedDeclaration<'sc> {
    VariableDeclaration(TypedVariableDeclaration<'sc>),
    FunctionDeclaration(TypedFunctionDeclaration<'sc>),
    TraitDeclaration(TypedTraitDeclaration<'sc>),
    StructDeclaration(StructDeclaration<'sc>),
    EnumDeclaration(EnumDeclaration<'sc>),
    Reassignment(TypedReassignment<'sc>),
    // no contents since it is a side-effectful declaration, i.e it populates the methods namespace
    ImplTraitDeclaration,
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
            ImplTraitDeclaration => "impl trait",
            ErrorRecovery => "invalid declaration",
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct TypedVariableDeclaration<'sc> {
    pub(crate) name: VarName<'sc>,
    pub(crate) body: TypedExpression<'sc>, // will be codeblock variant
    pub(crate) is_mutable: bool,
}

// TODO: type check generic type args and their usage
#[derive(Clone, Debug)]
pub(crate) struct TypedFunctionDeclaration<'sc> {
    pub(crate) name: VarName<'sc>,
    pub(crate) body: TypedCodeBlock<'sc>,
    pub(crate) parameters: Vec<FunctionParameter<'sc>>,
    pub(crate) span: pest::Span<'sc>,
    pub(crate) return_type: TypeInfo<'sc>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
}

#[derive(Clone, Debug)]
pub(crate) struct TypedTraitDeclaration<'sc> {
    pub(crate) name: VarName<'sc>,
    pub(crate) interface_surface: Vec<TraitFn<'sc>>, // TODO typed TraitFn which checks geneerics
    pub(crate) methods: Vec<TypedFunctionDeclaration<'sc>>,
    pub(crate) type_parameters: Vec<TypeParameter<'sc>>,
}

#[derive(Clone, Debug)]
pub(crate) struct TypedReassignment<'sc> {
    pub(crate) lhs: VarName<'sc>,
    pub(crate) rhs: TypedExpression<'sc>,
}

impl<'sc> TypedFunctionDeclaration<'sc> {
    pub(crate) fn type_check(
        fn_decl: FunctionDeclaration<'sc>,
        namespace: &HashMap<VarName<'sc>, TypedDeclaration<'sc>>,
        methods_namespace: &HashMap<TypeInfo<'sc>, Vec<TypedFunctionDeclaration<'sc>>>,
        return_type_annotation: Option<TypeInfo<'sc>>,
        help_text: impl Into<String>,
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
        } = fn_decl;
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
            TypedCodeBlock,
            body,
            &namespace,
            methods_namespace,
            Some(return_type.clone()),
            "Function body's return type does not match up with its return type annotation.",
            (TypedCodeBlock { contents: vec![] }, TypeInfo::Unit),
            warnings,
            errors
        );

        // check the generic types in the arguments, make sure they are in the type
        // scope
        let mut generic_params_buf_for_error_message = Vec::new();
        for param in parameters.iter() {
            if let TypeInfo::Generic { name } = param.r#type {
                generic_params_buf_for_error_message.push(name);
            }
        }
        let comma_separated_generic_params = generic_params_buf_for_error_message.join(", ");
        for FunctionParameter {
            ref r#type, name, ..
        } in parameters.iter()
        {
            let span = name.span.clone();
            if let TypeInfo::Generic { name, .. } = r#type {
                let args_span = parameters.iter().fold(
                    parameters[0].name.span.clone(),
                    |acc,
                     FunctionParameter {
                         name: VarName { span, .. },
                         ..
                     }| crate::utils::join_spans(acc, span.clone()),
                );
                if type_parameters.iter().find(|x| x.name == *name).is_none() {
                    errors.push(CompileError::TypeParameterNotInTypeScope {
                        name,
                        span: span.clone(),
                        comma_separated_generic_params: comma_separated_generic_params.clone(),
                        fn_name: name,
                        args: args_span.as_str(),
                        return_type: return_type.friendly_type_str(),
                    });
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
