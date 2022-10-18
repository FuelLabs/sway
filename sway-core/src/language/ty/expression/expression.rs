use std::fmt;

use sway_types::Span;

use crate::{
    declaration_engine::de_get_function,
    error::*,
    language::{ty::*, Literal},
    type_system::*,
    types::DeterministicallyAborts,
};

#[derive(Clone, Debug, Eq)]
pub struct TyExpression {
    pub expression: TyExpressionVariant,
    pub return_type: TypeId,
    pub span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TyExpression {
    fn eq(&self, other: &Self) -> bool {
        self.expression == other.expression
            && look_up_type_id(self.return_type) == look_up_type_id(other.return_type)
    }
}

impl CopyTypes for TyExpression {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.return_type.copy_types(type_mapping);
        self.expression.copy_types(type_mapping);
    }
}

impl fmt::Display for TyExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({})",
            self.expression,
            look_up_type_id(self.return_type)
        )
    }
}

impl CollectTypesMetadata for TyExpression {
    fn collect_types_metadata(&self) -> CompileResult<Vec<TypeMetadata>> {
        use TyExpressionVariant::*;
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut res = check!(
            self.return_type.collect_types_metadata(),
            return err(warnings, errors),
            warnings,
            errors
        );
        match &self.expression {
            FunctionApplication {
                arguments,
                function_decl_id,
                ..
            } => {
                for arg in arguments.iter() {
                    res.append(&mut check!(
                        arg.1.collect_types_metadata(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
                let function_decl = match de_get_function(function_decl_id.clone(), &self.span) {
                    Ok(decl) => decl,
                    Err(e) => return err(vec![], vec![e]),
                };
                for content in function_decl.body.contents.iter() {
                    res.append(&mut check!(
                        content.collect_types_metadata(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
            }
            Tuple { fields } => {
                for field in fields.iter() {
                    res.append(&mut check!(
                        field.collect_types_metadata(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
            }
            AsmExpression { registers, .. } => {
                for register in registers.iter() {
                    if let Some(init) = register.initializer.as_ref() {
                        res.append(&mut check!(
                            init.collect_types_metadata(),
                            return err(warnings, errors),
                            warnings,
                            errors
                        ));
                    }
                }
            }
            StructExpression { fields, .. } => {
                for field in fields.iter() {
                    res.append(&mut check!(
                        field.value.collect_types_metadata(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
            }
            LazyOperator { lhs, rhs, .. } => {
                res.append(&mut check!(
                    lhs.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
                res.append(&mut check!(
                    rhs.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
            }
            Array { contents } => {
                for content in contents.iter() {
                    res.append(&mut check!(
                        content.collect_types_metadata(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
            }
            ArrayIndex { prefix, index } => {
                res.append(&mut check!(
                    (**prefix).collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
                res.append(&mut check!(
                    (**index).collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
            }
            CodeBlock(block) => {
                for content in block.contents.iter() {
                    res.append(&mut check!(
                        content.collect_types_metadata(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
            }
            IfExp {
                condition,
                then,
                r#else,
            } => {
                res.append(&mut check!(
                    condition.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
                res.append(&mut check!(
                    then.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
                if let Some(r#else) = r#else {
                    res.append(&mut check!(
                        r#else.collect_types_metadata(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
            }
            StructFieldAccess {
                prefix,
                resolved_type_of_parent,
                ..
            } => {
                res.append(&mut check!(
                    prefix.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
                res.append(&mut check!(
                    resolved_type_of_parent.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
            }
            TupleElemAccess {
                prefix,
                resolved_type_of_parent,
                ..
            } => {
                res.append(&mut check!(
                    prefix.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
                res.append(&mut check!(
                    resolved_type_of_parent.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
            }
            EnumInstantiation {
                enum_decl,
                contents,
                ..
            } => {
                if let Some(contents) = contents {
                    res.append(&mut check!(
                        contents.collect_types_metadata(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
                for variant in enum_decl.variants.iter() {
                    res.append(&mut check!(
                        variant.type_id.collect_types_metadata(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
                for type_param in enum_decl.type_parameters.iter() {
                    res.append(&mut check!(
                        type_param.type_id.collect_types_metadata(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
            }
            AbiCast { address, .. } => {
                res.append(&mut check!(
                    address.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
            }
            IntrinsicFunction(kind) => {
                res.append(&mut check!(
                    kind.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
            }
            EnumTag { exp } => {
                res.append(&mut check!(
                    exp.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
            }
            UnsafeDowncast { exp, variant } => {
                res.append(&mut check!(
                    exp.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
                res.append(&mut check!(
                    variant.type_id.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
            }
            WhileLoop { condition, body } => {
                res.append(&mut check!(
                    condition.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
                for content in body.contents.iter() {
                    res.append(&mut check!(
                        content.collect_types_metadata(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
            }
            Return(exp) => res.append(&mut check!(
                exp.collect_types_metadata(),
                return err(warnings, errors),
                warnings,
                errors
            )),
            // storage access can never be generic
            // variable expressions don't ever have return types themselves, they're stored in
            // `TyExpression::return_type`. Variable expressions are just names of variables.
            VariableExpression { .. }
            | StorageAccess { .. }
            | Literal(_)
            | AbiName(_)
            | Break
            | Continue
            | FunctionParameter => {}
            Reassignment(reassignment) => {
                res.append(&mut check!(
                    reassignment.rhs.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
            }
            StorageReassignment(storage_reassignment) => {
                for field in storage_reassignment.fields.iter() {
                    res.append(&mut check!(
                        field.type_id.collect_types_metadata(),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }
                res.append(&mut check!(
                    storage_reassignment.rhs.collect_types_metadata(),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
            }
        }
        ok(res, warnings, errors)
    }
}

impl DeterministicallyAborts for TyExpression {
    fn deterministically_aborts(&self) -> bool {
        use TyExpressionVariant::*;
        match &self.expression {
            FunctionApplication {
                function_decl_id,
                arguments,
                ..
            } => {
                let function_decl = match de_get_function(function_decl_id.clone(), &self.span) {
                    Ok(decl) => decl,
                    Err(_e) => panic!("failed to get function"),
                };
                function_decl.body.deterministically_aborts()
                    || arguments.iter().any(|(_, x)| x.deterministically_aborts())
            }
            Tuple { fields, .. } => fields.iter().any(|x| x.deterministically_aborts()),
            Array { contents, .. } => contents.iter().any(|x| x.deterministically_aborts()),
            CodeBlock(contents) => contents.deterministically_aborts(),
            LazyOperator { lhs, .. } => lhs.deterministically_aborts(),
            StructExpression { fields, .. } => {
                fields.iter().any(|x| x.value.deterministically_aborts())
            }
            EnumInstantiation { contents, .. } => contents
                .as_ref()
                .map(|x| x.deterministically_aborts())
                .unwrap_or(false),
            AbiCast { address, .. } => address.deterministically_aborts(),
            StructFieldAccess { .. }
            | Literal(_)
            | StorageAccess { .. }
            | VariableExpression { .. }
            | FunctionParameter
            | TupleElemAccess { .. } => false,
            IntrinsicFunction(kind) => kind.deterministically_aborts(),
            ArrayIndex { prefix, index } => {
                prefix.deterministically_aborts() || index.deterministically_aborts()
            }
            AsmExpression { registers, .. } => registers.iter().any(|x| {
                x.initializer
                    .as_ref()
                    .map(|x| x.deterministically_aborts())
                    .unwrap_or(false)
            }),
            IfExp {
                condition,
                then,
                r#else,
                ..
            } => {
                condition.deterministically_aborts()
                    || (then.deterministically_aborts()
                        && r#else
                            .as_ref()
                            .map(|x| x.deterministically_aborts())
                            .unwrap_or(false))
            }
            AbiName(_) => false,
            EnumTag { exp } => exp.deterministically_aborts(),
            UnsafeDowncast { exp, .. } => exp.deterministically_aborts(),
            WhileLoop { condition, body } => {
                condition.deterministically_aborts() || body.deterministically_aborts()
            }
            Break => false,
            Continue => false,
            Reassignment(reassignment) => reassignment.rhs.deterministically_aborts(),
            StorageReassignment(storage_reassignment) => {
                storage_reassignment.rhs.deterministically_aborts()
            }
            // TODO: Is this correct?
            // I'm not sure what this function is supposed to do exactly. It's called
            // "deterministically_aborts" which I thought meant it checks for an abort/panic, but
            // it's actually checking for returns.
            //
            // Also, is it necessary to check the expression to see if avoids the return? eg.
            // someone could write `return break;` in a loop, which would mean the return never
            // gets executed.
            Return(..) => true,
        }
    }
}

impl TyExpression {
    pub(crate) fn error(span: Span) -> TyExpression {
        TyExpression {
            expression: TyExpressionVariant::Tuple { fields: vec![] },
            return_type: insert_type(TypeInfo::ErrorRecovery),
            span,
        }
    }

    /// recurse into `self` and get any return statements -- used to validate that all returns
    /// do indeed return the correct type
    /// This does _not_ extract implicit return statements as those are not control flow! This is
    /// _only_ for explicit returns.
    pub(crate) fn gather_return_statements(&self) -> Vec<&TyExpression> {
        self.expression.gather_return_statements()
    }

    /// gathers the mutability of the expressions within
    pub(crate) fn gather_mutability(&self) -> VariableMutability {
        match &self.expression {
            TyExpressionVariant::VariableExpression { mutability, .. } => *mutability,
            _ => VariableMutability::Immutable,
        }
    }

    /// Returns `self` as a literal, if possible.
    pub(crate) fn extract_literal_value(&self) -> Option<Literal> {
        self.expression.extract_literal_value()
    }
}
