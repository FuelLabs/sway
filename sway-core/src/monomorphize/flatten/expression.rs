use crate::{language::ty::*, monomorphize::priv_prelude::*, Engines};

pub(crate) fn find_from_exp<'a>(engines: Engines<'_>, exp: &'a TyExpression) -> Findings<'a> {
    find_from_exp_inner(engines, &exp.expression)
}

pub(crate) fn find_from_exp_inner<'a>(
    engines: Engines<'_>,
    exp: &'a TyExpressionVariant,
) -> Findings<'a> {
    use TyExpressionVariant::*;
    match exp {
        FunctionApplication {
            arguments,
            contract_call_params,
            ..
        } => arguments
            .iter()
            .map(|(_, arg)| find_from_exp(engines, arg))
            .chain(
                contract_call_params
                    .iter()
                    .map(|(_, arg)| find_from_exp(engines, arg)),
            )
            .collect(),
        LazyOperator { lhs, rhs, .. } => {
            find_from_exp(engines, lhs).add(find_from_exp(engines, rhs))
        }
        Tuple { fields } => fields
            .iter()
            .map(|field| find_from_exp(engines, field))
            .collect(),
        Array {
            contents,
            elem_type: _,
        } => contents
            .iter()
            .map(|content| find_from_exp(engines, content))
            .collect(),
        ArrayIndex { prefix, index } => {
            find_from_exp(engines, prefix).add(find_from_exp(engines, index))
        }
        StructExpression { .. } => todo!(),
        CodeBlock(block) => find_from_code_block(engines, block),
        IfExp { .. } => todo!(),
        MatchExp { .. } => todo!(),
        AsmExpression { .. } => todo!(),
        StructFieldAccess { .. } => todo!(),
        TupleElemAccess { prefix, .. } => find_from_exp(engines, prefix),
        EnumInstantiation { .. } => todo!(),
        AbiCast { .. } => todo!(),
        StorageAccess(_) => todo!(),
        IntrinsicFunction(_) => todo!(),
        AbiName(_) => todo!(),
        EnumTag { exp } => find_from_exp(engines, exp),
        UnsafeDowncast { .. } => todo!(),
        WhileLoop { .. } => todo!(),
        Reassignment(_) => todo!(),
        StorageReassignment(_) => todo!(),
        Return(exp) => find_from_exp(engines, exp),
        Literal(_) | Break | Continue | FunctionParameter | VariableExpression { .. } => {
            Findings::new()
        }
    }
}
