use crate::{
    declaration_engine::declaration_engine::de_get_constant,
    metadata::MetadataManager,
    semantic_analysis::{
        declaration::ProjectionKind, namespace, TyAstNode, TyAstNodeContent, TyConstantDeclaration,
        TyDeclaration, TyExpression, TyExpressionVariant, TyStructExpressionField,
    },
};

use super::{convert::convert_literal_to_constant, types::*};

use sway_error::error::CompileError;
use sway_ir::{
    constant::{Constant, ConstantValue},
    context::Context,
    module::Module,
    value::Value,
};
use sway_types::{ident::Ident, span::Spanned};

use std::collections::HashMap;

pub(crate) struct LookupEnv<'a> {
    pub(crate) context: &'a mut Context,
    pub(crate) md_mgr: &'a mut MetadataManager,
    pub(crate) module: Module,
    pub(crate) module_ns: Option<&'a namespace::Module>,
    pub(crate) lookup: fn(&mut LookupEnv, &Ident) -> Result<Option<Value>, CompileError>,
}

pub(crate) fn compile_const_decl(
    env: &mut LookupEnv,
    name: &Ident,
) -> Result<Option<Value>, CompileError> {
    // Check if it's a processed global constant.
    match (
        env.module.get_global_constant(env.context, name.as_str()),
        env.module_ns,
    ) {
        (Some(const_val), _) => Ok(Some(const_val)),
        (None, Some(module_ns)) => {
            // See if we it's a global const and whether we can compile it *now*.
            let decl = module_ns.check_symbol(name)?;
            let decl_name_value = match decl {
                TyDeclaration::ConstantDeclaration(decl_id) => {
                    let TyConstantDeclaration { name, value, .. } =
                        de_get_constant(decl_id.clone(), &name.span())?;
                    Some((name, value))
                }
                _otherwise => None,
            };
            if let Some((name, value)) = decl_name_value {
                let const_val = compile_constant_expression(
                    env.context,
                    env.md_mgr,
                    env.module,
                    env.module_ns,
                    &value,
                )?;
                env.module
                    .add_global_constant(env.context, name.as_str().to_owned(), const_val);
                Ok(Some(const_val))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

pub(super) fn compile_constant_expression(
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    module_ns: Option<&namespace::Module>,
    const_expr: &TyExpression,
) -> Result<Value, CompileError> {
    let span_id_idx = md_mgr.span_to_md(context, &const_expr.span);

    let constant_evaluated =
        compile_constant_expression_to_constant(context, md_mgr, module, module_ns, const_expr)?;
    Ok(Value::new_constant(context, constant_evaluated).add_metadatum(context, span_id_idx))
}

pub(crate) fn compile_constant_expression_to_constant(
    context: &mut Context,
    md_mgr: &mut MetadataManager,
    module: Module,
    module_ns: Option<&namespace::Module>,
    const_expr: &TyExpression,
) -> Result<Constant, CompileError> {
    let lookup = &mut LookupEnv {
        context,
        md_mgr,
        module,
        module_ns,
        lookup: compile_const_decl,
    };

    let err = match &const_expr.expression {
        // Special case functions because the span in `const_expr` is to the inlined function
        // definition, rather than the actual call site.
        TyExpressionVariant::FunctionApplication { call_path, .. } => {
            Err(CompileError::NonConstantDeclValue {
                span: call_path.span(),
            })
        }
        _otherwise => Err(CompileError::NonConstantDeclValue {
            span: const_expr.span.clone(),
        }),
    };
    let mut known_consts = MappedStack::<Ident, Constant>::new();

    const_eval_typed_expr(lookup, &mut known_consts, const_expr).map_or(err, Ok)
}

// A HashMap that can hold multiple values and
// fetch values in a LIFO manner. Rust's MultiMap
// handles values in a FIFO manner.
struct MappedStack<K: std::cmp::Eq + std::hash::Hash, V> {
    container: HashMap<K, Vec<V>>,
}

impl<K: std::cmp::Eq + std::hash::Hash, V> MappedStack<K, V> {
    fn new() -> MappedStack<K, V> {
        MappedStack {
            container: HashMap::<K, Vec<V>>::new(),
        }
    }
    fn push(&mut self, k: K, v: V) {
        match self.container.get_mut(&k) {
            Some(val_vec) => {
                val_vec.push(v);
            }
            None => {
                self.container.insert(k, vec![v]);
            }
        }
    }
    fn get(&self, k: &K) -> Option<&V> {
        self.container.get(k).and_then(|val_vec| val_vec.last())
    }
    fn pop(&mut self, k: &K) {
        if let Some(val_vec) = self.container.get_mut(k) {
            val_vec.pop();
            if val_vec.is_empty() {
                self.container.remove(k);
            }
        }
    }
}

impl<K: std::cmp::Eq + std::hash::Hash, V> Default for MappedStack<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

/// Given an environment mapping names to constants,
/// attempt to evaluate a typed expression to a constant.
fn const_eval_typed_expr(
    lookup: &mut LookupEnv,
    known_consts: &mut MappedStack<Ident, Constant>,
    expr: &TyExpression,
) -> Option<Constant> {
    match &expr.expression {
        TyExpressionVariant::Literal(l) => Some(convert_literal_to_constant(l)),
        TyExpressionVariant::FunctionApplication {
            arguments,
            function_decl,
            ..
        } => {
            let actuals_const = arguments
                .iter()
                .filter_map(|(name, sub_expr)| {
                    const_eval_typed_expr(lookup, known_consts, sub_expr)
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
            if function_decl.body.contents.len() > 1 {
                return None;
            }
            let res =
                function_decl.body.contents.last().and_then(|first_expr| {
                    const_eval_typed_ast_node(lookup, known_consts, first_expr)
                });
            for (name, _) in arguments {
                known_consts.pop(name);
            }
            res
        }
        TyExpressionVariant::VariableExpression { name, .. } => match known_consts.get(name) {
            // 1. Check if name is in known_consts.
            Some(cvs) => Some(cvs.clone()),
            None => {
                // 2. Check if name is a global constant.
                (lookup.lookup)(lookup, name)
                    .ok()
                    .flatten()
                    .and_then(|v| v.get_constant(lookup.context).cloned())
            }
        },
        TyExpressionVariant::StructExpression { fields, .. } => {
            let (field_typs, field_vals): (Vec<_>, Vec<_>) = fields
                .iter()
                .filter_map(|TyStructExpressionField { name: _, value, .. }| {
                    const_eval_typed_expr(lookup, known_consts, value)
                        .map(|cv| (value.return_type, cv))
                })
                .unzip();
            if field_vals.len() < fields.len() {
                // We couldn't evaluate all fields to a constant.
                return None;
            }
            let aggregate = get_aggregate_for_types(lookup.context, &field_typs).unwrap();
            Some(Constant::new_struct(&aggregate, field_vals))
        }
        TyExpressionVariant::Tuple { fields } => {
            let (field_typs, field_vals): (Vec<_>, Vec<_>) = fields
                .iter()
                .filter_map(|value| {
                    const_eval_typed_expr(lookup, known_consts, value)
                        .map(|cv| (value.return_type, cv))
                })
                .unzip();
            if field_vals.len() < fields.len() {
                // We couldn't evaluate all fields to a constant.
                return None;
            }
            let aggregate = create_tuple_aggregate(lookup.context, field_typs).unwrap();
            Some(Constant::new_struct(&aggregate, field_vals))
        }
        TyExpressionVariant::Array { contents } => {
            let (element_typs, element_vals): (Vec<_>, Vec<_>) = contents
                .iter()
                .filter_map(|value| {
                    const_eval_typed_expr(lookup, known_consts, value)
                        .map(|cv| (value.return_type, cv))
                })
                .unzip();
            if element_vals.len() < contents.len() || element_typs.is_empty() {
                // We couldn't evaluate all fields to a constant or cannot determine element type.
                return None;
            }
            let mut element_iter = element_typs.iter();
            let element_type_id = *element_iter.next().unwrap();
            if !element_iter.all(|tid| {
                crate::type_system::look_up_type_id(*tid)
                    == crate::type_system::look_up_type_id(element_type_id)
            }) {
                // This shouldn't happen if the type checker did its job.
                return None;
            }
            let aggregate = create_array_aggregate(
                lookup.context,
                element_type_id,
                element_typs.len().try_into().unwrap(),
            )
            .unwrap();
            Some(Constant::new_array(&aggregate, element_vals))
        }
        TyExpressionVariant::EnumInstantiation {
            enum_decl,
            tag,
            contents,
            ..
        } => {
            let aggregate =
                create_enum_aggregate(lookup.context, enum_decl.variants.clone()).unwrap();
            let tag_value = Constant::new_uint(64, *tag as u64);
            let mut fields: Vec<Constant> = vec![tag_value];
            match contents {
                None => fields.push(Constant::new_unit()),
                Some(subexpr) => const_eval_typed_expr(lookup, known_consts, subexpr)
                    .into_iter()
                    .for_each(|enum_val| {
                        fields.push(enum_val);
                    }),
            }
            Some(Constant::new_struct(&aggregate, fields))
        }
        TyExpressionVariant::StructFieldAccess {
            prefix,
            field_to_access,
            resolved_type_of_parent,
            ..
        } => match const_eval_typed_expr(lookup, known_consts, prefix) {
            Some(Constant {
                value: ConstantValue::Struct(fields),
                ..
            }) => {
                let field_kind = ProjectionKind::StructField {
                    name: field_to_access.name.clone(),
                };
                get_struct_name_field_index_and_type(*resolved_type_of_parent, field_kind)
                    .and_then(|(_struct_name, field_idx_and_type_opt)| {
                        field_idx_and_type_opt.map(|(field_idx, _field_type)| field_idx)
                    })
                    .and_then(|field_idx| fields.get(field_idx as usize).cloned())
            }
            _ => None,
        },
        TyExpressionVariant::TupleElemAccess {
            prefix,
            elem_to_access_num,
            ..
        } => match const_eval_typed_expr(lookup, known_consts, prefix) {
            Some(Constant {
                value: ConstantValue::Struct(fields),
                ..
            }) => fields.get(*elem_to_access_num).cloned(),
            _ => None,
        },
        TyExpressionVariant::Return(stmt) => {
            const_eval_typed_expr(lookup, known_consts, &stmt.expr)
        }
        TyExpressionVariant::ArrayIndex { .. }
        | TyExpressionVariant::IntrinsicFunction(_)
        | TyExpressionVariant::CodeBlock(_)
        | TyExpressionVariant::Reassignment(_)
        | TyExpressionVariant::StorageReassignment(_)
        | TyExpressionVariant::FunctionParameter
        | TyExpressionVariant::IfExp { .. }
        | TyExpressionVariant::AsmExpression { .. }
        | TyExpressionVariant::LazyOperator { .. }
        | TyExpressionVariant::AbiCast { .. }
        | TyExpressionVariant::StorageAccess(_)
        | TyExpressionVariant::AbiName(_)
        | TyExpressionVariant::EnumTag { .. }
        | TyExpressionVariant::UnsafeDowncast { .. }
        | TyExpressionVariant::Break
        | TyExpressionVariant::Continue
        | TyExpressionVariant::WhileLoop { .. } => None,
    }
}

fn const_eval_typed_ast_node(
    lookup: &mut LookupEnv,
    known_consts: &mut MappedStack<Ident, Constant>,
    expr: &TyAstNode,
) -> Option<Constant> {
    match &expr.content {
        TyAstNodeContent::Declaration(_) => {
            // TODO: add the binding to known_consts (if it's a const) and proceed.
            None
        }
        TyAstNodeContent::Expression(e) | TyAstNodeContent::ImplicitReturnExpression(e) => {
            const_eval_typed_expr(lookup, known_consts, e)
        }
        TyAstNodeContent::SideEffect => None,
    }
}
