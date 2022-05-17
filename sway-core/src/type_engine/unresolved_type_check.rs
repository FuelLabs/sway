//! This module handles the process of iterating through the typed AST and ensuring that all types
//! are well-defined and well-formed. This process is run on the AST before we pass it into the IR,
//! as the IR assumes all types are well-formed and will throw an ICE (internal compiler error) if
//! that is not the case.

use crate::{semantic_analysis::*, type_engine::*};

/// If any types contained by this node are unresolved or have yet to be inferred, throw an
/// error to signal to the user that more type information is needed.
pub(crate) trait UnresolvedTypeCheck {
    fn check_for_unresolved_types(&self) -> Vec<CompileError>;
}

impl UnresolvedTypeCheck for TypeId {
    fn check_for_unresolved_types(&self) -> Vec<CompileError> {
        use TypeInfo::*;
        match look_up_type_id(*self) {
            UnknownGeneric { name } => vec![CompileError::UnableToInferGeneric {
                ty: name.as_str().to_string(),
                span: name.span().clone(),
            }],
            _ => vec![],
        }
    }
}

impl UnresolvedTypeCheck for TypedAstNodeContent {
    fn check_for_unresolved_types(&self) -> Vec<CompileError> {
        use TypedAstNodeContent::*;
        match self {
            ReturnStatement(stmt) => stmt.expr.check_for_unresolved_types(),
            Declaration(decl) => decl.check_for_unresolved_types(),
            Expression(expr) => expr.check_for_unresolved_types(),
            ImplicitReturnExpression(expr) => expr.check_for_unresolved_types(),
            WhileLoop(lo) => {
                let mut condition = lo.condition.check_for_unresolved_types();
                let mut body = lo
                    .body
                    .contents
                    .iter()
                    .flat_map(TypedAstNode::check_for_unresolved_types)
                    .collect();
                condition.append(&mut body);
                condition
            }
            SideEffect => vec![],
        }
    }
}

impl UnresolvedTypeCheck for TypedExpression {
    fn check_for_unresolved_types(&self) -> Vec<CompileError> {
        use TypedExpressionVariant::*;
        match &self.expression {
            TypeProperty { type_id, .. } => type_id.check_for_unresolved_types(),
            FunctionApplication {
                arguments,
                function_body,
                ..
            } => {
                let mut args = arguments
                    .iter()
                    .map(|x| &x.1)
                    .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                    .collect::<Vec<_>>();
                args.append(
                    &mut function_body
                        .contents
                        .iter()
                        .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                        .collect(),
                );
                args
            }
            // expressions don't ever have return types themselves, they're stored in
            // `TypedExpression::return_type`. Variable expressions are just names of variables.
            VariableExpression { .. } => vec![],
            Tuple { fields } => fields
                .iter()
                .flat_map(|x| x.check_for_unresolved_types())
                .collect(),
            AsmExpression { registers, .. } => registers
                .iter()
                .filter_map(|x| x.initializer.as_ref())
                .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                .collect::<Vec<_>>(),
            StructExpression { fields, .. } => fields
                .iter()
                .flat_map(|x| x.value.check_for_unresolved_types())
                .collect(),
            LazyOperator { lhs, rhs, .. } => lhs
                .check_for_unresolved_types()
                .into_iter()
                .chain(rhs.check_for_unresolved_types().into_iter())
                .collect(),
            Array { contents } => contents
                .iter()
                .flat_map(|x| x.check_for_unresolved_types())
                .collect(),
            ArrayIndex { prefix, .. } => prefix.check_for_unresolved_types(),
            CodeBlock(block) => block
                .contents
                .iter()
                .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                .collect(),
            IfExp {
                condition,
                then,
                r#else,
            } => {
                let mut buf = condition
                    .check_for_unresolved_types()
                    .into_iter()
                    .chain(then.check_for_unresolved_types().into_iter())
                    .collect::<Vec<_>>();
                if let Some(r#else) = r#else {
                    buf.append(&mut r#else.check_for_unresolved_types());
                }
                buf
            }
            StructFieldAccess {
                prefix,
                resolved_type_of_parent,
                ..
            } => prefix
                .check_for_unresolved_types()
                .into_iter()
                .chain(
                    resolved_type_of_parent
                        .check_for_unresolved_types()
                        .into_iter(),
                )
                .collect(),
            IfLet {
                enum_type,
                expr,
                then,
                r#else,
                ..
            } => {
                let mut buf = enum_type
                    .check_for_unresolved_types()
                    .into_iter()
                    .chain(expr.check_for_unresolved_types().into_iter())
                    .chain(
                        then.contents
                            .iter()
                            .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                            .into_iter(),
                    )
                    .collect::<Vec<_>>();
                if let Some(el) = r#else {
                    buf.append(&mut el.check_for_unresolved_types());
                }
                buf
            }
            TupleElemAccess {
                prefix,
                resolved_type_of_parent,
                ..
            } => prefix
                .check_for_unresolved_types()
                .into_iter()
                .chain(
                    resolved_type_of_parent
                        .check_for_unresolved_types()
                        .into_iter(),
                )
                .collect(),
            EnumInstantiation {
                enum_decl,
                contents,
                ..
            } => {
                let mut buf = if let Some(contents) = contents {
                    contents.check_for_unresolved_types().into_iter().collect()
                } else {
                    vec![]
                };
                buf.append(
                    &mut enum_decl
                        .variants
                        .iter()
                        .flat_map(|x| x.r#type.check_for_unresolved_types())
                        .collect(),
                );
                buf
            }
            SizeOfValue { expr } => expr.check_for_unresolved_types(),
            AbiCast { address, .. } => address.check_for_unresolved_types(),
            // storage access can never be generic
            StorageAccess { .. } | Literal(_) | AbiName(_) | FunctionParameter => vec![],
        }
    }
}
impl UnresolvedTypeCheck for TypedAstNode {
    fn check_for_unresolved_types(&self) -> Vec<CompileError> {
        self.content.check_for_unresolved_types()
    }
}
impl UnresolvedTypeCheck for TypedDeclaration {
    // this is only run on entry nodes, which must have all well-formed types
    fn check_for_unresolved_types(&self) -> Vec<CompileError> {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(decl) => {
                let mut body = decl.body.check_for_unresolved_types();
                body.append(&mut decl.type_ascription.check_for_unresolved_types());
                body
            }
            FunctionDeclaration(decl) => {
                let mut body: Vec<CompileError> = decl
                    .body
                    .contents
                    .iter()
                    .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                    .collect();
                body.append(&mut decl.return_type.check_for_unresolved_types());
                body.append(
                    &mut decl
                        .type_parameters
                        .iter()
                        .map(|x| &x.type_id)
                        .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                        .collect(),
                );
                body
            }
            ConstantDeclaration(TypedConstantDeclaration { value, .. }) => {
                value.check_for_unresolved_types()
            }
            StorageReassignment(TypeCheckedStorageReassignment { fields, rhs, .. }) => fields
                .iter()
                .flat_map(|x| x.r#type.check_for_unresolved_types())
                .chain(rhs.check_for_unresolved_types().into_iter())
                .collect(),
            Reassignment(TypedReassignment { rhs, .. }) => rhs.check_for_unresolved_types(),
            ErrorRecovery
            | StorageDeclaration(_)
            | TraitDeclaration(_)
            | StructDeclaration(_)
            | EnumDeclaration(_)
            | ImplTrait { .. }
            | AbiDeclaration(_)
            | GenericTypeForFunctionScope { .. } => vec![],
        }
    }
}
