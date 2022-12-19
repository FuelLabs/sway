use crate::{
    language::ty::{self, TyExpression},
    Namespace,
};

// This analysis checks if an expression is known statically to evaluate
// to a non-zero value at runtime.
// It's intended to be used in the payability analysis to check if a non-payable
// method gets called with a non-zero amount of `coins`
pub fn possibly_nonzero_u64_expression(namespace: &Namespace, expr: &TyExpression) -> bool {
    use ty::TyExpressionVariant::*;
    match &expr.expression {
        Literal(crate::language::Literal::U64(value)) => *value != 0,
        // not a u64 literal, hence we return true to be on the safe side
        Literal(_) => true,
        VariableExpression { name, .. } => {
            match namespace.resolve_symbol(name).value {
                Some(ty_decl) => {
                    match ty_decl {
                        ty::TyDeclaration::VariableDeclaration(var_decl) => {
                            possibly_nonzero_u64_expression(namespace, &var_decl.body)
                        }
                        ty::TyDeclaration::ConstantDeclaration(decl_id) => {
                            match crate::declaration_engine::de_get_constant(
                                decl_id.clone(),
                                &expr.span,
                            ) {
                                Ok(const_decl) => {
                                    possibly_nonzero_u64_expression(namespace, &const_decl.value)
                                }
                                Err(_) => true,
                            }
                        }
                        _ => true, // impossible cases, true is a safer option here
                    }
                }
                None => {
                    // Unknown variable, but it's not possible in a well-typed expression
                    // returning true here just to be on the safe side
                    true
                }
            }
        }
        // We do not treat complex expressions at the moment: the rational for this
        // is that the `coins` contract call parameter is usually a literal, a variable,
        // or a constant.
        // Since we don't analyze the following types of expressions, we just assume
        // those result in non-zero amount of coins
        FunctionApplication { .. }
        | ArrayIndex { .. }
        | CodeBlock(_)
        | IfExp { .. }
        | AsmExpression { .. }
        | StructFieldAccess { .. }
        | TupleElemAccess { .. }
        | StorageAccess(_)
        | WhileLoop { .. } => true,
        // The following expression variants are unreachable, because of the type system
        // but we still consider these as non-zero to be on the safe side
        LazyOperator { .. }
        | Tuple { .. }
        | Array { .. }
        | StructExpression { .. }
        | FunctionParameter
        | EnumInstantiation { .. }
        | AbiCast { .. }
        | IntrinsicFunction(_)
        | AbiName(_)
        | UnsafeDowncast { .. }
        | EnumTag { .. }
        | Break
        | Continue
        | Reassignment(_)
        | Return(_)
        | StorageReassignment(_) => true,
    }
}
