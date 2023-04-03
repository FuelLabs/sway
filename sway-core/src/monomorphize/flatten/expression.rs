use crate::{language::ty::*, monomorphize::priv_prelude::*, type_system::*, Engines};

pub(crate) fn flatten_exp(engines: Engines<'_>, exp: TyExpression) -> TyExpression {
    flatten_exp_inner(engines, &mut exp.expression, exp.return_type)
}

pub(crate) fn flatten_exp_inner(
    engines: Engines<'_>,
    exp: TyExpressionVariant,
    _return_type: TypeId,
) -> TyExpressionVariant {
    use TyExpressionVariant::*;
    // NOTE: todo here
    match exp {
        FunctionApplication {
            arguments,
            contract_call_params,
            ..
        } => {
            arguments
                .iter()
                .for_each(|(_, arg)| flatten_exp(engines, arg));
            contract_call_params
                .iter()
                .for_each(|(_, arg)| flatten_exp(engines, arg));
            // NOTE: todo here
        }
        LazyOperator { lhs, rhs, .. } => {
            flatten_exp(engines, lhs);
            flatten_exp(engines, rhs);
        }
        VariableExpression { .. } => {
            // NOTE: may need to do something here later
        }
        Tuple { fields } => {
            fields.iter().for_each(|field| flatten_exp(engines, field));
        }
        Array { contents: _ } => {
            todo!();
            // contents
            //     .iter()
            //     .for_each(|elem| flatten_exp(engines, elem));
        }
        ArrayIndex {
            prefix: _,
            index: _,
        } => {
            todo!();
            // flatten_exp(engines, prefix);
            // flatten_exp(engines, index);
        }
        StructExpression { .. } => todo!(),
        CodeBlock(block) => {
            flatten_code_block(engines, block);
        }
        IfExp { .. } => todo!(),
        MatchExp { .. } => todo!(),
        AsmExpression { .. } => todo!(),
        StructFieldAccess { .. } => todo!(),
        TupleElemAccess { prefix, .. } => {
            flatten_exp(engines, prefix);
        }
        EnumInstantiation { .. } => todo!(),
        AbiCast { .. } => todo!(),
        StorageAccess(_) => todo!(),
        IntrinsicFunction(_) => todo!(),
        AbiName(_) => todo!(),
        EnumTag { exp } => {
            flatten_exp(engines, exp);
        }
        UnsafeDowncast { .. } => todo!(),
        WhileLoop { .. } => todo!(),
        Reassignment(_) => todo!(),
        StorageReassignment(_) => todo!(),
        Return(exp) => {
            flatten_exp(engines, exp);
        }
        Literal(_) => {}
        Break => {}
        Continue => {}
        FunctionParameter => {}
    }
}
