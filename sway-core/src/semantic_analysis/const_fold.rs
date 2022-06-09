use super::{
    TypedAstNode, TypedAstNodeContent, TypedExpression, TypedExpressionVariant,
    TypedStructExpressionField,
};

use std::collections::HashMap;
use sway_ir::{constant::Constant, context::Context, module::Module};
use sway_types::ident::Ident;

// A HashMap that can hold multiple values and
// fetch values in a LIFO manner. Rust's MultiMap
// handles values in a FIFO manner.
pub struct MappedStack<K: std::cmp::Eq + std::hash::Hash, V> {
    container: HashMap<K, Vec<V>>,
}

impl<K: std::cmp::Eq + std::hash::Hash, V> MappedStack<K, V> {
    pub fn new() -> MappedStack<K, V> {
        MappedStack {
            container: HashMap::<K, Vec<V>>::new(),
        }
    }
    pub fn push(&mut self, k: K, v: V) {
        match self.container.get_mut(&k) {
            Some(val_vec) => {
                val_vec.push(v);
            }
            None => {
                self.container.insert(k, vec![v]);
            }
        }
    }
    pub fn get(&self, k: &K) -> Option<&V> {
        self.container.get(k).and_then(|val_vec| val_vec.last())
    }
    pub fn pop(&mut self, k: &K) {
        match self.container.get_mut(k) {
            Some(val_vec) => {
                val_vec.pop();
                if val_vec.is_empty() {
                    self.container.remove(k);
                }
            }
            None => {}
        }
    }
}

/// Given an environment mapping names to constants,
/// attempt to evaluate a typed expression to a constant.
pub fn const_fold_typed_expr(
    context: &mut Context,
    module: Module,
    known_consts: &mut MappedStack<Ident, Constant>,
    expr: &TypedExpression,
) -> Option<Constant> {
    match &expr.expression {
        TypedExpressionVariant::Literal(l) => Some(crate::optimize::convert_literal_to_constant(l)),
        TypedExpressionVariant::FunctionApplication {
            arguments,
            function_body,
            ..
        } => {
            let actuals_const = arguments
                .iter()
                .filter_map(|(name, sub_expr)| {
                    const_fold_typed_expr(context, module, known_consts, sub_expr)
                        .map(|sub_const| (name, sub_const))
                })
                .collect::<Vec<_>>();
            // If all actual arguments don't evaluate a constant, bail out.
            // TODO: Explore if we could continue here and if it'll be useful.
            if actuals_const.len() < arguments.len() {
                return None;
            }
            for (name, cval) in actuals_const.into_iter() {
                known_consts.push(name.clone(), cval);
            }

            // TODO: Handle more than one statement in the block.
            if function_body.contents.len() > 1 {
                return None;
            }
            let res = function_body.contents.last().and_then(|first_expr| {
                const_fold_typed_ast_node(context, module, known_consts, &first_expr)
            });
            for (name, _) in arguments {
                known_consts.pop(name);
            }
            return res;
        }
        TypedExpressionVariant::VariableExpression { name } => match known_consts.get(name) {
            // 1. Check if name is in known_consts.
            Some(cvs) => Some(cvs.clone()),
            None => {
                // 2. Check if name is a global constant.
                use sway_ir::value::ValueDatum::Constant;
                module
                    .get_global_constant(context, name.as_str())
                    .and_then(|v| match &context.values[(v.0)].value {
                        Constant(cv) => Some(cv.clone()),
                        _ => None,
                    })
            }
        },
        TypedExpressionVariant::StructExpression { fields, .. } => {
            let (field_typs, field_vals): (Vec<_>, Vec<_>) = fields
                .iter()
                .filter_map(|TypedStructExpressionField { name: _, value }| {
                    const_fold_typed_expr(context, module, known_consts, &value)
                        .and_then(|cv| Some((value.return_type, cv)))
                })
                .unzip();
            if field_vals.len() < fields.len() {
                // We couldn't evaluate all fields to a constant.
                return None;
            }
            let aggregate = crate::optimize::get_aggregate_for_types(context, &field_typs).unwrap();
            Some(Constant::new_struct(&aggregate, field_vals))
        }
        TypedExpressionVariant::Tuple { .. }
        | TypedExpressionVariant::Array { .. }
        | TypedExpressionVariant::ArrayIndex { .. }
        | TypedExpressionVariant::CodeBlock(_)
        | TypedExpressionVariant::FunctionParameter
        | TypedExpressionVariant::IfExp { .. }
        | TypedExpressionVariant::AsmExpression { .. }
        | TypedExpressionVariant::StructFieldAccess { .. }
        | TypedExpressionVariant::TupleElemAccess { .. }
        | TypedExpressionVariant::EnumInstantiation { .. }
        | TypedExpressionVariant::LazyOperator { .. }
        | TypedExpressionVariant::AbiCast { .. }
        | TypedExpressionVariant::StorageAccess(_)
        | TypedExpressionVariant::TypeProperty { .. }
        | TypedExpressionVariant::GetStorageKey { .. }
        | TypedExpressionVariant::SizeOfValue { .. }
        | TypedExpressionVariant::AbiName(_)
        | TypedExpressionVariant::EnumTag { .. }
        | TypedExpressionVariant::UnsafeDowncast { .. } => None,
    }
}

fn const_fold_typed_ast_node(
    context: &mut Context,
    module: Module,
    known_consts: &mut MappedStack<Ident, Constant>,
    expr: &TypedAstNode,
) -> Option<Constant> {
    match &expr.content {
        TypedAstNodeContent::ReturnStatement(trs) => {
            const_fold_typed_expr(context, module, known_consts, &trs.expr)
        }
        TypedAstNodeContent::Declaration(_) => {
            // TODO: add the binding to known_consts (if it's a const) and proceed.
            None
        }
        TypedAstNodeContent::Expression(e) | TypedAstNodeContent::ImplicitReturnExpression(e) => {
            const_fold_typed_expr(context, module, known_consts, &e)
        }
        TypedAstNodeContent::WhileLoop(_) | TypedAstNodeContent::SideEffect => None,
    }
}
