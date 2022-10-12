use super::{
    compile::compile_function,
    convert::*,
    lexical_map::LexicalMap,
    storage::{add_to_b256, get_storage_key},
    types::*,
};
use crate::{
    asm_generation::from_ir::ir_type_size_in_bytes,
    declaration_engine::declaration_engine,
    ir_generation::const_eval::{
        compile_constant_expression, compile_constant_expression_to_constant,
    },
    language::{ty, *},
    metadata::MetadataManager,
    semantic_analysis::*,
    type_system::{look_up_type_id, to_typeinfo, TypeId, TypeInfo},
};
use sway_ast::intrinsics::Intrinsic;
use sway_error::error::{CompileError, Hint};
use sway_ir::{Context, *};
use sway_types::{
    constants,
    ident::Ident,
    integer_bits::IntegerBits,
    span::{Span, Spanned},
    state::StateIndex,
};

use std::collections::HashMap;

pub(super) struct FnCompiler {
    module: Module,
    pub(super) function: Function,
    pub(super) current_block: Block,
    pub(super) block_to_break_to: Option<Block>,
    pub(super) block_to_continue_to: Option<Block>,
    pub(super) current_fn_param: Option<TyFunctionParameter>,
    lexical_map: LexicalMap,
    recreated_fns: HashMap<(Span, Vec<TypeId>, Vec<TypeId>), Function>,
}

impl FnCompiler {
    pub(super) fn new(context: &mut Context, module: Module, function: Function) -> Self {
        let lexical_map = LexicalMap::from_iter(
            function
                .args_iter(context)
                .map(|(name, _value)| name.clone()),
        );
        FnCompiler {
            module,
            function,
            current_block: function.get_entry_block(context),
            block_to_break_to: None,
            block_to_continue_to: None,
            lexical_map,
            recreated_fns: HashMap::new(),
            current_fn_param: None,
        }
    }

    fn compile_with_new_scope<F, T>(&mut self, inner: F) -> Result<T, CompileError>
    where
        F: FnOnce(&mut FnCompiler) -> Result<T, CompileError>,
    {
        self.lexical_map.enter_scope();
        let result = inner(self);
        self.lexical_map.leave_scope();
        result
    }

    pub(super) fn compile_code_block(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_block: TyCodeBlock,
    ) -> Result<Value, CompileError> {
        self.compile_with_new_scope(|fn_compiler| {
            fn_compiler.compile_code_block_inner(context, md_mgr, ast_block)
        })
    }

    fn compile_code_block_inner(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_block: TyCodeBlock,
    ) -> Result<Value, CompileError> {
        self.lexical_map.enter_scope();

        let mut ast_nodes = ast_block.contents.into_iter();
        let value_res = loop {
            let ast_node = match ast_nodes.next() {
                Some(ast_node) => ast_node,
                None => break Ok(Constant::get_unit(context)),
            };
            match self.compile_ast_node(context, md_mgr, ast_node) {
                Ok(Some(val)) => break Ok(val),
                Ok(None) => (),
                Err(err) => break Err(err),
            }
        };

        self.lexical_map.leave_scope();
        value_res
    }

