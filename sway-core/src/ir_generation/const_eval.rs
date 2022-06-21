use crate::{
    asm_generation::from_ir::ir_type_size_in_bytes,
    error::CompileError,
    semantic_analysis::{
        declaration::ProjectionKind, TypedAstNode, TypedAstNodeContent, TypedExpression,
        TypedExpressionVariant, TypedStructExpressionField,
    },
};

use super::{
    convert::{add_to_b256, convert_literal_to_constant, get_storage_key},
    types::*,
};

use sway_ir::{
    constant::{Constant, ConstantValue},
    context::Context,
    irtype::{AggregateContent, Type},
    metadata::MetadataIndex,
    module::Module,
    value::Value,
};

use fuel_types::{Bytes32, Bytes8};
use sway_types::{state::StateIndex, JsonStorageInitializers, StorageInitializer};

use sway_types::{ident::Ident, span::Spanned};

use std::collections::HashMap;

pub(super) fn compile_constant_expression(
    context: &mut Context,
    module: Module,
    const_expr: &TypedExpression,
) -> Result<Value, CompileError> {
    let span_id_idx = MetadataIndex::from_span(context, &const_expr.span);

    let constant_evaluated = compile_constant_expression_to_constant(context, module, const_expr)?;
    Ok(Value::new_constant(
        context,
        constant_evaluated,
        span_id_idx,
    ))
}

