use sway_error::handler::Handler;

use crate::language::ty;

use super::TypeCheckContext;

// This analysis checks if an expression is known statically to evaluate
// to a non-zero value at runtime.
// It's intended to be used in the payability analysis to check if a non-payable
// method gets called with a non-zero amount of `coins`
pub fn possibly_nonzero_u64_expression(ctx: &TypeCheckContext, expr: &ty::TyExpression) -> bool {
    use ty::TyExpressionVariant::*;
    match &expr.expression {
        Literal(crate::language::Literal::U64(value)) => *value != 0,
        Literal(crate::language::Literal::Numeric(value)) => *value != 0,
        // not a u64 literal, hence we return true to be on the safe side
        Literal(_) => true,
        ConstantExpression { decl, .. } => match &decl.value {
            Some(expr) => possibly_nonzero_u64_expression(ctx, expr),
            None => false,
        },
        ConfigurableExpression { decl, .. } => match &decl.value {
            Some(expr) => possibly_nonzero_u64_expression(ctx, expr),
            None => false,
        },
        ConstGenericExpression { decl, .. } => {
            match decl.value.as_ref() {
                Some(expr) => possibly_nonzero_u64_expression(ctx, expr),
                None => true,
            }
        }
        VariableExpression { name, .. } => {
            match ctx.resolve_symbol(&Handler::default(), name).ok() {
                Some(ty_decl) => {
                    match ty_decl {
                        ty::TyDecl::VariableDecl(var_decl) => {
                            possibly_nonzero_u64_expression(ctx, &var_decl.body)
                        }
                        ty::TyDecl::ConstantDecl(ty::ConstantDecl { decl_id, .. }) => {
                            let const_decl = ctx.engines.de().get_constant(&decl_id);
                            match &const_decl.value {
                                Some(value) => possibly_nonzero_u64_expression(ctx, value),
                                None => true,
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
        | MatchExp { .. }
        | IfExp { .. }
        | AsmExpression { .. }
        | StructFieldAccess { .. }
        | TupleElemAccess { .. }
        | StorageAccess(_)
        | WhileLoop { .. }
        | ForLoop { .. } => true,
        // The following expression variants are unreachable, because of the type system
        // but we still consider these as non-zero to be on the safe side
        LazyOperator { .. }
        | Tuple { .. }
        | ArrayExplicit { .. }
        | ArrayRepeat { .. }
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
        | ImplicitReturn(_)
        | Return(_)
        | Panic(_)
        | Ref(_)
        | Deref(_) => true,
    }
}