    fn compile_ast_node(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_node: TyAstNode,
    ) -> Result<Option<Value>, CompileError> {
        let span_md_idx = md_mgr.span_to_md(context, &ast_node.span);
        match ast_node.content {
            TyAstNodeContent::Declaration(td) => match td {
                ty::TyDeclaration::VariableDeclaration(tvd) => {
                    self.compile_var_decl(context, md_mgr, *tvd, span_md_idx)
                }
                ty::TyDeclaration::ConstantDeclaration(decl_id) => {
                    let tcd = declaration_engine::de_get_constant(decl_id, &ast_node.span)?;
                    self.compile_const_decl(context, md_mgr, tcd, span_md_idx)?;
                    Ok(None)
                }
                ty::TyDeclaration::FunctionDeclaration(_) => {
                    Err(CompileError::UnexpectedDeclaration {
                        decl_type: "function",
                        span: ast_node.span,
                    })
                }
                ty::TyDeclaration::TraitDeclaration(_) => {
                    Err(CompileError::UnexpectedDeclaration {
                        decl_type: "trait",
                        span: ast_node.span,
                    })
                }
                ty::TyDeclaration::StructDeclaration(_) => {
                    Err(CompileError::UnexpectedDeclaration {
                        decl_type: "struct",
                        span: ast_node.span,
                    })
                }
                ty::TyDeclaration::EnumDeclaration(decl_id) => {
                    let ted = declaration_engine::de_get_enum(decl_id, &ast_node.span)?;
                    create_enum_aggregate(context, ted.variants).map(|_| ())?;
                    Ok(None)
                }
                ty::TyDeclaration::ImplTrait(_) => {
                    // XXX What if we ignore the trait implementation???  Potentially since
                    // we currently inline everything and below we 'recreate' the functions
                    // lazily as they are called, nothing needs to be done here.  BUT!
                    // This is obviously not really correct, and eventually we want to
                    // compile and then call these properly.
                    Ok(None)
                }
                ty::TyDeclaration::AbiDeclaration(_) => Err(CompileError::UnexpectedDeclaration {
                    decl_type: "abi",
                    span: ast_node.span,
                }),
                ty::TyDeclaration::GenericTypeForFunctionScope { .. } => {
                    Err(CompileError::UnexpectedDeclaration {
                        decl_type: "abi",
                        span: ast_node.span,
                    })
                }
                ty::TyDeclaration::ErrorRecovery { .. } => {
                    Err(CompileError::UnexpectedDeclaration {
                        decl_type: "error recovery",
                        span: ast_node.span,
                    })
                }
                ty::TyDeclaration::StorageDeclaration(_) => {
                    Err(CompileError::UnexpectedDeclaration {
                        decl_type: "storage",
                        span: ast_node.span,
                    })
                }
            },
            TyAstNodeContent::Expression(te) => {
                // An expression with an ignored return value... I assume.
                let value = self.compile_expression(context, md_mgr, te)?;
                if value.is_diverging(context) {
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }
            TyAstNodeContent::ImplicitReturnExpression(te) => {
                let value = self.compile_expression(context, md_mgr, te)?;
                Ok(Some(value))
            }
            // a side effect can be () because it just impacts the type system/namespacing.
            // There should be no new IR generated.
            TyAstNodeContent::SideEffect => Ok(None),
        }
    }

    fn compile_expression(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_expr: ty::TyExpression,
    ) -> Result<Value, CompileError> {
        let span_md_idx = md_mgr.span_to_md(context, &ast_expr.span);
        match ast_expr.expression {
            ty::TyExpressionVariant::Literal(l) => {
                Ok(convert_literal_to_value(context, &l).add_metadatum(context, span_md_idx))
            }
            ty::TyExpressionVariant::FunctionApplication {
                call_path: name,
                contract_call_params,
                arguments,
                function_decl,
                self_state_idx,
                selector,
            } => {
                if let Some(metadata) = selector {
                    self.compile_contract_call(
                        context,
                        md_mgr,
                        &metadata,
                        &contract_call_params,
                        name.suffix.as_str(),
                        arguments,
                        ast_expr.return_type,
                        span_md_idx,
                    )
                } else {
                    self.compile_fn_call(
                        context,
                        md_mgr,
                        arguments,
                        function_decl,
                        self_state_idx,
                        span_md_idx,
                    )
                }
            }
            ty::TyExpressionVariant::LazyOperator { op, lhs, rhs } => {
                self.compile_lazy_op(context, md_mgr, op, *lhs, *rhs, span_md_idx)
            }
            ty::TyExpressionVariant::VariableExpression { name, .. } => {
                self.compile_var_expr(context, name.as_str(), span_md_idx)
            }
            ty::TyExpressionVariant::Array { contents } => {
                self.compile_array_expr(context, md_mgr, contents, span_md_idx)
            }
            ty::TyExpressionVariant::ArrayIndex { prefix, index } => {
                self.compile_array_index(context, md_mgr, *prefix, *index, span_md_idx)
            }
            ty::TyExpressionVariant::StructExpression { fields, .. } => {
                self.compile_struct_expr(context, md_mgr, fields, span_md_idx)
            }
            ty::TyExpressionVariant::CodeBlock(cb) => self.compile_code_block(context, md_mgr, cb),
            ty::TyExpressionVariant::FunctionParameter => Err(CompileError::Internal(
                "Unexpected function parameter declaration.",
                ast_expr.span,
            )),
            ty::TyExpressionVariant::IfExp {
                condition,
                then,
                r#else,
            } => self.compile_if(context, md_mgr, *condition, *then, r#else),
            ty::TyExpressionVariant::AsmExpression {
                registers,
                body,
                returns,
                whole_block_span,
            } => {
                let span_md_idx = md_mgr.span_to_md(context, &whole_block_span);
                self.compile_asm_expr(
                    context,
                    md_mgr,
                    registers,
                    body,
                    ast_expr.return_type,
                    returns,
                    span_md_idx,
                )
            }
            ty::TyExpressionVariant::StructFieldAccess {
                prefix,
                field_to_access,
                resolved_type_of_parent,
                ..
            } => {
                let span_md_idx = md_mgr.span_to_md(context, &field_to_access.span);
                self.compile_struct_field_expr(
                    context,
                    md_mgr,
                    *prefix,
                    resolved_type_of_parent,
                    field_to_access,
                    span_md_idx,
                )
            }
            ty::TyExpressionVariant::EnumInstantiation {
                enum_decl,
                tag,
                contents,
                ..
            } => self.compile_enum_expr(context, md_mgr, enum_decl, tag, contents),
            ty::TyExpressionVariant::Tuple { fields } => {
                self.compile_tuple_expr(context, md_mgr, fields, span_md_idx)
            }
            ty::TyExpressionVariant::TupleElemAccess {
                prefix,
                elem_to_access_num: idx,
                elem_to_access_span: span,
                resolved_type_of_parent: tuple_type,
            } => self.compile_tuple_elem_expr(context, md_mgr, *prefix, tuple_type, idx, span),
            ty::TyExpressionVariant::AbiCast { span, .. } => {
                let span_md_idx = md_mgr.span_to_md(context, &span);
                Ok(Constant::get_unit(context).add_metadatum(context, span_md_idx))
            }
            ty::TyExpressionVariant::StorageAccess(access) => {
                let span_md_idx = md_mgr.span_to_md(context, &access.span());
                self.compile_storage_access(
                    context,
                    md_mgr,
                    &access.fields,
                    &access.ix,
                    span_md_idx,
                )
            }
            ty::TyExpressionVariant::IntrinsicFunction(kind) => {
                self.compile_intrinsic_function(context, md_mgr, kind, ast_expr.span)
            }
            ty::TyExpressionVariant::AbiName(_) => {
                Ok(Value::new_constant(context, Constant::new_unit()))
            }
            ty::TyExpressionVariant::UnsafeDowncast { exp, variant } => {
                self.compile_unsafe_downcast(context, md_mgr, exp, variant)
            }
            ty::TyExpressionVariant::EnumTag { exp } => self.compile_enum_tag(context, md_mgr, exp),
            ty::TyExpressionVariant::WhileLoop { body, condition } => {
                self.compile_while_loop(context, md_mgr, body, *condition, span_md_idx)
            }
            ty::TyExpressionVariant::Break => {
                match self.block_to_break_to {
                    // If `self.block_to_break_to` is not None, then it has been set inside
                    // a loop and the use of `break` here is legal, so create a branch
                    // instruction. Error out otherwise.
                    Some(block_to_break_to) => Ok(self
                        .current_block
                        .ins(context)
                        .branch(block_to_break_to, vec![])),
                    None => Err(CompileError::BreakOutsideLoop {
                        span: ast_expr.span,
                    }),
                }
            }
            ty::TyExpressionVariant::Continue { .. } => match self.block_to_continue_to {
                // If `self.block_to_continue_to` is not None, then it has been set inside
                // a loop and the use of `continue` here is legal, so create a branch
                // instruction. Error out otherwise.
                Some(block_to_continue_to) => Ok(self
                    .current_block
                    .ins(context)
                    .branch(block_to_continue_to, vec![])),
                None => Err(CompileError::ContinueOutsideLoop {
                    span: ast_expr.span,
                }),
            },
            ty::TyExpressionVariant::Reassignment(reassignment) => {
                self.compile_reassignment(context, md_mgr, *reassignment, span_md_idx)
            }
            ty::TyExpressionVariant::StorageReassignment(storage_reassignment) => self
                .compile_storage_reassignment(
                    context,
                    md_mgr,
                    &storage_reassignment.fields,
                    &storage_reassignment.ix,
                    &storage_reassignment.rhs,
                    span_md_idx,
                ),
            ty::TyExpressionVariant::Return(exp) => {
                self.compile_return_statement(context, md_mgr, *exp)
            }
        }
    }

    fn compile_intrinsic_function(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        TyIntrinsicFunctionKind {
            kind,
            arguments,
            type_arguments,
            span: _,
        }: TyIntrinsicFunctionKind,
        span: Span,
    ) -> Result<Value, CompileError> {
        fn store_key_in_local_mem(
            compiler: &mut FnCompiler,
            context: &mut Context,
            value: Value,
            span_md_idx: Option<MetadataIndex>,
        ) -> Result<Value, CompileError> {
            // New name for the key
            let key_name = "key_for_storage".to_string();
            let alias_key_name = compiler.lexical_map.insert(key_name.as_str().to_owned());

            // Local pointer for the key
            let key_ptr = compiler
                .function
                .new_local_ptr(context, alias_key_name, Type::B256, true, None)
                .map_err(|ir_error| {
                    CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                })?;

            // Convert the key pointer to a value using get_ptr
            let key_ptr_ty = *key_ptr.get_type(context);
            let key_ptr_val = compiler
                .current_block
                .ins(context)
                .get_ptr(key_ptr, key_ptr_ty, 0)
                .add_metadatum(context, span_md_idx);

            // Store the value to the key pointer value
            compiler
                .current_block
                .ins(context)
                .store(key_ptr_val, value)
                .add_metadatum(context, span_md_idx);
            Ok(key_ptr_val)
        }

        // We safely index into arguments and type_arguments arrays below
        // because the type-checker ensures that the arguments are all there.
        match kind {
            Intrinsic::SizeOfVal => {
                let exp = arguments[0].clone();
                // Compile the expression in case of side-effects but ignore its value.
                let ir_type = convert_resolved_typeid(context, &exp.return_type, &exp.span)?;
                self.compile_expression(context, md_mgr, exp)?;
                Ok(Constant::get_uint(
                    context,
                    64,
                    ir_type_size_in_bytes(context, &ir_type),
                ))
            }
            Intrinsic::SizeOfType => {
                let targ = type_arguments[0].clone();
                let ir_type = convert_resolved_typeid(context, &targ.type_id, &targ.span)?;
                Ok(Constant::get_uint(
                    context,
                    64,
                    ir_type_size_in_bytes(context, &ir_type),
                ))
            }
            Intrinsic::IsReferenceType => {
                let targ = type_arguments[0].clone();
                let ir_type = convert_resolved_typeid(context, &targ.type_id, &targ.span)?;
                Ok(Constant::get_bool(context, !ir_type.is_copy_type()))
            }
            Intrinsic::GetStorageKey => {
                let span_md_idx = md_mgr.span_to_md(context, &span);
                Ok(self
                    .current_block
                    .ins(context)
                    .get_storage_key()
                    .add_metadatum(context, span_md_idx))
            }
            Intrinsic::Eq => {
                let lhs = arguments[0].clone();
                let rhs = arguments[1].clone();
                let lhs_value = self.compile_expression(context, md_mgr, lhs)?;
                let rhs_value = self.compile_expression(context, md_mgr, rhs)?;
                Ok(self
                    .current_block
                    .ins(context)
                    .cmp(Predicate::Equal, lhs_value, rhs_value))
            }
            Intrinsic::Gtf => {
                // The index is just a Value
                let index = self.compile_expression(context, md_mgr, arguments[0].clone())?;

                // The tx field ID has to be a compile-time constant because it becomes an
                // immediate
                let tx_field_id_constant = compile_constant_expression_to_constant(
                    context,
                    md_mgr,
                    self.module,
                    None,
                    &arguments[1],
                )?;
                let tx_field_id = match tx_field_id_constant.value {
                    ConstantValue::Uint(n) => n,
                    _ => {
                        return Err(CompileError::Internal(
                            "Transaction field ID for gtf intrinsic is not an integer. \
                            This should have been in caught in type checking",
                            span,
                        ))
                    }
                };

                // Get the target type from the type argument provided
                let target_type = type_arguments[0].clone();
                let target_ir_type =
                    convert_resolved_typeid(context, &target_type.type_id, &target_type.span)?;

                let span_md_idx = md_mgr.span_to_md(context, &span);

                // The `gtf` instruction
                let gtf_reg = self
                    .current_block
                    .ins(context)
                    .gtf(index, tx_field_id)
                    .add_metadatum(context, span_md_idx);

                // Reinterpret the result of th `gtf` instruction (which is always `u64`) as type
                // `T`. This requires an `int_to_ptr` instruction if `T` is a reference type.
                if target_ir_type.is_copy_type() {
                    Ok(gtf_reg)
                } else {
                    Ok(self
                        .current_block
                        .ins(context)
                        .int_to_ptr(gtf_reg, target_ir_type)
                        .add_metadatum(context, span_md_idx))
                }
            }
            Intrinsic::AddrOf => {
                let exp = arguments[0].clone();
                let value = self.compile_expression(context, md_mgr, exp)?;
                let span_md_idx = md_mgr.span_to_md(context, &span);
                Ok(self
                    .current_block
                    .ins(context)
                    .addr_of(value)
                    .add_metadatum(context, span_md_idx))
            }
            Intrinsic::StateLoadWord => {
                let exp = arguments[0].clone();
                let value = self.compile_expression(context, md_mgr, exp)?;
                let span_md_idx = md_mgr.span_to_md(context, &span);
                let key_ptr_val = store_key_in_local_mem(self, context, value, span_md_idx)?;
                Ok(self
                    .current_block
                    .ins(context)
                    .state_load_word(key_ptr_val)
                    .add_metadatum(context, span_md_idx))
            }
            Intrinsic::StateStoreWord => {
                let key_exp = arguments[0].clone();
                let val_exp = arguments[1].clone();
                // Validate that the val_exp is of the right type. We couldn't do it
                // earlier during type checking as the type arguments may not have been resolved.
                let val_ty = to_typeinfo(val_exp.return_type, &span)?;
                if !val_ty.is_copy_type() {
                    return Err(CompileError::IntrinsicUnsupportedArgType {
                        name: kind.to_string(),
                        span,
                        hint: Hint::new("This argument must be a copy type".to_string()),
                    });
                }
                let key_value = self.compile_expression(context, md_mgr, key_exp)?;
                let val_value = self.compile_expression(context, md_mgr, val_exp)?;
                let span_md_idx = md_mgr.span_to_md(context, &span);
                let key_ptr_val = store_key_in_local_mem(self, context, key_value, span_md_idx)?;
                Ok(self
                    .current_block
                    .ins(context)
                    .state_store_word(val_value, key_ptr_val)
                    .add_metadatum(context, span_md_idx))
            }
            Intrinsic::StateLoadQuad | Intrinsic::StateStoreQuad => {
                let key_exp = arguments[0].clone();
                let val_exp = arguments[1].clone();
                // Validate that the val_exp is of the right type. We couldn't do it
                // earlier during type checking as the type arguments may not have been resolved.
                let val_ty = to_typeinfo(val_exp.return_type, &span)?;
                if val_ty != TypeInfo::UnsignedInteger(IntegerBits::SixtyFour) {
                    return Err(CompileError::IntrinsicUnsupportedArgType {
                        name: kind.to_string(),
                        span,
                        hint: Hint::new("This argument must be u64".to_string()),
                    });
                }
                let key_value = self.compile_expression(context, md_mgr, key_exp)?;
                let val_value = self.compile_expression(context, md_mgr, val_exp)?;
                let span_md_idx = md_mgr.span_to_md(context, &span);
                let key_ptr_val = store_key_in_local_mem(self, context, key_value, span_md_idx)?;
                // For quad word, the IR instructions take in a pointer rather than a raw u64.
                let val_ptr = self
                    .current_block
                    .ins(context)
                    .int_to_ptr(val_value, Type::B256)
                    .add_metadatum(context, span_md_idx);
                match kind {
                    Intrinsic::StateLoadQuad => Ok(self
                        .current_block
                        .ins(context)
                        .state_load_quad_word(val_ptr, key_ptr_val)
                        .add_metadatum(context, span_md_idx)),
                    Intrinsic::StateStoreQuad => Ok(self
                        .current_block
                        .ins(context)
                        .state_store_quad_word(val_ptr, key_ptr_val)
                        .add_metadatum(context, span_md_idx)),
                    _ => unreachable!(),
                }
            }
            Intrinsic::Log => {
                // The log value and the log ID are just Value.
                let log_val = self.compile_expression(context, md_mgr, arguments[0].clone())?;
                let log_id = convert_literal_to_value(
                    context,
                    &Literal::U64(*arguments[0].return_type as u64),
                );

                match log_val.get_stripped_ptr_type(context) {
                    None => Err(CompileError::Internal(
                        "Unable to determine type for logged value.",
                        span,
                    )),
                    Some(log_ty) => {
                        let span_md_idx = md_mgr.span_to_md(context, &span);

                        // The `log` instruction
                        Ok(self
                            .current_block
                            .ins(context)
                            .log(log_val, log_ty, log_id)
                            .add_metadatum(context, span_md_idx))
                    }
                }
            }
            Intrinsic::Add | Intrinsic::Sub | Intrinsic::Mul | Intrinsic::Div => {
                let op = match kind {
                    Intrinsic::Add => BinaryOpKind::Add,
                    Intrinsic::Sub => BinaryOpKind::Sub,
                    Intrinsic::Mul => BinaryOpKind::Mul,
                    Intrinsic::Div => BinaryOpKind::Div,
                    _ => unreachable!(),
                };
                let lhs = arguments[0].clone();
                let rhs = arguments[1].clone();
                let lhs_value = self.compile_expression(context, md_mgr, lhs)?;
                let rhs_value = self.compile_expression(context, md_mgr, rhs)?;
                Ok(self
                    .current_block
                    .ins(context)
                    .binary_op(op, lhs_value, rhs_value))
            }
            Intrinsic::Revert => {
                let revert_code_val =
                    self.compile_expression(context, md_mgr, arguments[0].clone())?;

                // The `revert` instruction
                let span_md_idx = md_mgr.span_to_md(context, &span);
                Ok(self
                    .current_block
                    .ins(context)
                    .revert(revert_code_val)
                    .add_metadatum(context, span_md_idx))
            }
        }
    }

    fn compile_return_statement(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_expr: ty::TyExpression,
    ) -> Result<Value, CompileError> {
        // Nothing to do if the current block already has a terminator
        if self.current_block.is_terminated(context) {
            return Ok(Constant::get_unit(context));
        }

        let ret_value = self.compile_expression(context, md_mgr, ast_expr.clone())?;
        if ret_value.is_diverging(context) {
            return Ok(ret_value);
        }
        match ret_value.get_stripped_ptr_type(context) {
            None => Err(CompileError::Internal(
                "Unable to determine type for return statement expression.",
                ast_expr.span,
            )),
            Some(ret_ty) => {
                let span_md_idx = md_mgr.span_to_md(context, &ast_expr.span);
                Ok(self
                    .current_block
                    .ins(context)
                    .ret(ret_value, ret_ty)
                    .add_metadatum(context, span_md_idx))
            }
        }
    }

    fn compile_lazy_op(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_op: LazyOp,
        ast_lhs: ty::TyExpression,
        ast_rhs: ty::TyExpression,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // Short-circuit: if LHS is true for AND we still must eval the RHS block; for OR we can
        // skip the RHS block, and vice-versa.
        let lhs_val = self.compile_expression(context, md_mgr, ast_lhs)?;
        let cond_block_end = self.current_block;
        let rhs_block = self.function.create_block(context, None);
        let final_block = self.function.create_block(context, None);

        self.current_block = rhs_block;
        let rhs_val = self.compile_expression(context, md_mgr, ast_rhs)?;

        let merge_val_arg_idx = final_block.new_arg(
            context,
            lhs_val
                .get_type(context)
                .unwrap_or_else(|| rhs_val.get_type(context).unwrap_or(Type::Unit)),
        );

        if !cond_block_end.is_terminated(context) {
            let cond_builder = cond_block_end.ins(context);
            match ast_op {
                LazyOp::And => cond_builder.conditional_branch(
                    lhs_val,
                    rhs_block,
                    final_block,
                    vec![],
                    vec![lhs_val],
                ),
                LazyOp::Or => cond_builder.conditional_branch(
                    lhs_val,
                    final_block,
                    rhs_block,
                    vec![lhs_val],
                    vec![],
                ),
            }
            .add_metadatum(context, span_md_idx);
        }

        if !self.current_block.is_terminated(context) {
            self.current_block
                .ins(context)
                .branch(final_block, vec![rhs_val])
                .add_metadatum(context, span_md_idx);
        }

        self.current_block = final_block;
        Ok(final_block.get_arg(context, merge_val_arg_idx).unwrap())
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_contract_call(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        call_params: &ContractCallParams,
        contract_call_parameters: &HashMap<String, ty::TyExpression>,
        ast_name: &str,
        ast_args: Vec<(Ident, ty::TyExpression)>,
        return_type: TypeId,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // Compile each user argument
        let compiled_args = ast_args
            .into_iter()
            .map(|(_, expr)| self.compile_expression(context, md_mgr, expr))
            .collect::<Result<Vec<Value>, CompileError>>()?;

        let user_args_val = match compiled_args.len() {
            0 => Constant::get_uint(context, 64, 0),
            1 => {
                // The single arg doesn't need to be put into a struct.
                let arg0 = compiled_args[0];
                let arg0_type = arg0.get_stripped_ptr_type(context).unwrap();
                if arg0_type.is_copy_type() {
                    self.current_block
                        .ins(context)
                        .bitcast(arg0, Type::Uint(64))
                        .add_metadatum(context, span_md_idx)
                } else {
                    // Copy this value to a new location.  This is quite inefficient but we need to
                    // pass by reference rather than by value.  Optimisation passes can remove all
                    // the unnecessary copying eventually, though it feels like we're jumping
                    // through a bunch of hoops here (employing the single arg optimisation) for
                    // minimal returns.
                    let by_reference_arg_name = self
                        .lexical_map
                        .insert(format!("{}{}", "arg_for_", ast_name));
                    let by_reference_arg = self
                        .function
                        .new_local_ptr(context, by_reference_arg_name, arg0_type, false, None)
                        .map_err(|ir_error| {
                            CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                        })?;

                    let arg0_ptr =
                        self.current_block
                            .ins(context)
                            .get_ptr(by_reference_arg, arg0_type, 0);
                    self.current_block.ins(context).store(arg0_ptr, arg0);

                    // NOTE: Here we're fetching the original stack pointer, cast to u64.
                    // TODO: Instead of casting here, we should use an `ptrtoint` instruction.
                    self.current_block
                        .ins(context)
                        .get_ptr(by_reference_arg, Type::Uint(64), 0)
                        .add_metadatum(context, span_md_idx)
                }
            }
            _ => {
                // New struct type to hold the user arguments bundled together.
                let field_types = compiled_args
                    .iter()
                    .filter_map(|val| val.get_stripped_ptr_type(context))
                    .collect::<Vec<_>>();
                let user_args_struct_aggregate = Aggregate::new_struct(context, field_types);

                // New local pointer for the struct to hold all user arguments
                let user_args_struct_local_name = self
                    .lexical_map
                    .insert(format!("{}{}", "args_struct_for_", ast_name));
                let user_args_struct_ptr = self
                    .function
                    .new_local_ptr(
                        context,
                        user_args_struct_local_name,
                        Type::Struct(user_args_struct_aggregate),
                        true,
                        None,
                    )
                    .map_err(|ir_error| {
                        CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                    })?;

                // Initialise each of the fields in the user args struct.
                compiled_args.into_iter().enumerate().fold(
                    self.current_block
                        .ins(context)
                        .get_ptr(
                            user_args_struct_ptr,
                            Type::Struct(user_args_struct_aggregate),
                            0,
                        )
                        .add_metadatum(context, span_md_idx),
                    |user_args_struct_ptr_val, (insert_idx, insert_val)| {
                        self.current_block
                            .ins(context)
                            .insert_value(
                                user_args_struct_ptr_val,
                                user_args_struct_aggregate,
                                insert_val,
                                vec![insert_idx as u64],
                            )
                            .add_metadatum(context, span_md_idx)
                    },
                );

                // NOTE: Here we're fetching the original stack pointer, cast to u64.
                self.current_block
                    .ins(context)
                    .get_ptr(user_args_struct_ptr, Type::Uint(64), 0)
                    .add_metadatum(context, span_md_idx)
            }
        };

        // Now handle the contract address and the selector. The contract address is just
        // as B256 while the selector is a [u8; 4] which we have to convert to a U64.
        let ra_struct_aggregate = Aggregate::new_struct(
            context,
            [Type::B256, Type::Uint(64), Type::Uint(64)].to_vec(),
        );

        let ra_struct_ptr = self
            .function
            .new_local_ptr(
                context,
                self.lexical_map.insert_anon(),
                Type::Struct(ra_struct_aggregate),
                false,
                None,
            )
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;
        let ra_struct_ptr_ty = *ra_struct_ptr.get_type(context);
        let mut ra_struct_val = self
            .current_block
            .ins(context)
            .get_ptr(ra_struct_ptr, ra_struct_ptr_ty, 0)
            .add_metadatum(context, span_md_idx);

        // Insert the contract address
        let addr =
            self.compile_expression(context, md_mgr, *call_params.contract_address.clone())?;
        ra_struct_val = self
            .current_block
            .ins(context)
            .insert_value(ra_struct_val, ra_struct_aggregate, addr, vec![0])
            .add_metadatum(context, span_md_idx);

        // Convert selector to U64 and then insert it
        let sel = call_params.func_selector;
        let sel_val = convert_literal_to_value(
            context,
            &Literal::U64(
                sel[3] as u64 + 256 * (sel[2] as u64 + 256 * (sel[1] as u64 + 256 * sel[0] as u64)),
            ),
        )
        .add_metadatum(context, span_md_idx);
        ra_struct_val = self
            .current_block
            .ins(context)
            .insert_value(ra_struct_val, ra_struct_aggregate, sel_val, vec![1])
            .add_metadatum(context, span_md_idx);

        // Insert the user args value.
        ra_struct_val = self
            .current_block
            .ins(context)
            .insert_value(ra_struct_val, ra_struct_aggregate, user_args_val, vec![2])
            .add_metadatum(context, span_md_idx);

        // Compile all other call parameters
        let coins = match contract_call_parameters
            .get(&constants::CONTRACT_CALL_COINS_PARAMETER_NAME.to_string())
        {
            Some(coins_expr) => self.compile_expression(context, md_mgr, coins_expr.clone())?,
            None => convert_literal_to_value(
                context,
                &Literal::U64(constants::CONTRACT_CALL_COINS_PARAMETER_DEFAULT_VALUE),
            )
            .add_metadatum(context, span_md_idx),
        };

        let asset_id = match contract_call_parameters
            .get(&constants::CONTRACT_CALL_ASSET_ID_PARAMETER_NAME.to_string())
        {
            Some(asset_id_expr) => {
                self.compile_expression(context, md_mgr, asset_id_expr.clone())?
            }
            None => convert_literal_to_value(
                context,
                &Literal::B256(constants::CONTRACT_CALL_ASSET_ID_PARAMETER_DEFAULT_VALUE),
            )
            .add_metadatum(context, span_md_idx),
        };

        let gas = match contract_call_parameters
            .get(&constants::CONTRACT_CALL_GAS_PARAMETER_NAME.to_string())
        {
            Some(gas_expr) => self.compile_expression(context, md_mgr, gas_expr.clone())?,
            None => self
                .current_block
                .ins(context)
                .read_register(sway_ir::Register::Cgas)
                .add_metadatum(context, span_md_idx),
        };

        let return_type = convert_resolved_typeid_no_span(context, &return_type)?;

        // Insert the contract_call instruction
        Ok(self
            .current_block
            .ins(context)
            .contract_call(
                return_type,
                ast_name.to_string(),
                ra_struct_val,
                coins,
                asset_id,
                gas,
            )
            .add_metadatum(context, span_md_idx))
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_fn_call(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_args: Vec<(Ident, ty::TyExpression)>,
        callee: TyFunctionDeclaration,
        self_state_idx: Option<StateIndex>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // The compiler inlines everything very lazily.  Function calls include the body of the
        // callee (i.e., the callee_body arg above). Library functions are provided in an initial
        // namespace from Forc and when the parser builds the AST (or is it during type checking?)
        // these function bodies are embedded.
        //
        // Here we build little single-use instantiations of the callee and then call them.  Naming
        // is not yet absolute so we must ensure the function names are unique.
        //

        // Eventually we need to Do It Properly and inline into the AST only when necessary, and
        // compile the standard library to an actual module.

        // Get the callee from the cache if we've already compiled it.  We can't insert it with
        // .entry() since `compile_function()` returns a Result we need to handle.  The key to our
        // cache, to uniquely identify a function instance, is the span and the type IDs of any
        // args and type parameters.  It's using the Sway types rather than IR types, which would
        // be more accurate but also more fiddly.
        let fn_key = (
            callee.span(),
            callee.parameters.iter().map(|p| p.type_id).collect(),
            callee.type_parameters.iter().map(|tp| tp.type_id).collect(),
        );
        let new_callee = match self.recreated_fns.get(&fn_key).copied() {
            Some(func) => func,
            None => {
                let callee_fn_decl = TyFunctionDeclaration {
                    type_parameters: Vec::new(),
                    name: Ident::new(Span::from_string(format!(
                        "{}_{}",
                        callee.name,
                        context.get_unique_id()
                    ))),
                    parameters: callee.parameters.clone(),
                    ..callee
                };
                let new_func =
                    compile_function(context, md_mgr, self.module, callee_fn_decl)?.unwrap();
                self.recreated_fns.insert(fn_key, new_func);
                new_func
            }
        };

        // Now actually call the new function.
        let args = {
            let mut args = Vec::with_capacity(ast_args.len());
            for ((_, expr), param) in ast_args.into_iter().zip(callee.parameters.into_iter()) {
                self.current_fn_param = Some(param);
                let arg = self.compile_expression(context, md_mgr, expr)?;
                if arg.is_diverging(context) {
                    return Ok(arg);
                }
                self.current_fn_param = None;
                args.push(arg);
            }
            args
        };
        let state_idx_md_idx = match self_state_idx {
            Some(self_state_idx) => {
                md_mgr.storage_key_to_md(context, self_state_idx.to_usize() as u64)
            }
            None => None,
        };
        Ok(self
            .current_block
            .ins(context)
            .call(new_callee, &args)
            .add_metadatum(context, span_md_idx)
            .add_metadatum(context, state_idx_md_idx))
    }

    fn compile_if(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_condition: ty::TyExpression,
        ast_then: ty::TyExpression,
        ast_else: Option<Box<ty::TyExpression>>,
    ) -> Result<Value, CompileError> {
        // Compile the condition expression in the entry block.  Then save the current block so we
        // can jump to the true and false blocks after we've created them.
        let cond_span_md_idx = md_mgr.span_to_md(context, &ast_condition.span);
        let cond_value = self.compile_expression(context, md_mgr, ast_condition)?;
        if cond_value.is_diverging(context) {
            return Ok(cond_value);
        }
        let cond_block = self.current_block;

        // To keep the blocks in a nice order we create them only as we populate them.  It's
        // possible when compiling other expressions for the 'current' block to change, and it
        // should always be the block to which instructions are added.  So for the true and false
        // blocks we create them in turn, compile their contents and save the current block
        // afterwards.
        //
        // Then once they're both created we can add the conditional branch to them from the entry
        // block.
        //
        // Then we create the merge block and jump from the saved blocks to it, again to keep them
        // in a nice top-to-bottom order.  Perhaps there's a better way to order them, using
        // post-processing CFG analysis, but... meh.

        let true_block_begin = self.function.create_block(context, None);
        self.current_block = true_block_begin;
        let true_value = self.compile_expression(context, md_mgr, ast_then)?;
        let true_block_end = self.current_block;

        let false_block_begin = self.function.create_block(context, None);
        self.current_block = false_block_begin;
        let false_value = match ast_else {
            None => Constant::get_unit(context),
            Some(expr) => self.compile_expression(context, md_mgr, *expr)?,
        };
        let false_block_end = self.current_block;

        cond_block
            .ins(context)
            .conditional_branch(
                cond_value,
                true_block_begin,
                false_block_begin,
                vec![],
                vec![],
            )
            .add_metadatum(context, cond_span_md_idx);

        let merge_block = self.function.create_block(context, None);
        // Add a single argument to merge_block that merges true_value and false_value.
        let merge_val_arg_idx = merge_block.new_arg(
            context,
            true_value
                .get_type(context)
                .unwrap_or_else(|| false_value.get_type(context).unwrap_or(Type::Unit)),
        );
        if !true_block_end.is_terminated(context) {
            true_block_end
                .ins(context)
                .branch(merge_block, vec![true_value]);
        }
        if !false_block_end.is_terminated(context) {
            false_block_end
                .ins(context)
                .branch(merge_block, vec![false_value]);
        }

        self.current_block = merge_block;
        Ok(merge_block.get_arg(context, merge_val_arg_idx).unwrap())
    }

    fn compile_unsafe_downcast(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        exp: Box<ty::TyExpression>,
        variant: TyEnumVariant,
    ) -> Result<Value, CompileError> {
        // retrieve the aggregate info for the enum
        let enum_aggregate = match convert_resolved_typeid(context, &exp.return_type, &exp.span)? {
            Type::Struct(aggregate) => aggregate,
            _ => {
                return Err(CompileError::Internal(
                    "Enum type for `unsafe downcast` is not an enum.",
                    exp.span,
                ));
            }
        };
        // compile the expression to asm
        let compiled_value = self.compile_expression(context, md_mgr, *exp)?;
        // retrieve the value minus the tag
        Ok(self.current_block.ins(context).extract_value(
            compiled_value,
            enum_aggregate,
            vec![1, variant.tag as u64],
        ))
    }

    fn compile_enum_tag(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        exp: Box<ty::TyExpression>,
    ) -> Result<Value, CompileError> {
        let tag_span_md_idx = md_mgr.span_to_md(context, &exp.span);
        let enum_aggregate = match convert_resolved_typeid(context, &exp.return_type, &exp.span)? {
            Type::Struct(aggregate) => aggregate,
            _ => {
                return Err(CompileError::Internal("Expected enum type here.", exp.span));
            }
        };
        let exp = self.compile_expression(context, md_mgr, *exp)?;
        Ok(self
            .current_block
            .ins(context)
            .extract_value(exp, enum_aggregate, vec![0])
            .add_metadatum(context, tag_span_md_idx))
    }

    fn compile_while_loop(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        body: TyCodeBlock,
        condition: ty::TyExpression,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // We're dancing around a bit here to make the blocks sit in the right order.  Ideally we
        // have the cond block, followed by the body block which may contain other blocks, and the
        // final block comes after any body block(s).

        // Jump to the while cond block.
        let cond_block = self.function.create_block(context, Some("while".into()));

        if !self.current_block.is_terminated(context) {
            self.current_block.ins(context).branch(cond_block, vec![]);
        }

        // Fill in the body block now, jump unconditionally to the cond block at its end.
        let body_block = self
            .function
            .create_block(context, Some("while_body".into()));

        // Create the final block after we're finished with the body.
        let final_block = self
            .function
            .create_block(context, Some("end_while".into()));

        // Keep track of the previous blocks we have to jump to in case of a break or a continue.
        // This should be `None` if we're not in a loop already or the previous break or continue
        // destinations for the outer loop that contains the current loop.
        let prev_block_to_break_to = self.block_to_break_to;
        let prev_block_to_continue_to = self.block_to_continue_to;

        // Keep track of the current blocks to jump to in case of a break or continue.
        self.block_to_break_to = Some(final_block);
        self.block_to_continue_to = Some(cond_block);

        // Compile the body and a branch to the condition block if no branch is already present in
        // the body block
        self.current_block = body_block;
        self.compile_code_block(context, md_mgr, body)?;
        if !self.current_block.is_terminated(context) {
            self.current_block.ins(context).branch(cond_block, vec![]);
        }

        // Restore the blocks to jump to now that we're done with the current loop
        self.block_to_break_to = prev_block_to_break_to;
        self.block_to_continue_to = prev_block_to_continue_to;

        // Add the conditional which jumps into the body or out to the final block.
        self.current_block = cond_block;
        let cond_value = self.compile_expression(context, md_mgr, condition)?;
        if !self.current_block.is_terminated(context) {
            self.current_block.ins(context).conditional_branch(
                cond_value,
                body_block,
                final_block,
                vec![],
                vec![],
            );
        }

        self.current_block = final_block;
        Ok(Constant::get_unit(context).add_metadatum(context, span_md_idx))
    }

    fn compile_var_expr(
        &mut self,
        context: &mut Context,
        name: &str,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // We need to check the symbol map first, in case locals are shadowing the args, other
        // locals or even constants.
        if let Some(ptr) = self
            .lexical_map
            .get(name)
            .and_then(|local_name| self.function.get_local_ptr(context, local_name))
        {
            let ptr_ty = *ptr.get_type(context);
            let ptr_val = self
                .current_block
                .ins(context)
                .get_ptr(ptr, ptr_ty, 0)
                .add_metadatum(context, span_md_idx);
            let fn_param = self.current_fn_param.as_ref();
            let is_ref_primitive = fn_param.is_some()
                && look_up_type_id(fn_param.unwrap().type_id).is_copy_type()
                && fn_param.unwrap().is_reference
                && fn_param.unwrap().is_mutable;
            Ok(if ptr.is_aggregate_ptr(context) || is_ref_primitive {
                ptr_val
            } else {
                self.current_block
                    .ins(context)
                    .load(ptr_val)
                    .add_metadatum(context, span_md_idx)
            })
        } else if let Some(val) = self.function.get_arg(context, name) {
            let is_ptr = val.get_type(context).filter(|f| f.is_ptr_type()).is_some();
            if is_ptr {
                Ok(self
                    .current_block
                    .ins(context)
                    .load(val)
                    .add_metadatum(context, span_md_idx))
            } else {
                Ok(val)
            }
        } else if let Some(const_val) = self.module.get_global_constant(context, name) {
            Ok(const_val)
        } else {
            Err(CompileError::InternalOwned(
                format!("Unable to resolve variable '{name}'."),
                Span::dummy(),
            ))
        }
    }

    fn compile_var_decl(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_var_decl: TyVariableDeclaration,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Option<Value>, CompileError> {
        let TyVariableDeclaration {
            name,
            body,
            mutability,
            ..
        } = ast_var_decl;
        // Nothing to do for an abi cast declarations. The address specified in them is already
        // provided in each contract call node in the AST.
        if matches!(
            &to_typeinfo(body.return_type, &body.span).map_err(|ty_err| {
                CompileError::InternalOwned(format!("{:?}", ty_err), body.span.clone())
            })?,
            TypeInfo::ContractCaller { .. }
        ) {
            return Ok(None);
        }

        // Grab these before we move body into compilation.
        let return_type = convert_resolved_typeid(context, &body.return_type, &body.span)?;

        // We must compile the RHS before checking for shadowing, as it will still be in the
        // previous scope.
        let init_val = self.compile_expression(context, md_mgr, body)?;
        if init_val.is_diverging(context) {
            return Ok(Some(init_val));
        }
        let local_name = self.lexical_map.insert(name.as_str().to_owned());
        let ptr = self
            .function
            .new_local_ptr(
                context,
                local_name,
                return_type,
                mutability.is_mutable(),
                None,
            )
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        // We can have empty aggregates, especially arrays, which shouldn't be initialised, but
        // otherwise use a store.
        let ptr_ty = *ptr.get_type(context);
        if ir_type_size_in_bytes(context, &ptr_ty) > 0 {
            let ptr_val = self
                .current_block
                .ins(context)
                .get_ptr(ptr, ptr_ty, 0)
                .add_metadatum(context, span_md_idx);
            self.current_block
                .ins(context)
                .store(ptr_val, init_val)
                .add_metadatum(context, span_md_idx);
        }
        Ok(None)
    }

    fn compile_const_decl(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_const_decl: TyConstantDeclaration,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<(), CompileError> {
        // This is local to the function, so we add it to the locals, rather than the module
        // globals like other const decls.
        let TyConstantDeclaration { name, value, .. } = ast_const_decl;
        let const_expr_val =
            compile_constant_expression(context, md_mgr, self.module, None, &value)?;
        let local_name = self.lexical_map.insert(name.as_str().to_owned());
        let return_type = convert_resolved_typeid(context, &value.return_type, &value.span)?;

        // We compile consts the same as vars are compiled. This is because ASM generation
        // cannot handle
        //    1. initializing aggregates
        //    2. get_ptr()
        // into the data section.
        let ptr = self
            .function
            .new_local_ptr(context, local_name, return_type, false, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        // We can have empty aggregates, especially arrays, which shouldn't be initialised, but
        // otherwise use a store.
        let ptr_ty = *ptr.get_type(context);
        if ir_type_size_in_bytes(context, &ptr_ty) > 0 {
            let ptr_val = self
                .current_block
                .ins(context)
                .get_ptr(ptr, ptr_ty, 0)
                .add_metadatum(context, span_md_idx);
            self.current_block
                .ins(context)
                .store(ptr_val, const_expr_val)
                .add_metadatum(context, span_md_idx);
        }
        Ok(())
    }

    fn compile_reassignment(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_reassignment: TyReassignment,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let name = self
            .lexical_map
            .get(ast_reassignment.lhs_base_name.as_str())
            .expect("All local symbols must be in the lexical symbol map.");

        // First look for a local ptr with the required name
        let val = match self.function.get_local_ptr(context, name) {
            Some(ptr) => {
                let ptr_ty = *ptr.get_type(context);
                self.current_block
                    .ins(context)
                    .get_ptr(ptr, ptr_ty, 0)
                    .add_metadatum(context, span_md_idx)
            }
            None => {
                // Now look for an argument with the required name
                self.function
                    .args_iter(context)
                    .find(|arg| &arg.0 == name)
                    .ok_or_else(|| {
                        CompileError::InternalOwned(
                            format!("variable not found: {name}"),
                            ast_reassignment.lhs_base_name.span(),
                        )
                    })?
                    .1
            }
        };

        let reassign_val = self.compile_expression(context, md_mgr, ast_reassignment.rhs)?;
        if reassign_val.is_diverging(context) {
            return Ok(reassign_val);
        }

        if ast_reassignment.lhs_indices.is_empty() {
            // A non-aggregate; use a `store`.
            self.current_block
                .ins(context)
                .store(val, reassign_val)
                .add_metadatum(context, span_md_idx);
        } else {
            // An aggregate.  Iterate over the field names from the left hand side and collect
            // field indices.  The struct type from the previous iteration is used to determine the
            // field type for the current iteration.
            let field_idcs = get_indices_for_struct_access(
                ast_reassignment.lhs_type,
                &ast_reassignment.lhs_indices,
            )?;

            let ty = match val.get_stripped_ptr_type(context).unwrap() {
                Type::Struct(aggregate) => aggregate,
                _otherwise => {
                    let spans = ast_reassignment
                        .lhs_indices
                        .iter()
                        .fold(ast_reassignment.lhs_base_name.span(), |acc, lhs| {
                            Span::join(acc, lhs.span())
                        });
                    return Err(CompileError::Internal(
                        "Reassignment with multiple accessors to non-aggregate.",
                        spans,
                    ));
                }
            };

            self.current_block
                .ins(context)
                .insert_value(val, ty, reassign_val, field_idcs)
                .add_metadatum(context, span_md_idx);
        }

        Ok(Constant::get_unit(context).add_metadatum(context, span_md_idx))
    }

    fn compile_storage_reassignment(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        fields: &[TyStorageReassignDescriptor],
        ix: &StateIndex,
        rhs: &ty::TyExpression,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // Compile the RHS into a value
        let rhs = self.compile_expression(context, md_mgr, rhs.clone())?;
        if rhs.is_diverging(context) {
            return Ok(rhs);
        }

        // Get the type of the access which can be a subfield
        let access_type = convert_resolved_typeid_no_span(
            context,
            &fields.last().expect("guaranteed by grammar").type_id,
        )?;

        // Get the list of indices used to access the storage field. This will be empty
        // if the storage field type is not a struct.
        let base_type = fields[0].type_id;
        let field_idcs = get_indices_for_struct_access(base_type, &fields[1..])?;

        // Do the actual work. This is a recursive function because we want to drill down
        // to store each primitive type in the storage field in its own storage slot.
        self.compile_storage_write(
            context,
            md_mgr,
            ix,
            &field_idcs,
            &access_type,
            rhs,
            span_md_idx,
        )?;
        Ok(Constant::get_unit(context).add_metadatum(context, span_md_idx))
    }

    fn compile_array_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        contents: Vec<ty::TyExpression>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let elem_type = if contents.is_empty() {
            // A zero length array is a pointer to nothing, which is still supported by Sway.
            // We're unable to get the type though it's irrelevant because it can't be indexed, so
            // we'll just use Unit.
            Type::Unit
        } else {
            convert_resolved_typeid_no_span(context, &contents[0].return_type)?
        };
        let aggregate = Aggregate::new_array(context, elem_type, contents.len() as u64);

        // Compile each element and insert it immediately.
        let temp_name = self.lexical_map.insert_anon();
        let array_ptr = self
            .function
            .new_local_ptr(context, temp_name, Type::Array(aggregate), false, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;
        let array_ptr_ty = *array_ptr.get_type(context);
        let mut array_value = self
            .current_block
            .ins(context)
            .get_ptr(array_ptr, array_ptr_ty, 0)
            .add_metadatum(context, span_md_idx);

        for (idx, elem_expr) in contents.into_iter().enumerate() {
            let elem_value = self.compile_expression(context, md_mgr, elem_expr)?;
            if elem_value.is_diverging(context) {
                return Ok(elem_value);
            }
            let index_val =
                Constant::get_uint(context, 64, idx as u64).add_metadatum(context, span_md_idx);
            array_value = self
                .current_block
                .ins(context)
                .insert_element(array_value, aggregate, elem_value, index_val)
                .add_metadatum(context, span_md_idx);
        }
        Ok(array_value)
    }

    fn compile_array_index(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        array_expr: ty::TyExpression,
        index_expr: ty::TyExpression,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let array_expr_span = array_expr.span.clone();

        let array_val = self.compile_expression(context, md_mgr, array_expr)?;
        if array_val.is_diverging(context) {
            return Ok(array_val);
        }

        let aggregate = if let Some(instruction) = array_val.get_instruction(context) {
            instruction.get_aggregate(context).ok_or_else(|| {
                CompileError::InternalOwned(
                    format!(
                        "Unsupported instruction as array value for index expression. \
                        {instruction:?}"
                    ),
                    array_expr_span,
                )
            })
        } else if let Some(Type::Array(agg)) = array_val.get_argument_type(context) {
            Ok(agg)
        } else if let Some(Constant {
            ty: Type::Array(agg),
            ..
        }) = array_val.get_constant(context)
        {
            Ok(*agg)
        } else {
            Err(CompileError::InternalOwned(
                "Unsupported array value for index expression.".to_owned(),
                array_expr_span,
            ))
        }?;

        // Check for out of bounds if we have a literal index.
        let (_, count) = aggregate.get_content(context).array_type();
        if let ty::TyExpressionVariant::Literal(Literal::U64(index)) = index_expr.expression {
            if index >= *count {
                // XXX Here is a very specific case where we want to return an Error enum
                // specifically, if not an actual CompileError.  This should be a
                // CompileError::ArrayOutOfBounds, or at least converted to one.
                return Err(CompileError::ArrayOutOfBounds {
                    index,
                    count: *count,
                    span: index_expr.span,
                });
            }
        }

        let index_val = self.compile_expression(context, md_mgr, index_expr)?;
        if index_val.is_diverging(context) {
            return Ok(index_val);
        }

        Ok(self
            .current_block
            .ins(context)
            .extract_element(array_val, aggregate, index_val)
            .add_metadatum(context, span_md_idx))
    }

    fn compile_struct_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        fields: Vec<TyStructExpressionField>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // NOTE: This is a struct instantiation with initialisers for each field of a named struct.
        // We don't know the actual type of the struct, but the AST guarantees that the fields are
        // in the declared order (regardless of how they are initialised in source) so we can
        // create an aggregate with the field types to construct the struct value.

        // Compile each of the values for field initialisers, calculate their indices and also
        // gather their types with which to make an aggregate.

        let mut inserted_values_indices = Vec::with_capacity(fields.len());
        let mut field_types = Vec::with_capacity(fields.len());
        for (insert_idx, struct_field) in fields.into_iter().enumerate() {
            let field_ty = struct_field.value.return_type;
            let insert_val = self.compile_expression(context, md_mgr, struct_field.value)?;
            if insert_val.is_diverging(context) {
                return Ok(insert_val);
            }
            inserted_values_indices.push((insert_val, insert_idx as u64));
            field_types.push(field_ty);
        }

        // Start with a temporary empty struct and then fill in the values.
        let aggregate = get_aggregate_for_types(context, &field_types)?;
        let temp_name = self.lexical_map.insert_anon();
        let struct_ptr = self
            .function
            .new_local_ptr(context, temp_name, Type::Struct(aggregate), false, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;
        let struct_ptr_ty = *struct_ptr.get_type(context);
        let agg_value = self
            .current_block
            .ins(context)
            .get_ptr(struct_ptr, struct_ptr_ty, 0)
            .add_metadatum(context, span_md_idx);

        Ok(inserted_values_indices.into_iter().fold(
            agg_value,
            |agg_value, (insert_val, insert_idx)| {
                self.current_block
                    .ins(context)
                    .insert_value(agg_value, aggregate, insert_val, vec![insert_idx])
                    .add_metadatum(context, span_md_idx)
            },
        ))
    }

    fn compile_struct_field_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_struct_expr: ty::TyExpression,
        struct_type_id: TypeId,
        ast_field: TyStructField,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let ast_struct_expr_span = ast_struct_expr.span.clone();
        let struct_val = self.compile_expression(context, md_mgr, ast_struct_expr)?;
        let aggregate = if let Some(instruction) = struct_val.get_instruction(context) {
            instruction.get_aggregate(context).ok_or_else(|| {
                    CompileError::InternalOwned(format!(
                        "Unsupported instruction as struct value for field expression. {instruction:?}"),
                        ast_struct_expr_span)
                })
        } else if let Some(Type::Struct(agg)) = struct_val.get_argument_type(context) {
            Ok(agg)
        } else if let Some(Constant {
            ty: Type::Struct(agg),
            ..
        }) = struct_val.get_constant(context)
        {
            Ok(*agg)
        } else {
            Err(CompileError::InternalOwned(
                "Unsupported struct value for field expression.".to_owned(),
                ast_struct_expr_span,
            ))
        }?;

        let field_kind = ProjectionKind::StructField {
            name: ast_field.name.clone(),
        };
        let field_idx = match get_struct_name_field_index_and_type(struct_type_id, field_kind) {
            None => Err(CompileError::Internal(
                "Unknown struct in field expression.",
                ast_field.span,
            )),
            Some((struct_name, field_idx_and_type_opt)) => match field_idx_and_type_opt {
                None => Err(CompileError::InternalOwned(
                    format!(
                        "Unknown field name '{}' for struct '{struct_name}' in field expression.",
                        ast_field.name
                    ),
                    ast_field.span,
                )),
                Some((field_idx, _field_type)) => Ok(field_idx),
            },
        }?;

        Ok(self
            .current_block
            .ins(context)
            .extract_value(struct_val, aggregate, vec![field_idx])
            .add_metadatum(context, span_md_idx))
    }

    fn compile_enum_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        enum_decl: TyEnumDeclaration,
        tag: usize,
        contents: Option<Box<ty::TyExpression>>,
    ) -> Result<Value, CompileError> {
        // XXX The enum instantiation AST node includes the full declaration.  If the enum was
        // declared in a different module then it seems for now there's no easy way to pre-analyse
        // it and add its type/aggregate to the context.  We can re-use them here if we recognise
        // the name, and if not add a new aggregate... OTOH the naming seems a little fragile and
        // we could potentially use the wrong aggregate with the same name, different module...
        // dunno.
        let span_md_idx = md_mgr.span_to_md(context, &enum_decl.span);
        let aggregate = create_enum_aggregate(context, enum_decl.variants)?;
        let tag_value =
            Constant::get_uint(context, 64, tag as u64).add_metadatum(context, span_md_idx);

        // Start with a temporary local struct and insert the tag.
        let temp_name = self.lexical_map.insert_anon();
        let enum_ptr = self
            .function
            .new_local_ptr(context, temp_name, Type::Struct(aggregate), false, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;
        let enum_ptr_ty = *enum_ptr.get_type(context);
        let enum_ptr_value = self
            .current_block
            .ins(context)
            .get_ptr(enum_ptr, enum_ptr_ty, 0)
            .add_metadatum(context, span_md_idx);
        let agg_value = self
            .current_block
            .ins(context)
            .insert_value(enum_ptr_value, aggregate, tag_value, vec![0])
            .add_metadatum(context, span_md_idx);

        // If the struct representing the enum has only one field, then that field is basically the
        // tag and all the variants must have unit types, hence the absence of the union.
        // Therefore, there is no need for another `insert_value` instruction here.
        match aggregate.get_content(context) {
            AggregateContent::FieldTypes(field_tys) => {
                Ok(if field_tys.len() == 1 {
                    agg_value
                } else {
                    match contents {
                        None => agg_value,
                        Some(te) => {
                            // Insert the value too.
                            let contents_value = self.compile_expression(context, md_mgr, *te)?;
                            self.current_block
                                .ins(context)
                                .insert_value(agg_value, aggregate, contents_value, vec![1])
                                .add_metadatum(context, span_md_idx)
                        }
                    }
                })
            }
            _ => unreachable!("Wrong content for struct."),
        }
    }

    fn compile_tuple_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        fields: Vec<ty::TyExpression>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        if fields.is_empty() {
            // This is a Unit.  We're still debating whether Unit should just be an empty tuple in
            // the IR or not... it is a special case for now.
            Ok(Constant::get_unit(context).add_metadatum(context, span_md_idx))
        } else {
            let mut init_values = Vec::with_capacity(fields.len());
            let mut init_types = Vec::with_capacity(fields.len());
            for field_expr in fields {
                let init_type = convert_resolved_typeid_no_span(context, &field_expr.return_type)?;
                let init_value = self.compile_expression(context, md_mgr, field_expr)?;
                if init_value.is_diverging(context) {
                    return Ok(init_value);
                }
                init_values.push(init_value);
                init_types.push(init_type);
            }

            let aggregate = Aggregate::new_struct(context, init_types);
            let temp_name = self.lexical_map.insert_anon();
            let tuple_ptr = self
                .function
                .new_local_ptr(context, temp_name, Type::Struct(aggregate), false, None)
                .map_err(|ir_error| {
                    CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                })?;
            let tuple_ptr_ty = *tuple_ptr.get_type(context);
            let agg_value = self
                .current_block
                .ins(context)
                .get_ptr(tuple_ptr, tuple_ptr_ty, 0)
                .add_metadatum(context, span_md_idx);

            Ok(init_values.into_iter().enumerate().fold(
                agg_value,
                |agg_value, (insert_idx, insert_val)| {
                    self.current_block
                        .ins(context)
                        .insert_value(agg_value, aggregate, insert_val, vec![insert_idx as u64])
                        .add_metadatum(context, span_md_idx)
                },
            ))
        }
    }

    fn compile_tuple_elem_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        tuple: ty::TyExpression,
        tuple_type: TypeId,
        idx: usize,
        span: Span,
    ) -> Result<Value, CompileError> {
        let tuple_value = self.compile_expression(context, md_mgr, tuple)?;
        if let Type::Struct(aggregate) = convert_resolved_typeid(context, &tuple_type, &span)? {
            let span_md_idx = md_mgr.span_to_md(context, &span);
            Ok(self
                .current_block
                .ins(context)
                .extract_value(tuple_value, aggregate, vec![idx as u64])
                .add_metadatum(context, span_md_idx))
        } else {
            Err(CompileError::Internal(
                "Invalid (non-aggregate?) tuple type for TupleElemAccess.",
                span,
            ))
        }
    }

    fn compile_storage_access(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        fields: &[ty::TyStorageAccessDescriptor],
        ix: &StateIndex,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // Get the type of the access which can be a subfield
        let access_type = convert_resolved_typeid_no_span(
            context,
            &fields.last().expect("guaranteed by grammar").type_id,
        )?;

        // Get the list of indices used to access the storage field. This will be empty
        // if the storage field type is not a struct.
        // FIXME: shouldn't have to extract the first field like this.
        let base_type = fields[0].type_id;
        let field_idcs = get_indices_for_struct_access(base_type, &fields[1..])?;

        // Do the actual work. This is a recursive function because we want to drill down
        // to load each primitive type in the storage field in its own storage slot.
        self.compile_storage_read(context, md_mgr, ix, &field_idcs, &access_type, span_md_idx)
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_asm_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        registers: Vec<ty::TyAsmRegisterDeclaration>,
        body: Vec<AsmOp>,
        return_type: TypeId,
        returns: Option<(AsmRegister, Span)>,
        whole_block_span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let registers = registers
            .into_iter()
            .map(
                |ty::TyAsmRegisterDeclaration {
                     initializer, name, ..
                 }| {
                    // Take the optional initialiser, map it to an Option<Result<Value>>,
                    // transpose that to Result<Option<Value>> and map that to an AsmArg.
                    initializer
                        .map(|init_expr| self.compile_expression(context, md_mgr, init_expr))
                        .transpose()
                        .map(|init| AsmArg {
                            name,
                            initializer: init,
                        })
                },
            )
            .collect::<Result<Vec<AsmArg>, CompileError>>()?;
        let body = body
            .into_iter()
            .map(
                |AsmOp {
                     op_name,
                     op_args,
                     immediate,
                     span,
                 }| AsmInstruction {
                    name: op_name,
                    args: op_args,
                    immediate,
                    metadata: md_mgr.span_to_md(context, &span),
                },
            )
            .collect();
        let returns = returns
            .as_ref()
            .map(|(_, asm_reg_span)| Ident::new(asm_reg_span.clone()));
        let return_type = convert_resolved_typeid_no_span(context, &return_type)?;
        Ok(self
            .current_block
            .ins(context)
            .asm_block(registers, body, return_type, returns)
            .add_metadatum(context, whole_block_span_md_idx))
    }

    fn compile_storage_read(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ix: &StateIndex,
        indices: &[u64],
        ty: &Type,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        match ty {
            Type::Struct(aggregate) => {
                let temp_name = self.lexical_map.insert_anon();
                let struct_ptr = self
                    .function
                    .new_local_ptr(context, temp_name, Type::Struct(*aggregate), false, None)
                    .map_err(|ir_error| {
                        CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                    })?;
                let struct_ptr_ty = *struct_ptr.get_type(context);
                let mut struct_val = self
                    .current_block
                    .ins(context)
                    .get_ptr(struct_ptr, struct_ptr_ty, 0)
                    .add_metadatum(context, span_md_idx);

                let fields = aggregate.get_content(context).field_types().clone();
                for (field_idx, field_type) in fields.into_iter().enumerate() {
                    let field_idx = field_idx as u64;

                    // Recurse. The base case is for primitive types that fit in a single storage slot.
                    let mut new_indices = indices.to_owned();
                    new_indices.push(field_idx);

                    let val_to_insert = self.compile_storage_read(
                        context,
                        md_mgr,
                        ix,
                        &new_indices,
                        &field_type,
                        span_md_idx,
                    )?;

                    //  Insert the loaded value to the aggregate at the given index
                    struct_val = self
                        .current_block
                        .ins(context)
                        .insert_value(struct_val, *aggregate, val_to_insert, vec![field_idx])
                        .add_metadatum(context, span_md_idx);
                }
                Ok(struct_val)
            }
            _ => {
                let storage_key = get_storage_key(ix, indices);

                // New name for the key
                let mut key_name = format!("{}{}", "key_for_", ix.to_usize());
                for ix in indices {
                    key_name = format!("{}_{}", key_name, ix);
                }
                let alias_key_name = self.lexical_map.insert(key_name.as_str().to_owned());

                // Local pointer for the key
                let key_ptr = self
                    .function
                    .new_local_ptr(context, alias_key_name, Type::B256, true, None)
                    .map_err(|ir_error| {
                        CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                    })?;

                // Const value for the key from the hash
                let const_key =
                    convert_literal_to_value(context, &Literal::B256(storage_key.into()))
                        .add_metadatum(context, span_md_idx);

                // Convert the key pointer to a value using get_ptr
                let key_ptr_ty = *key_ptr.get_type(context);
                let mut key_ptr_val = self
                    .current_block
                    .ins(context)
                    .get_ptr(key_ptr, key_ptr_ty, 0)
                    .add_metadatum(context, span_md_idx);

                // Store the const hash value to the key pointer value
                self.current_block
                    .ins(context)
                    .store(key_ptr_val, const_key)
                    .add_metadatum(context, span_md_idx);

                match ty {
                    Type::Array(_) => Err(CompileError::Internal(
                        "Arrays in storage have not been implemented yet.",
                        Span::dummy(),
                    )),
                    Type::Pointer(_) => Err(CompileError::Internal(
                        "Pointers in storage have not been implemented yet.",
                        Span::dummy(),
                    )),
                    Type::B256 => self.compile_b256_storage_read(
                        context,
                        ix,
                        indices,
                        &key_ptr_val,
                        span_md_idx,
                    ),
                    Type::Bool | Type::Uint(_) => self.compile_uint_or_bool_storage_read(
                        context,
                        &key_ptr_val,
                        ty,
                        span_md_idx,
                    ),
                    Type::String(_) | Type::Union(_) => self.compile_union_or_string_storage_read(
                        context,
                        ix,
                        indices,
                        &mut key_ptr_val,
                        &key_ptr,
                        &storage_key,
                        ty,
                        span_md_idx,
                    ),
                    Type::Struct(_) => unreachable!("structs are already handled!"),
                    Type::Unit => {
                        Ok(Constant::get_unit(context).add_metadatum(context, span_md_idx))
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_storage_write(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ix: &StateIndex,
        indices: &[u64],
        ty: &Type,
        rhs: Value,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<(), CompileError> {
        match ty {
            Type::Struct(aggregate) => {
                let fields = aggregate.get_content(context).field_types().clone();
                for (field_idx, field_type) in fields.into_iter().enumerate() {
                    let field_idx = field_idx as u64;

                    // Recurse. The base case is for primitive types that fit in a single storage slot.
                    let mut new_indices = indices.to_owned();
                    new_indices.push(field_idx);

                    // Extract the value from the aggregate at the given index
                    let rhs = self
                        .current_block
                        .ins(context)
                        .extract_value(rhs, *aggregate, vec![field_idx])
                        .add_metadatum(context, span_md_idx);

                    self.compile_storage_write(
                        context,
                        md_mgr,
                        ix,
                        &new_indices,
                        &field_type,
                        rhs,
                        span_md_idx,
                    )?;
                }
                Ok(())
            }
            _ => {
                let storage_key = get_storage_key(ix, indices);

                // New name for the key
                let mut key_name = format!("{}{}", "key_for_", ix.to_usize());
                for ix in indices {
                    key_name = format!("{}_{}", key_name, ix);
                }
                let alias_key_name = self.lexical_map.insert(key_name.as_str().to_owned());

                // Local pointer for the key
                let key_ptr = self
                    .function
                    .new_local_ptr(context, alias_key_name, Type::B256, true, None)
                    .map_err(|ir_error| {
                        CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                    })?;

                // Const value for the key from the hash
                let const_key =
                    convert_literal_to_value(context, &Literal::B256(storage_key.into()))
                        .add_metadatum(context, span_md_idx);

                // Convert the key pointer to a value using get_ptr
                let key_ptr_ty = *key_ptr.get_type(context);
                let mut key_ptr_val = self
                    .current_block
                    .ins(context)
                    .get_ptr(key_ptr, key_ptr_ty, 0)
                    .add_metadatum(context, span_md_idx);

                // Store the const hash value to the key pointer value
                self.current_block
                    .ins(context)
                    .store(key_ptr_val, const_key)
                    .add_metadatum(context, span_md_idx);

                match ty {
                    Type::Array(_) => Err(CompileError::Internal(
                        "Arrays in storage have not been implemented yet.",
                        Span::dummy(),
                    )),
                    Type::Pointer(_) => Err(CompileError::Internal(
                        "Pointers in storage have not been implemented yet.",
                        Span::dummy(),
                    )),
                    Type::B256 => self.compile_b256_storage_write(
                        context,
                        ix,
                        indices,
                        &key_ptr_val,
                        rhs,
                        span_md_idx,
                    ),
                    Type::Bool | Type::Uint(_) => self.compile_uint_or_bool_storage_write(
                        context,
                        &key_ptr_val,
                        rhs,
                        span_md_idx,
                    ),
                    Type::String(_) | Type::Union(_) => self.compile_union_or_string_storage_write(
                        context,
                        ix,
                        indices,
                        &mut key_ptr_val,
                        &key_ptr,
                        &storage_key,
                        ty,
                        rhs,
                        span_md_idx,
                    ),
                    Type::Struct(_) => unreachable!("structs are already handled!"),
                    Type::Unit => Ok(()),
                }
            }
        }
    }

    fn compile_uint_or_bool_storage_read(
        &mut self,
        context: &mut Context,
        key_ptr_val: &Value,
        ty: &Type,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // `state_load_word` always returns a `u64`. Cast the result back
        // to the right type before returning
        let load_val = self
            .current_block
            .ins(context)
            .state_load_word(*key_ptr_val)
            .add_metadatum(context, span_md_idx);
        let val = self
            .current_block
            .ins(context)
            .bitcast(load_val, *ty)
            .add_metadatum(context, span_md_idx);
        Ok(val)
    }

    fn compile_uint_or_bool_storage_write(
        &mut self,
        context: &mut Context,
        key_ptr_val: &Value,
        rhs: Value,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<(), CompileError> {
        // `state_store_word` requires a `u64`. Cast the value to store to
        // `u64` first before actually storing.
        let rhs_u64 = self
            .current_block
            .ins(context)
            .bitcast(rhs, Type::Uint(64))
            .add_metadatum(context, span_md_idx);
        self.current_block
            .ins(context)
            .state_store_word(rhs_u64, *key_ptr_val)
            .add_metadatum(context, span_md_idx);
        Ok(())
    }

    fn compile_b256_storage_read(
        &mut self,
        context: &mut Context,
        ix: &StateIndex,
        indices: &[u64],
        key_ptr_val: &Value,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // B256 requires 4 words. Use state_load_quad_word/state_store_quad_word
        // First, create a name for the value to load from or store to
        let mut value_name = format!("{}{}", "val_for_", ix.to_usize());
        for ix in indices {
            value_name = format!("{}_{}", value_name, ix);
        }
        let alias_value_name = self.lexical_map.insert(value_name.as_str().to_owned());

        // Local pointer to hold the B256
        let value_ptr = self
            .function
            .new_local_ptr(context, alias_value_name, Type::B256, true, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        // Convert the local pointer created to a value using get_ptr
        let value_ptr_val = self
            .current_block
            .ins(context)
            .get_ptr(value_ptr, Type::B256, 0)
            .add_metadatum(context, span_md_idx);

        self.current_block
            .ins(context)
            .state_load_quad_word(value_ptr_val, *key_ptr_val)
            .add_metadatum(context, span_md_idx);
        Ok(value_ptr_val)
    }

    fn compile_b256_storage_write(
        &mut self,
        context: &mut Context,
        ix: &StateIndex,
        indices: &[u64],
        key_ptr_val: &Value,
        rhs: Value,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<(), CompileError> {
        // B256 requires 4 words. Use state_load_quad_word/state_store_quad_word
        // First, create a name for the value to load from or store to
        let mut value_name = format!("{}{}", "val_for_", ix.to_usize());
        for ix in indices {
            value_name = format!("{}_{}", value_name, ix);
        }
        let alias_value_name = self.lexical_map.insert(value_name.as_str().to_owned());

        // Local pointer to hold the B256
        let value_ptr = self
            .function
            .new_local_ptr(context, alias_value_name, Type::B256, true, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        // Convert the local pointer created to a value using get_ptr
        let value_ptr_val = self
            .current_block
            .ins(context)
            .get_ptr(value_ptr, Type::B256, 0)
            .add_metadatum(context, span_md_idx);

        // Store the value to the local pointer created for rhs
        self.current_block
            .ins(context)
            .store(value_ptr_val, rhs)
            .add_metadatum(context, span_md_idx);

        // Finally, just call state_load_quad_word/state_store_quad_word
        self.current_block
            .ins(context)
            .state_store_quad_word(value_ptr_val, *key_ptr_val)
            .add_metadatum(context, span_md_idx);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_union_or_string_storage_read(
        &mut self,
        context: &mut Context,
        ix: &StateIndex,
        indices: &[u64],
        key_ptr_val: &mut Value,
        key_ptr: &Pointer,
        storage_key: &fuel_types::Bytes32,
        r#type: &Type,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // Use state_load_quad_word/state_store_quad_word as many times as needed
        // using sequential keys

        // First, create a name for the value to load from or store to
        let value_name = format!(
            "val_for_{}{}",
            ix.to_usize(),
            indices
                .iter()
                .map(|idx| format!("_{idx}"))
                .collect::<Vec<_>>()
                .join("")
        );
        let alias_value_name = self.lexical_map.insert(value_name);

        // Create an array of `b256` that will hold the value to store into storage
        // or the value loaded from storage. The array has to fit the whole type.
        let number_of_elements = (ir_type_size_in_bytes(context, r#type) + 31) / 32;
        let b256_array_type = Type::Array(Aggregate::new_array(
            context,
            Type::B256,
            number_of_elements,
        ));

        // Local pointer to hold the array of b256s
        let value_ptr = self
            .function
            .new_local_ptr(context, alias_value_name, b256_array_type, true, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        // Convert the local pointer created to a value of the original type using
        // get_ptr.
        let value_ptr_val = self
            .current_block
            .ins(context)
            .get_ptr(value_ptr, *r#type, 0)
            .add_metadatum(context, span_md_idx);

        for array_index in 0..number_of_elements {
            if array_index > 0 {
                // Prepare key for the next iteration but not for array index 0
                // because the first key was generated earlier.
                // Const value for the key from the initial hash + array_index
                let const_key = convert_literal_to_value(
                    context,
                    &Literal::B256(*add_to_b256(*storage_key, array_index)),
                )
                .add_metadatum(context, span_md_idx);

                // Convert the key pointer to a value using get_ptr
                let key_ptr_ty = *key_ptr.get_type(context);
                *key_ptr_val = self
                    .current_block
                    .ins(context)
                    .get_ptr(*key_ptr, key_ptr_ty, 0)
                    .add_metadatum(context, span_md_idx);

                // Store the const hash value to the key pointer value
                self.current_block
                    .ins(context)
                    .store(*key_ptr_val, const_key)
                    .add_metadatum(context, span_md_idx);
            }

            // Get the b256 from the array at index iter
            let value_ptr_val_b256 = self
                .current_block
                .ins(context)
                .get_ptr(value_ptr, Type::B256, array_index)
                .add_metadatum(context, span_md_idx);

            self.current_block
                .ins(context)
                .state_load_quad_word(value_ptr_val_b256, *key_ptr_val)
                .add_metadatum(context, span_md_idx);
        }
        Ok(value_ptr_val)
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_union_or_string_storage_write(
        &mut self,
        context: &mut Context,
        ix: &StateIndex,
        indices: &[u64],
        key_ptr_val: &mut Value,
        key_ptr: &Pointer,
        storage_key: &fuel_types::Bytes32,
        r#type: &Type,
        rhs: Value,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<(), CompileError> {
        // Use state_load_quad_word/state_store_quad_word as many times as needed
        // using sequential keys

        // First, create a name for the value to load from or store to
        let value_name = format!(
            "val_for_{}{}",
            ix.to_usize(),
            indices
                .iter()
                .map(|idx| format!("_{idx}"))
                .collect::<Vec<_>>()
                .join("")
        );
        let alias_value_name = self.lexical_map.insert(value_name);

        // Create an array of `b256` that will hold the value to store into storage
        // or the value loaded from storage. The array has to fit the whole type.
        let number_of_elements = (ir_type_size_in_bytes(context, r#type) + 31) / 32;
        let b256_array_type = Type::Array(Aggregate::new_array(
            context,
            Type::B256,
            number_of_elements,
        ));

        // Local pointer to hold the array of b256s
        let value_ptr = self
            .function
            .new_local_ptr(context, alias_value_name, b256_array_type, true, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        // Convert the local pointer created to a value of the original type using
        // get_ptr.
        let value_ptr_val = self
            .current_block
            .ins(context)
            .get_ptr(value_ptr, *r#type, 0)
            .add_metadatum(context, span_md_idx);

        // Store the value to the local pointer created for rhs
        self.current_block
            .ins(context)
            .store(value_ptr_val, rhs)
            .add_metadatum(context, span_md_idx);

        for array_index in 0..number_of_elements {
            if array_index > 0 {
                // Prepare key for the next iteration but not for array index 0
                // because the first key was generated earlier.
                // Const value for the key from the initial hash + array_index
                let const_key = convert_literal_to_value(
                    context,
                    &Literal::B256(*add_to_b256(*storage_key, array_index)),
                )
                .add_metadatum(context, span_md_idx);

                // Convert the key pointer to a value using get_ptr
                let key_ptr_ty = *key_ptr.get_type(context);
                *key_ptr_val = self
                    .current_block
                    .ins(context)
                    .get_ptr(*key_ptr, key_ptr_ty, 0)
                    .add_metadatum(context, span_md_idx);

                // Store the const hash value to the key pointer value
                self.current_block
                    .ins(context)
                    .store(*key_ptr_val, const_key)
                    .add_metadatum(context, span_md_idx);
            }

            // Get the b256 from the array at index iter
            let value_ptr_val_b256 = self
                .current_block
                .ins(context)
                .get_ptr(value_ptr, Type::B256, array_index)
                .add_metadatum(context, span_md_idx);

            // Finally, just call state_load_quad_word/state_store_quad_word
            self.current_block
                .ins(context)
                .state_store_quad_word(value_ptr_val_b256, *key_ptr_val)
                .add_metadatum(context, span_md_idx);
        }

        Ok(())
    }
}