pub(crate) fn compile_constant_expression_to_constant(
    context: &mut Context,
    module: Module,
    const_expr: &TypedExpression,
) -> Result<Constant, CompileError> {
    let err = match &const_expr.expression {
        // Special case functions because the span in `const_expr` is to the inlined function
        // definition, rather than the actual call site.
        TypedExpressionVariant::FunctionApplication { call_path, .. } => {
            Err(CompileError::NonConstantDeclValue {
                span: call_path.span(),
            })
        }
        _otherwise => Err(CompileError::NonConstantDeclValue {
            span: const_expr.span.clone(),
        }),
    };
    let mut known_consts = MappedStack::<Ident, Constant>::new();

    const_eval_typed_expr(context, module, &mut known_consts, const_expr).map_or(err, Ok)
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

impl<K: std::cmp::Eq + std::hash::Hash, V> Default for MappedStack<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

/// Given an environment mapping names to constants,
/// attempt to evaluate a typed expression to a constant.
fn const_eval_typed_expr(
    context: &mut Context,
    module: Module,
    known_consts: &mut MappedStack<Ident, Constant>,
    expr: &TypedExpression,
) -> Option<Constant> {
    match &expr.expression {
        TypedExpressionVariant::Literal(l) => Some(convert_literal_to_constant(l)),
        TypedExpressionVariant::FunctionApplication {
            arguments,
            function_body,
            ..
        } => {
            let actuals_const = arguments
                .iter()
                .filter_map(|(name, sub_expr)| {
                    const_eval_typed_expr(context, module, known_consts, sub_expr)
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
                const_eval_typed_ast_node(context, module, known_consts, first_expr)
            });
            for (name, _) in arguments {
                known_consts.pop(name);
            }
            res
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
                .filter_map(|TypedStructExpressionField { name: _, value, .. }| {
                    const_eval_typed_expr(context, module, known_consts, value)
                        .map(|cv| (value.return_type, cv))
                })
                .unzip();
            if field_vals.len() < fields.len() {
                // We couldn't evaluate all fields to a constant.
                return None;
            }
            let aggregate = get_aggregate_for_types(context, &field_typs).unwrap();
            Some(Constant::new_struct(&aggregate, field_vals))
        }
        TypedExpressionVariant::Tuple { fields } => {
            let (field_typs, field_vals): (Vec<_>, Vec<_>) = fields
                .iter()
                .filter_map(|value| {
                    const_eval_typed_expr(context, module, known_consts, value)
                        .map(|cv| (value.return_type, cv))
                })
                .unzip();
            if field_vals.len() < fields.len() {
                // We couldn't evaluate all fields to a constant.
                return None;
            }
            let aggregate = create_tuple_aggregate(context, field_typs).unwrap();
            Some(Constant::new_struct(&aggregate, field_vals))
        }
        TypedExpressionVariant::Array { contents } => {
            let (element_typs, element_vals): (Vec<_>, Vec<_>) = contents
                .iter()
                .filter_map(|value| {
                    const_eval_typed_expr(context, module, known_consts, value)
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
                crate::type_engine::look_up_type_id(*tid)
                    == crate::type_engine::look_up_type_id(element_type_id)
            }) {
                // This shouldn't happen if the type checker did its job.
                return None;
            }
            let aggregate = create_array_aggregate(
                context,
                element_type_id,
                element_typs.len().try_into().unwrap(),
            )
            .unwrap();
            Some(Constant::new_array(&aggregate, element_vals))
        }
        TypedExpressionVariant::EnumInstantiation {
            enum_decl,
            tag,
            contents,
            ..
        } => {
            let aggregate = create_enum_aggregate(context, enum_decl.variants.clone()).unwrap();
            let tag_value = Constant::new_uint(64, *tag as u64);
            let mut fields: Vec<Constant> = vec![tag_value];
            contents.iter().for_each(|subexpr| {
                const_eval_typed_expr(context, module, known_consts, &*subexpr)
                    .into_iter()
                    .for_each(|enum_val| {
                        fields.push(enum_val);
                    })
            });
            Some(Constant::new_struct(&aggregate, fields))
        }
        TypedExpressionVariant::StructFieldAccess {
            prefix,
            field_to_access,
            resolved_type_of_parent,
        } => match const_eval_typed_expr(context, module, known_consts, &*prefix) {
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
        TypedExpressionVariant::TupleElemAccess {
            prefix,
            elem_to_access_num,
            ..
        } => match const_eval_typed_expr(context, module, known_consts, &*prefix) {
            Some(Constant {
                value: ConstantValue::Struct(fields),
                ..
            }) => fields.get(*elem_to_access_num).cloned(),
            _ => None,
        },
        TypedExpressionVariant::ArrayIndex { .. }
        | TypedExpressionVariant::IntrinsicFunction(_)
        | TypedExpressionVariant::CodeBlock(_)
        | TypedExpressionVariant::FunctionParameter
        | TypedExpressionVariant::IfExp { .. }
        | TypedExpressionVariant::AsmExpression { .. }
        | TypedExpressionVariant::LazyOperator { .. }
        | TypedExpressionVariant::AbiCast { .. }
        | TypedExpressionVariant::StorageAccess(_)
        | TypedExpressionVariant::AbiName(_)
        | TypedExpressionVariant::EnumTag { .. }
        | TypedExpressionVariant::UnsafeDowncast { .. } => None,
    }
}

fn const_eval_typed_ast_node(
    context: &mut Context,
    module: Module,
    known_consts: &mut MappedStack<Ident, Constant>,
    expr: &TypedAstNode,
) -> Option<Constant> {
    match &expr.content {
        TypedAstNodeContent::ReturnStatement(trs) => {
            const_eval_typed_expr(context, module, known_consts, &trs.expr)
        }
        TypedAstNodeContent::Declaration(_) => {
            // TODO: add the binding to known_consts (if it's a const) and proceed.
            None
        }
        TypedAstNodeContent::Expression(e) | TypedAstNodeContent::ImplicitReturnExpression(e) => {
            const_eval_typed_expr(context, module, known_consts, e)
        }
        TypedAstNodeContent::WhileLoop(_) | TypedAstNodeContent::SideEffect => None,
    }
}

pub fn serialize_to_storage_slots(
    constant: &Constant,
    context: &Context,
    ix: &StateIndex,
    ty: &Type,
    indices: &[usize],
) -> JsonStorageInitializers {
    match (&ty, &constant.value) {
        (_, ConstantValue::Undef) => vec![],
        (Type::Unit, ConstantValue::Unit) => vec![StorageInitializer {
            slot: get_storage_key(ix, indices),
            value: Bytes32::new([0; 32]),
        }],
        (Type::Bool, ConstantValue::Bool(b)) => {
            vec![StorageInitializer {
                slot: get_storage_key(ix, indices),
                value: Bytes32::new(
                    [0; 31]
                        .iter()
                        .cloned()
                        .chain([if *b { 0x01 } else { 0x00 }].iter().cloned())
                        .collect::<Vec<u8>>()
                        .try_into()
                        .unwrap(),
                ),
            }]
        }
        (Type::Uint(_), ConstantValue::Uint(n)) => {
            vec![StorageInitializer {
                slot: get_storage_key(ix, indices),
                value: Bytes32::new(
                    [0; 24]
                        .iter()
                        .cloned()
                        .chain(n.to_be_bytes().iter().cloned())
                        .collect::<Vec<u8>>()
                        .try_into()
                        .unwrap(),
                ),
            }]
        }
        (Type::B256, ConstantValue::B256(b)) => {
            vec![StorageInitializer {
                slot: get_storage_key(ix, indices),
                value: Bytes32::new(*b),
            }]
        }
        (Type::Array(_), ConstantValue::Array(_a)) => {
            unimplemented!("Arrays in storage have not been implemented yet.")
        }
        (Type::Struct(aggregate), ConstantValue::Struct(vec)) => {
            match &context.aggregates[aggregate.0] {
                AggregateContent::FieldTypes(field_tys) => vec
                    .iter()
                    .zip(field_tys.iter())
                    .enumerate()
                    .flat_map(|(i, (f, ty))| {
                        serialize_to_storage_slots(
                            f,
                            context,
                            ix,
                            ty,
                            &indices
                                .iter()
                                .cloned()
                                .chain(vec![i].iter().cloned())
                                .collect::<Vec<usize>>(),
                        )
                    })
                    .collect(),
                _ => unreachable!("Wrong content for struct."),
            }
        }
        (Type::Union(_), _) | (Type::String(_), _) => {
            // Serialize the constant data in words and add zero words until the number of words
            // is a multiple of 4. This is useful because each storage slot is 4 words.
            let mut packed = serialize_to_words(constant, context, ty);
            packed.extend(vec![
                Bytes8::new([0; 8]);
                ((packed.len() + 3) / 4) * 4 - packed.len()
            ]);

            assert!(packed.len() % 4 == 0);

            // Return a list of StorageInitializers
            // First get the keys then get the values
            (0..(ir_type_size_in_bytes(context, ty) + 31) / 32)
                .into_iter()
                .map(|i| add_to_b256(get_storage_key(ix, indices), i))
                .zip((0..packed.len() / 4).into_iter().map(|i| {
                    Bytes32::new(
                        Vec::from_iter((0..4).into_iter().flat_map(|j| *packed[4 * i + j]))
                            .try_into()
                            .unwrap(),
                    )
                }))
                .map(|(k, r)| StorageInitializer { slot: k, value: r })
                .collect()
        }
        _ => vec![],
    }
}

pub fn serialize_to_words(constant: &Constant, context: &Context, ty: &Type) -> Vec<Bytes8> {
    match (&ty, &constant.value) {
        (_, ConstantValue::Undef) => vec![],
        (Type::Unit, ConstantValue::Unit) => vec![Bytes8::new([0; 8])],
        (Type::Bool, ConstantValue::Bool(b)) => {
            vec![Bytes8::new(
                [0; 7]
                    .iter()
                    .cloned()
                    .chain([if *b { 0x01 } else { 0x00 }].iter().cloned())
                    .collect::<Vec<u8>>()
                    .try_into()
                    .unwrap(),
            )]
        }
        (Type::Uint(_), ConstantValue::Uint(n)) => {
            vec![Bytes8::new(n.to_be_bytes())]
        }
        (Type::B256, ConstantValue::B256(b)) => Vec::from_iter(
            (0..4)
                .into_iter()
                .map(|i| Bytes8::new(b[8 * i..8 * i + 8].try_into().unwrap())),
        ),
        (Type::String(_), ConstantValue::String(s)) => {
            // Turn the serialized words (Bytes8) into seriliazed storage slots (Bytes32)
            // Pad to word alignment
            let mut s = s.clone();
            s.extend(vec![0; ((s.len() + 3) / 4) * 4 - s.len()]);

            assert!(s.len() % 8 == 0);

            // Group into words
            Vec::from_iter((0..s.len() / 8).into_iter().map(|i| {
                Bytes8::new(
                    Vec::from_iter((0..8).into_iter().map(|j| s[8 * i + j]))
                        .try_into()
                        .unwrap(),
                )
            }))
        }
        (Type::Array(_), ConstantValue::Array(_a)) => {
            unimplemented!("Arrays in storage have not been implemented yet.")
        }
        (Type::Struct(aggregate), ConstantValue::Struct(vec)) => {
            match &context.aggregates[aggregate.0] {
                AggregateContent::FieldTypes(field_tys) => vec
                    .iter()
                    .zip(field_tys.iter())
                    .flat_map(|(f, ty)| serialize_to_words(f, context, ty))
                    .collect(),
                _ => unreachable!("Wrong content for struct."),
            }
        }
        (Type::Union(_), _) => {
            let value_size = ir_type_size_in_bytes(context, ty) / 8;
            let constant_size = ir_type_size_in_bytes(context, &constant.ty) / 8;
            assert!(value_size >= constant_size);

            // Add enough left padding to satisfy the actual size of the union
            let padding_size = value_size - constant_size;
            vec![Bytes8::new([0; 8]); padding_size as usize]
                .iter()
                .cloned()
                .chain(
                    serialize_to_words(constant, context, &constant.ty)
                        .iter()
                        .cloned(),
                )
                .collect()
        }
        _ => vec![],
    }
}
