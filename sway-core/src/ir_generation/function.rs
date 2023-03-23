use super::{
    compile::compile_function, convert::*, lexical_map::LexicalMap, storage::get_storage_key,
    types::*,
};
use crate::{
    asm_generation::from_ir::ir_type_size_in_bytes,
    decl_engine::DeclEngine,
    engine_threading::*,
    ir_generation::const_eval::{
        compile_constant_expression, compile_constant_expression_to_constant,
    },
    language::{
        ty::{self, ProjectionKind},
        *,
    },
    metadata::MetadataManager,
    type_system::*,
    types::*,
};
use sway_ast::intrinsics::Intrinsic;
use sway_error::error::{CompileError, Hint};
use sway_ir::{Context, *};
use sway_types::{
    constants,
    ident::Ident,
    span::{Span, Spanned},
    state::StateIndex,
};

use std::collections::HashMap;

/// Engine for compiling a function and all of the AST nodes within.
///
/// This is mostly recursively compiling expressions, as Sway is fairly heavily expression based.
///
/// The rule here is to use compile_expression() when a value is desired, as opposed to a pointer.
/// This is most of the time, as we try to be target agnostic and not make assuptions about which
/// values must be used by reference.
///
/// compile_expression() will force the result to be a value, by using a temporary if necessary.
///
/// compile_ptr_expression() will compile the expression without forcing anything.  If the
/// expression has a reference type, like getting a struct or an explicit ref arg, it will return a
/// pointer value, but otherwise will return a value.
///
/// compile_ptr_or_tmp_expression() will compile the expression and force it to be a pointer, also
/// by using a temporary if necessary.  This can be slightly dangerous, if the reference is
/// supposed to be to a particular value but is accidentally made to a temporary value then
/// mutations or other side-effects might not be applied in the correct context.
///
/// So in general the methods in FnCompiler will return a pointer if they can and will get it be
/// forced into a value if that is desired.  All the temporary values are manipulated with simple
/// loads and stores, rather than anything more complicated like mem_copys.

pub(crate) struct FnCompiler<'eng> {
    type_engine: &'eng TypeEngine,
    decl_engine: &'eng DeclEngine,
    module: Module,
    pub(super) function: Function,
    pub(super) current_block: Block,
    block_to_break_to: Option<Block>,
    block_to_continue_to: Option<Block>,
    current_fn_param: Option<ty::TyFunctionParameter>,
    lexical_map: LexicalMap,
    recreated_fns: HashMap<(Span, Vec<TypeId>, Vec<TypeId>), Function>,
    // This is a map from the type IDs of a logged type and the ID of the corresponding log
    logged_types_map: HashMap<TypeId, LogId>,
    // This is a map from the type IDs of a message data type and the ID of the corresponding smo
    messages_types_map: HashMap<TypeId, MessageId>,
}

impl<'eng> FnCompiler<'eng> {
    pub(super) fn new(
        engines: Engines<'eng>,
        context: &mut Context,
        module: Module,
        function: Function,
        logged_types_map: &HashMap<TypeId, LogId>,
        messages_types_map: &HashMap<TypeId, MessageId>,
    ) -> Self {
        let (type_engine, decl_engine) = engines.unwrap();
        let lexical_map = LexicalMap::from_iter(
            function
                .args_iter(context)
                .map(|(name, _value)| name.clone()),
        );
        FnCompiler {
            type_engine,
            decl_engine,
            module,
            function,
            current_block: function.get_entry_block(context),
            block_to_break_to: None,
            block_to_continue_to: None,
            lexical_map,
            recreated_fns: HashMap::new(),
            current_fn_param: None,
            logged_types_map: logged_types_map.clone(),
            messages_types_map: messages_types_map.clone(),
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
        ast_block: &ty::TyCodeBlock,
    ) -> Result<Value, CompileError> {
        self.compile_with_new_scope(|fn_compiler| {
            let mut ast_nodes = ast_block.contents.iter();
            loop {
                let ast_node = match ast_nodes.next() {
                    Some(ast_node) => ast_node,
                    None => break Ok(Constant::get_unit(context)),
                };
                match fn_compiler.compile_ast_node(context, md_mgr, ast_node) {
                    Ok(Some(val)) => break Ok(val),
                    Ok(None) => (),
                    Err(err) => break Err(err),
                }
            }
        })
    }

    fn compile_ast_node(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_node: &ty::TyAstNode,
    ) -> Result<Option<Value>, CompileError> {
        let unexpected_decl = |decl_type: &'static str| {
            Err(CompileError::UnexpectedDeclaration {
                decl_type,
                span: ast_node.span.clone(),
            })
        };

        let span_md_idx = md_mgr.span_to_md(context, &ast_node.span);
        match &ast_node.content {
            ty::TyAstNodeContent::Declaration(td) => match td {
                ty::TyDecl::VariableDecl(tvd) => {
                    self.compile_var_decl(context, md_mgr, tvd, span_md_idx)
                }
                ty::TyDecl::ConstantDecl { decl_id, .. } => {
                    let tcd = self.decl_engine.get_constant(decl_id);
                    self.compile_const_decl(context, md_mgr, tcd, span_md_idx)?;
                    Ok(None)
                }
                ty::TyDecl::EnumDecl { decl_id, .. } => {
                    let ted = self.decl_engine.get_enum(decl_id);
                    create_tagged_union_type(
                        self.type_engine,
                        self.decl_engine,
                        context,
                        &ted.variants,
                    )
                    .map(|_| ())?;
                    Ok(None)
                }
                ty::TyDecl::TypeAliasDecl { .. } => {
                    Err(CompileError::UnexpectedDeclaration {
                        decl_type: "type alias",
                        span: ast_node.span.clone(),
                    })
                }
                ty::TyDecl::ImplTrait { .. } => {
                    // XXX What if we ignore the trait implementation???  Potentially since
                    // we currently inline everything and below we 'recreate' the functions
                    // lazily as they are called, nothing needs to be done here.  BUT!
                    // This is obviously not really correct, and eventually we want to
                    // compile and then call these properly.
                    Ok(None)
                }
                ty::TyDecl::FunctionDecl { .. } => unexpected_decl("function"),
                ty::TyDecl::TraitDecl { .. } => unexpected_decl("trait"),
                ty::TyDecl::StructDecl { .. } => unexpected_decl("struct"),
                ty::TyDecl::AbiDecl { .. } => unexpected_decl("abi"),
                ty::TyDecl::GenericTypeForFunctionScope { .. } => {
                    unexpected_decl("generic type")
                }
                ty::TyDecl::ErrorRecovery { .. } => unexpected_decl("error recovery"),
                ty::TyDecl::StorageDecl { .. } => unexpected_decl("storage"),
            },
            ty::TyAstNodeContent::Expression(te) => {
                // An expression with an ignored return value... I assume.
                let value = self.compile_expression(context, md_mgr, te)?;
                if value.is_diverging(context) {
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }
            ty::TyAstNodeContent::ImplicitReturnExpression(te) => {
                self.compile_expression(context, md_mgr, te).map(Some)
            }
            // a side effect can be () because it just impacts the type system/namespacing.
            // There should be no new IR generated.
            ty::TyAstNodeContent::SideEffect(_) => Ok(None),
        }
    }

    fn compile_expression(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_expr: &ty::TyExpression,
    ) -> Result<Value, CompileError> {
        // Compile expression which *may* be a pointer.  We can't return a pointer value here
        // though, so add a `load` to it.
        self.compile_ptr_expression(context, md_mgr, ast_expr)
            .map(|val| {
                if val.get_type(context).map_or(false, |ty| ty.is_ptr(context)) {
                    self.current_block.ins(context).load(val)
                } else {
                    val
                }
            })
    }

    fn compile_ptr_or_tmp_expression(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_expr: &ty::TyExpression,
    ) -> Result<Value, CompileError> {
        // Compile expression which *may* be a pointer.  We can't return a value so create a
        // temporary here, store the value and return its pointer.
        let val = self.compile_ptr_expression(context, md_mgr, ast_expr)?;
        let ty = match val.get_type(context) {
            Some(ty) if !ty.is_ptr(context) => ty,
            _ => return Ok(val),
        };

        // Create a temporary.
        let temp_name = self.lexical_map.insert_anon();
        let tmp_var = self
            .function
            .new_local_var(context, temp_name, ty, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;
        let tmp_val = self.current_block.ins(context).get_local(tmp_var);
        self.current_block.ins(context).store(tmp_val, val);

        Ok(tmp_val)
    }

    fn compile_ptr_expression(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_expr: &ty::TyExpression,
    ) -> Result<Value, CompileError> {
        let span_md_idx = md_mgr.span_to_md(context, &ast_expr.span);
        match &ast_expr.expression {
            ty::TyExpressionVariant::Literal(l) => {
                Ok(convert_literal_to_value(context, l).add_metadatum(context, span_md_idx))
            }
            ty::TyExpressionVariant::FunctionApplication {
                call_path: name,
                contract_call_params,
                arguments,
                fn_ref,
                self_state_idx,
                selector,
                type_binding: _,
            } => {
                if let Some(metadata) = selector {
                    self.compile_contract_call(
                        context,
                        md_mgr,
                        metadata,
                        contract_call_params,
                        name.suffix.as_str(),
                        arguments,
                        ast_expr.return_type,
                        span_md_idx,
                    )
                } else {
                    let function_decl = self.decl_engine.get_function(fn_ref);
                    self.compile_fn_call(
                        context,
                        md_mgr,
                        arguments,
                        &function_decl,
                        *self_state_idx,
                        span_md_idx,
                    )
                }
            }
            ty::TyExpressionVariant::LazyOperator { op, lhs, rhs } => {
                self.compile_lazy_op(context, md_mgr, op, lhs, rhs, span_md_idx)
            }
            ty::TyExpressionVariant::VariableExpression {
                name, call_path, ..
            } => self.compile_var_expr(context, call_path, name, span_md_idx),
            ty::TyExpressionVariant::Array {
                elem_type,
                contents,
            } => self.compile_array_expr(context, md_mgr, elem_type, contents, span_md_idx),
            ty::TyExpressionVariant::ArrayIndex { prefix, index } => {
                self.compile_array_index(context, md_mgr, prefix, index, span_md_idx)
            }
            ty::TyExpressionVariant::StructExpression { fields, .. } => {
                self.compile_struct_expr(context, md_mgr, fields, span_md_idx)
            }
            ty::TyExpressionVariant::CodeBlock(cb) => self.compile_code_block(context, md_mgr, cb),
            ty::TyExpressionVariant::FunctionParameter => Err(CompileError::Internal(
                "Unexpected function parameter declaration.",
                ast_expr.span.clone(),
            )),
            ty::TyExpressionVariant::MatchExp { desugared, .. } => {
                self.compile_expression(context, md_mgr, desugared)
            }
            ty::TyExpressionVariant::IfExp {
                condition,
                then,
                r#else,
            } => self.compile_if(
                context,
                md_mgr,
                condition,
                then,
                r#else.as_deref(),
                ast_expr.return_type,
            ),
            ty::TyExpressionVariant::AsmExpression {
                registers,
                body,
                returns,
                whole_block_span,
            } => {
                let span_md_idx = md_mgr.span_to_md(context, whole_block_span);
                self.compile_asm_expr(
                    context,
                    md_mgr,
                    registers,
                    body,
                    ast_expr.return_type,
                    returns.as_ref(),
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
                    prefix,
                    *resolved_type_of_parent,
                    field_to_access,
                    span_md_idx,
                )
            }
            ty::TyExpressionVariant::EnumInstantiation {
                enum_ref,
                tag,
                contents,
                ..
            } => {
                let enum_decl = self.decl_engine.get_enum(enum_ref);
                self.compile_enum_expr(context, md_mgr, &enum_decl, *tag, contents.as_deref())
            }
            ty::TyExpressionVariant::Tuple { fields } => {
                self.compile_tuple_expr(context, md_mgr, fields, span_md_idx)
            }
            ty::TyExpressionVariant::TupleElemAccess {
                prefix,
                elem_to_access_num: idx,
                elem_to_access_span: span,
                resolved_type_of_parent: tuple_type,
            } => self.compile_tuple_elem_expr(
                context,
                md_mgr,
                prefix,
                *tuple_type,
                *idx,
                span.clone(),
            ),
            ty::TyExpressionVariant::AbiCast { span, .. } => {
                let span_md_idx = md_mgr.span_to_md(context, span);
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
                self.compile_intrinsic_function(context, md_mgr, kind, ast_expr.span.clone())
            }
            ty::TyExpressionVariant::AbiName(_) => {
                Ok(Value::new_constant(context, Constant::new_unit(context)))
            }
            ty::TyExpressionVariant::UnsafeDowncast { exp, variant } => {
                self.compile_unsafe_downcast(context, md_mgr, exp, variant)
            }
            ty::TyExpressionVariant::EnumTag { exp } => {
                self.compile_enum_tag(context, md_mgr, exp.to_owned())
            }
            ty::TyExpressionVariant::WhileLoop { body, condition } => self.compile_while_loop(
                context,
                md_mgr,
                body,
                condition,
                span_md_idx,
                ast_expr.span.clone(),
            ),
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
                        span: ast_expr.span.clone(),
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
                    span: ast_expr.span.clone(),
                }),
            },
            ty::TyExpressionVariant::Reassignment(reassignment) => {
                self.compile_reassignment(context, md_mgr, reassignment, span_md_idx)
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
                self.compile_return_statement(context, md_mgr, exp)
            }
        }
    }

    fn compile_intrinsic_function(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ty::TyIntrinsicFunctionKind {
            kind,
            arguments,
            type_arguments,
            span: _,
        }: &ty::TyIntrinsicFunctionKind,
        span: Span,
    ) -> Result<Value, CompileError> {
        fn store_key_in_local_mem(
            compiler: &mut FnCompiler,
            context: &mut Context,
            value: Value,
            span_md_idx: Option<MetadataIndex>,
        ) -> Result<Value, CompileError> {
            // New name for the key
            let key_name = compiler.lexical_map.insert("key_for_storage".to_owned());

            // Local variable for the key
            let key_var = compiler
                .function
                .new_local_var(context, key_name, Type::get_b256(context), None)
                .map_err(|ir_error| {
                    CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                })?;

            // Convert the key variable to a value using get_local.
            let key_val = compiler
                .current_block
                .ins(context)
                .get_local(key_var)
                .add_metadatum(context, span_md_idx);

            // Store the value to the key pointer value
            compiler
                .current_block
                .ins(context)
                .store(key_val, value)
                .add_metadatum(context, span_md_idx);
            Ok(key_val)
        }

        let engines = Engines::new(self.type_engine, self.decl_engine);

        // We safely index into arguments and type_arguments arrays below
        // because the type-checker ensures that the arguments are all there.
        match kind {
            Intrinsic::SizeOfVal => {
                let exp = &arguments[0];
                // Compile the expression in case of side-effects but ignore its value.
                let ir_type = convert_resolved_typeid(
                    self.type_engine,
                    self.decl_engine,
                    context,
                    &exp.return_type,
                    &exp.span,
                )?;
                self.compile_expression(context, md_mgr, exp)?;
                Ok(Constant::get_uint(
                    context,
                    64,
                    ir_type_size_in_bytes(context, &ir_type),
                ))
            }
            Intrinsic::SizeOfType => {
                let targ = type_arguments[0].clone();
                let ir_type = convert_resolved_typeid(
                    self.type_engine,
                    self.decl_engine,
                    context,
                    &targ.type_id,
                    &targ.span,
                )?;
                Ok(Constant::get_uint(
                    context,
                    64,
                    ir_type_size_in_bytes(context, &ir_type),
                ))
            }
            Intrinsic::IsReferenceType => {
                let targ = type_arguments[0].clone();
                let val = !self.type_engine.get_unaliased(targ.type_id).is_copy_type();
                Ok(Constant::get_bool(context, val))
            }
            Intrinsic::GetStorageKey => {
                let span_md_idx = md_mgr.span_to_md(context, &span);
                Ok(self
                    .current_block
                    .ins(context)
                    .get_storage_key()
                    .add_metadatum(context, span_md_idx))
            }
            Intrinsic::Eq | Intrinsic::Gt | Intrinsic::Lt => {
                let lhs = &arguments[0];
                let rhs = &arguments[1];
                let lhs_value = self.compile_expression(context, md_mgr, lhs)?;
                let rhs_value = self.compile_expression(context, md_mgr, rhs)?;
                let pred = match kind {
                    Intrinsic::Eq => Predicate::Equal,
                    Intrinsic::Gt => Predicate::GreaterThan,
                    Intrinsic::Lt => Predicate::LessThan,
                    _ => unreachable!(),
                };
                Ok(self
                    .current_block
                    .ins(context)
                    .cmp(pred, lhs_value, rhs_value))
            }
            Intrinsic::Gtf => {
                // The index is just a Value
                let index = self.compile_expression(context, md_mgr, &arguments[0])?;

                // The tx field ID has to be a compile-time constant because it becomes an
                // immediate
                let tx_field_id_constant = compile_constant_expression_to_constant(
                    engines,
                    context,
                    md_mgr,
                    self.module,
                    None,
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
                let target_type = &type_arguments[0];
                let target_ir_type = convert_resolved_typeid(
                    self.type_engine,
                    self.decl_engine,
                    context,
                    &target_type.type_id,
                    &target_type.span,
                )?;

                let span_md_idx = md_mgr.span_to_md(context, &span);

                // The `gtf` instruction
                let gtf_reg = self
                    .current_block
                    .ins(context)
                    .gtf(index, tx_field_id)
                    .add_metadatum(context, span_md_idx);

                // Reinterpret the result of the `gtf` instruction (which is always `u64`) as type
                // `T`. This requires an `int_to_ptr` instruction if `T` is a reference type.
                if self
                    .type_engine
                    .get_unaliased(target_type.type_id)
                    .is_copy_type()
                {
                    Ok(gtf_reg)
                } else {
                    let ptr_ty = Type::new_ptr(context, target_ir_type);
                    Ok(self
                        .current_block
                        .ins(context)
                        .int_to_ptr(gtf_reg, ptr_ty)
                        .add_metadatum(context, span_md_idx))
                }
            }
            Intrinsic::AddrOf => {
                let exp = &arguments[0];
                let value = self.compile_ptr_expression(context, md_mgr, exp)?;
                let int_ty = Type::new_uint(context, 64);
                let span_md_idx = md_mgr.span_to_md(context, &span);
                Ok(self
                    .current_block
                    .ins(context)
                    .ptr_to_int(value, int_ty)
                    .add_metadatum(context, span_md_idx))
            }
            Intrinsic::StateClear => {
                let key_exp = arguments[0].clone();
                let number_of_slots_exp = arguments[1].clone();
                let key_value = self.compile_expression(context, md_mgr, &key_exp)?;
                let number_of_slots_value =
                    self.compile_expression(context, md_mgr, &number_of_slots_exp)?;
                let span_md_idx = md_mgr.span_to_md(context, &span);
                let key_var = store_key_in_local_mem(self, context, key_value, span_md_idx)?;
                Ok(self
                    .current_block
                    .ins(context)
                    .state_clear(key_var, number_of_slots_value)
                    .add_metadatum(context, span_md_idx))
            }
            Intrinsic::StateLoadWord => {
                let exp = &arguments[0];
                let value = self.compile_expression(context, md_mgr, exp)?;
                let span_md_idx = md_mgr.span_to_md(context, &span);
                let key_var = store_key_in_local_mem(self, context, value, span_md_idx)?;
                Ok(self
                    .current_block
                    .ins(context)
                    .state_load_word(key_var)
                    .add_metadatum(context, span_md_idx))
            }
            Intrinsic::StateStoreWord => {
                let key_exp = &arguments[0];
                let val_exp = &arguments[1];
                // Validate that the val_exp is of the right type. We couldn't do it
                // earlier during type checking as the type arguments may not have been resolved.
                let val_ty = self.type_engine.get_unaliased(val_exp.return_type);
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
                let key_var = store_key_in_local_mem(self, context, key_value, span_md_idx)?;
                Ok(self
                    .current_block
                    .ins(context)
                    .state_store_word(val_value, key_var)
                    .add_metadatum(context, span_md_idx))
            }
            Intrinsic::StateLoadQuad | Intrinsic::StateStoreQuad => {
                let key_exp = arguments[0].clone();
                let val_exp = arguments[1].clone();
                let number_of_slots_exp = arguments[2].clone();
                // Validate that the val_exp is of the right type. We couldn't do it
                // earlier during type checking as the type arguments may not have been resolved.
                let val_ty = self.type_engine.get_unaliased(val_exp.return_type);
                if !val_ty.eq(&TypeInfo::RawUntypedPtr, engines) {
                    return Err(CompileError::IntrinsicUnsupportedArgType {
                        name: kind.to_string(),
                        span,
                        hint: Hint::new("This argument must be raw_ptr".to_string()),
                    });
                }
                let key_value = self.compile_expression(context, md_mgr, &key_exp)?;
                let val_value = self.compile_expression(context, md_mgr, &val_exp)?;
                let number_of_slots_value =
                    self.compile_expression(context, md_mgr, &number_of_slots_exp)?;
                let span_md_idx = md_mgr.span_to_md(context, &span);
                let key_var = store_key_in_local_mem(self, context, key_value, span_md_idx)?;
                let b256_ty = Type::get_b256(context);
                let b256_ptr_ty = Type::new_ptr(context, b256_ty);
                // For quad word, the IR instructions take in a pointer rather than a raw u64.
                let val_ptr = self
                    .current_block
                    .ins(context)
                    .int_to_ptr(val_value, b256_ptr_ty)
                    .add_metadatum(context, span_md_idx);
                match kind {
                    Intrinsic::StateLoadQuad => Ok(self
                        .current_block
                        .ins(context)
                        .state_load_quad_word(val_ptr, key_var, number_of_slots_value)
                        .add_metadatum(context, span_md_idx)),
                    Intrinsic::StateStoreQuad => Ok(self
                        .current_block
                        .ins(context)
                        .state_store_quad_word(val_ptr, key_var, number_of_slots_value)
                        .add_metadatum(context, span_md_idx)),
                    _ => unreachable!(),
                }
            }
            Intrinsic::Log => {
                // The log value and the log ID are just Value.
                let log_val = self.compile_expression(context, md_mgr, &arguments[0])?;
                let log_id = match self.logged_types_map.get(&arguments[0].return_type) {
                    None => {
                        return Err(CompileError::Internal(
                            "Unable to determine ID for log instance.",
                            span,
                        ))
                    }
                    Some(log_id) => {
                        convert_literal_to_value(context, &Literal::U64(**log_id as u64))
                    }
                };

                match log_val.get_type(context) {
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
            Intrinsic::Add
            | Intrinsic::Sub
            | Intrinsic::Mul
            | Intrinsic::Div
            | Intrinsic::And
            | Intrinsic::Or
            | Intrinsic::Xor => {
                let op = match kind {
                    Intrinsic::Add => BinaryOpKind::Add,
                    Intrinsic::Sub => BinaryOpKind::Sub,
                    Intrinsic::Mul => BinaryOpKind::Mul,
                    Intrinsic::Div => BinaryOpKind::Div,
                    Intrinsic::And => BinaryOpKind::And,
                    Intrinsic::Or => BinaryOpKind::Or,
                    Intrinsic::Xor => BinaryOpKind::Xor,
                    _ => unreachable!(),
                };
                let lhs = &arguments[0];
                let rhs = &arguments[1];
                let lhs_value = self.compile_expression(context, md_mgr, lhs)?;
                let rhs_value = self.compile_expression(context, md_mgr, rhs)?;
                Ok(self
                    .current_block
                    .ins(context)
                    .binary_op(op, lhs_value, rhs_value))
            }
            Intrinsic::Revert => {
                let revert_code_val = self.compile_expression(context, md_mgr, &arguments[0])?;

                // The `revert` instruction
                let span_md_idx = md_mgr.span_to_md(context, &span);
                Ok(self
                    .current_block
                    .ins(context)
                    .revert(revert_code_val)
                    .add_metadatum(context, span_md_idx))
            }
            Intrinsic::PtrAdd | Intrinsic::PtrSub => {
                let op = match kind {
                    Intrinsic::PtrAdd => BinaryOpKind::Add,
                    Intrinsic::PtrSub => BinaryOpKind::Sub,
                    _ => unreachable!(),
                };

                let len = type_arguments[0].clone();
                let ir_type = convert_resolved_typeid(
                    self.type_engine,
                    self.decl_engine,
                    context,
                    &len.type_id,
                    &len.span,
                )?;
                let len_value =
                    Constant::get_uint(context, 64, ir_type_size_in_bytes(context, &ir_type));

                let lhs = &arguments[0];
                let count = &arguments[1];
                let lhs_value = self.compile_expression(context, md_mgr, lhs)?;
                let count_value = self.compile_expression(context, md_mgr, count)?;
                let rhs_value = self.current_block.ins(context).binary_op(
                    BinaryOpKind::Mul,
                    len_value,
                    count_value,
                );
                Ok(self
                    .current_block
                    .ins(context)
                    .binary_op(op, lhs_value, rhs_value))
            }
            Intrinsic::Smo => {
                let span_md_idx = md_mgr.span_to_md(context, &span);

                /* First operand: recipient + message data */
                // Step 1: compile the user data and get its type
                let user_message = self.compile_expression(context, md_mgr, &arguments[1])?;
                let user_message_type = user_message.get_type(context).ok_or_else(|| {
                    CompileError::Internal(
                        "Unable to determine type for message data.",
                        span.clone(),
                    )
                })?;

                // Step 2: build a struct with two fields:
                // - The first field is a `b256` that contains the `recipient`
                // - The second field is a `u64` that contains the message ID
                // - The third field contains the actual user data
                let b256_ty = Type::get_b256(context);
                let u64_ty = Type::get_uint64(context);
                let field_types = [b256_ty, u64_ty, user_message_type];
                let recipient_and_message_aggregate =
                    Type::new_struct(context, field_types.to_vec());

                // Step 3: construct a local pointer for the recipient and message data struct
                let recipient_and_message_aggregate_local_name = self.lexical_map.insert_anon();
                let recipient_and_message_ptr = self
                    .function
                    .new_local_var(
                        context,
                        recipient_and_message_aggregate_local_name,
                        recipient_and_message_aggregate,
                        None,
                    )
                    .map_err(|ir_error| {
                        CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                    })?;

                // Step 4: Convert the local variable into a value via `get_local`.
                let recipient_and_message = self
                    .current_block
                    .ins(context)
                    .get_local(recipient_and_message_ptr)
                    .add_metadatum(context, span_md_idx);

                // Step 5: compile the `recipient` and insert it as the first field of the struct
                let recipient = self.compile_expression(context, md_mgr, &arguments[0])?;
                let gep_val = self.current_block.ins(context).get_elem_ptr_with_idx(
                    recipient_and_message,
                    b256_ty,
                    0,
                );
                self.current_block
                    .ins(context)
                    .store(gep_val, recipient)
                    .add_metadatum(context, span_md_idx);

                // Step 6: Grab the message ID from `messages_types_map` and insert it as the
                // second field of the struct
                let message_id_val = self
                    .messages_types_map
                    .get(&arguments[1].return_type)
                    .map(|&msg_id| Constant::get_uint(context, 64, *msg_id as u64))
                    .ok_or_else(|| {
                        CompileError::Internal(
                            "Unable to determine ID for smo instance.",
                            span.clone(),
                        )
                    })?;
                let gep_val = self.current_block.ins(context).get_elem_ptr_with_idx(
                    recipient_and_message,
                    u64_ty,
                    1,
                );
                self.current_block
                    .ins(context)
                    .store(gep_val, message_id_val)
                    .add_metadatum(context, span_md_idx);

                // Step 7: Insert the user message data as the third field of the struct
                let gep_val = self.current_block.ins(context).get_elem_ptr_with_idx(
                    recipient_and_message,
                    user_message_type,
                    2,
                );
                let user_message_size = 8 + ir_type_size_in_bytes(context, &user_message_type);
                self.current_block
                    .ins(context)
                    .store(gep_val, user_message)
                    .add_metadatum(context, span_md_idx);

                /* Second operand: the size of the message data */
                let user_message_size_val = Constant::get_uint(context, 64, user_message_size);

                /* Third operand: the output index */
                let output_index = self.compile_expression(context, md_mgr, &arguments[2])?;

                /* Fourth operand: the amount of coins to send */
                let coins = self.compile_expression(context, md_mgr, &arguments[3])?;

                Ok(self
                    .current_block
                    .ins(context)
                    .smo(
                        recipient_and_message,
                        user_message_size_val,
                        output_index,
                        coins,
                    )
                    .add_metadatum(context, span_md_idx))
            }
        }
    }

    fn compile_return_statement(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_expr: &ty::TyExpression,
    ) -> Result<Value, CompileError> {
        // Nothing to do if the current block already has a terminator
        if self.current_block.is_terminated(context) {
            return Ok(Constant::get_unit(context));
        }

        let ret_value = self.compile_expression(context, md_mgr, ast_expr)?;
        if ret_value.is_diverging(context) {
            return Ok(ret_value);
        }

        let span_md_idx = md_mgr.span_to_md(context, &ast_expr.span);
        ret_value
            .get_type(context)
            .map(|ret_ty| {
                self.current_block
                    .ins(context)
                    .ret(ret_value, ret_ty)
                    .add_metadatum(context, span_md_idx)
            })
            .ok_or_else(|| {
                CompileError::Internal(
                    "Unable to determine type for return statement expression.",
                    ast_expr.span.clone(),
                )
            })
    }

    fn compile_lazy_op(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_op: &LazyOp,
        ast_lhs: &ty::TyExpression,
        ast_rhs: &ty::TyExpression,
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
            lhs_val.get_type(context).unwrap_or_else(|| {
                rhs_val
                    .get_type(context)
                    .unwrap_or_else(|| Type::get_unit(context))
            }),
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
        call_params: &ty::ContractCallParams,
        contract_call_parameters: &HashMap<String, ty::TyExpression>,
        ast_name: &str,
        ast_args: &[(Ident, ty::TyExpression)],
        ast_return_type: TypeId,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // XXX This is very FuelVM specific and needs to be broken out of here and called
        // conditionally based on the target.

        // Compile each user argument
        let compiled_args = ast_args
            .iter()
            .map(|(_, expr)| self.compile_expression(context, md_mgr, expr))
            .collect::<Result<Vec<Value>, CompileError>>()?;

        let u64_ty = Type::get_uint64(context);

        let user_args_val = match compiled_args.len() {
            0 => Constant::get_uint(context, 64, 0),
            1 => {
                // The single arg doesn't need to be put into a struct.
                let arg0 = compiled_args[0];
                if self
                    .type_engine
                    .get_unaliased(ast_args[0].1.return_type)
                    .is_copy_type()
                {
                    self.current_block
                        .ins(context)
                        .bitcast(arg0, u64_ty)
                        .add_metadatum(context, span_md_idx)
                } else {
                    // Use a temporary to pass a reference to the arg.
                    let arg0_type = arg0.get_type(context).unwrap();
                    let temp_arg_name = self
                        .lexical_map
                        .insert(format!("{}{}", "arg_for_", ast_name));
                    let temp_var = self
                        .function
                        .new_local_var(context, temp_arg_name, arg0_type, None)
                        .map_err(|ir_error| {
                            CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                        })?;

                    let temp_val = self.current_block.ins(context).get_local(temp_var);
                    self.current_block.ins(context).store(temp_val, arg0);

                    // NOTE: Here we're casting the temp pointer to an integer.
                    self.current_block.ins(context).ptr_to_int(temp_val, u64_ty)
                }
            }
            _ => {
                // New struct type to hold the user arguments bundled together.
                let field_types = compiled_args
                    .iter()
                    .filter_map(|val| val.get_type(context))
                    .collect::<Vec<_>>();
                let user_args_struct_type = Type::new_struct(context, field_types.clone());

                // New local pointer for the struct to hold all user arguments
                let user_args_struct_local_name = self
                    .lexical_map
                    .insert(format!("{}{}", "args_struct_for_", ast_name));
                let user_args_struct_var = self
                    .function
                    .new_local_var(
                        context,
                        user_args_struct_local_name,
                        user_args_struct_type,
                        None,
                    )
                    .map_err(|ir_error| {
                        CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                    })?;

                // Initialise each of the fields in the user args struct.
                let user_args_struct_val = self
                    .current_block
                    .ins(context)
                    .get_local(user_args_struct_var)
                    .add_metadatum(context, span_md_idx);
                compiled_args
                    .into_iter()
                    .zip(field_types.into_iter())
                    .enumerate()
                    .for_each(|(insert_idx, (field_val, field_type))| {
                        let gep_val = self
                            .current_block
                            .ins(context)
                            .get_elem_ptr_with_idx(
                                user_args_struct_val,
                                field_type,
                                insert_idx as u64,
                            )
                            .add_metadatum(context, span_md_idx);

                        self.current_block
                            .ins(context)
                            .store(gep_val, field_val)
                            .add_metadatum(context, span_md_idx);
                    });

                // NOTE: Here we're casting the args struct pointer to an integer.
                self.current_block
                    .ins(context)
                    .ptr_to_int(user_args_struct_val, u64_ty)
                    .add_metadatum(context, span_md_idx)
            }
        };

        // Now handle the contract address and the selector. The contract address is just
        // as B256 while the selector is a [u8; 4] which we have to convert to a U64.
        let b256_ty = Type::get_b256(context);
        let ra_struct_type = Type::new_struct(context, [b256_ty, u64_ty, u64_ty].to_vec());

        let ra_struct_var = self
            .function
            .new_local_var(
                context,
                self.lexical_map.insert_anon(),
                ra_struct_type,
                None,
            )
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        let ra_struct_ptr_val = self
            .current_block
            .ins(context)
            .get_local(ra_struct_var)
            .add_metadatum(context, span_md_idx);

        // Insert the contract address
        let addr = self.compile_expression(context, md_mgr, &call_params.contract_address)?;
        let gep_val =
            self.current_block
                .ins(context)
                .get_elem_ptr_with_idx(ra_struct_ptr_val, b256_ty, 0);
        self.current_block
            .ins(context)
            .store(gep_val, addr)
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
        let gep_val =
            self.current_block
                .ins(context)
                .get_elem_ptr_with_idx(ra_struct_ptr_val, u64_ty, 1);
        self.current_block
            .ins(context)
            .store(gep_val, sel_val)
            .add_metadatum(context, span_md_idx);

        // Insert the user args value.
        let gep_val =
            self.current_block
                .ins(context)
                .get_elem_ptr_with_idx(ra_struct_ptr_val, u64_ty, 2);
        self.current_block
            .ins(context)
            .store(gep_val, user_args_val)
            .add_metadatum(context, span_md_idx);

        // Compile all other call parameters
        let coins = match contract_call_parameters
            .get(&constants::CONTRACT_CALL_COINS_PARAMETER_NAME.to_string())
        {
            Some(coins_expr) => self.compile_expression(context, md_mgr, coins_expr)?,
            None => convert_literal_to_value(
                context,
                &Literal::U64(constants::CONTRACT_CALL_COINS_PARAMETER_DEFAULT_VALUE),
            )
            .add_metadatum(context, span_md_idx),
        };

        // As this is Fuel VM specific we can compile the asset ID directly to a `ptr b256`
        // pointer.
        let asset_id = match contract_call_parameters
            .get(&constants::CONTRACT_CALL_ASSET_ID_PARAMETER_NAME.to_string())
        {
            Some(asset_id_expr) => {
                self.compile_ptr_or_tmp_expression(context, md_mgr, asset_id_expr)?
            }
            None => {
                let asset_id_val = convert_literal_to_value(
                    context,
                    &Literal::B256(constants::CONTRACT_CALL_ASSET_ID_PARAMETER_DEFAULT_VALUE),
                )
                .add_metadatum(context, span_md_idx);

                let tmp_asset_id_name = self.lexical_map.insert_anon();
                let tmp_var = self
                    .function
                    .new_local_var(context, tmp_asset_id_name, b256_ty, None)
                    .map_err(|ir_error| {
                        CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                    })?;
                let tmp_val = self.current_block.ins(context).get_local(tmp_var);
                self.current_block.ins(context).store(tmp_val, asset_id_val);
                tmp_val
            }
        };

        let gas = match contract_call_parameters
            .get(&constants::CONTRACT_CALL_GAS_PARAMETER_NAME.to_string())
        {
            Some(gas_expr) => self.compile_expression(context, md_mgr, gas_expr)?,
            None => self
                .current_block
                .ins(context)
                .read_register(sway_ir::Register::Cgas)
                .add_metadatum(context, span_md_idx),
        };

        // Convert the return type.  If it's a reference type then make it a pointer.
        let return_type = convert_resolved_typeid_no_span(
            self.type_engine,
            self.decl_engine,
            context,
            &ast_return_type,
        )?;
        let ret_is_copy_type = self
            .type_engine
            .get_unaliased(ast_return_type)
            .is_copy_type();
        let return_type = if ret_is_copy_type {
            return_type
        } else {
            Type::new_ptr(context, return_type)
        };

        // Insert the contract_call instruction
        let call_val = self
            .current_block
            .ins(context)
            .contract_call(
                return_type,
                ast_name.to_string(),
                ra_struct_ptr_val,
                coins,
                asset_id,
                gas,
            )
            .add_metadatum(context, span_md_idx);

        // If it's a pointer then also load it.
        Ok(if ret_is_copy_type {
            call_val
        } else {
            self.current_block.ins(context).load(call_val)
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_fn_call(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_args: &[(Ident, ty::TyExpression)],
        callee: &ty::TyFunctionDecl,
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
            callee
                .parameters
                .iter()
                .map(|p| p.type_argument.type_id)
                .collect(),
            callee.type_parameters.iter().map(|tp| tp.type_id).collect(),
        );
        let new_callee = match self.recreated_fns.get(&fn_key).copied() {
            Some(func) => func,
            None => {
                let callee_fn_decl = ty::TyFunctionDecl {
                    type_parameters: Vec::new(),
                    name: Ident::new(Span::from_string(format!(
                        "{}_{}",
                        callee.name,
                        context.get_unique_id()
                    ))),
                    parameters: callee.parameters.clone(),
                    ..callee.clone()
                };
                let is_entry = false;
                let new_func = compile_function(
                    Engines::new(self.type_engine, self.decl_engine),
                    context,
                    md_mgr,
                    self.module,
                    &callee_fn_decl,
                    &self.logged_types_map,
                    &self.messages_types_map,
                    is_entry,
                    None,
                )?
                .unwrap();
                self.recreated_fns.insert(fn_key, new_func);
                new_func
            }
        };

        // Now actually call the new function.
        let mut args = Vec::with_capacity(ast_args.len());
        for ((_, expr), param) in ast_args.iter().zip(callee.parameters.iter()) {
            self.current_fn_param = Some(param.clone());
            let arg = if param.is_reference && param.is_mutable {
                self.compile_ptr_or_tmp_expression(context, md_mgr, expr)
            } else {
                self.compile_expression(context, md_mgr, expr)
            }?;
            if arg.is_diverging(context) {
                return Ok(arg);
            }
            self.current_fn_param = None;
            args.push(arg);
        }

        let state_idx_md_idx = self_state_idx.and_then(|self_state_idx| {
            md_mgr.storage_key_to_md(context, self_state_idx.to_usize() as u64)
        });

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
        ast_condition: &ty::TyExpression,
        ast_then: &ty::TyExpression,
        ast_else: Option<&ty::TyExpression>,
        return_type: TypeId,
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
            Some(expr) => self.compile_expression(context, md_mgr, expr)?,
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

        let return_type = convert_resolved_typeid_no_span(
            self.type_engine,
            self.decl_engine,
            context,
            &return_type,
        )
        .unwrap_or_else(|_| Type::get_unit(context));
        let merge_block = self.function.create_block(context, None);
        // Add a single argument to merge_block that merges true_value and false_value.
        // Rely on the type of the ast node when creating that argument
        let merge_val_arg_idx = merge_block.new_arg(context, return_type);
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
        exp: &ty::TyExpression,
        variant: &ty::TyEnumVariant,
    ) -> Result<Value, CompileError> {
        // Retrieve the type info for the enum.
        let enum_type = match convert_resolved_typeid(
            self.type_engine,
            self.decl_engine,
            context,
            &exp.return_type,
            &exp.span,
        )? {
            ty if ty.is_struct(context) => ty,
            _ => {
                return Err(CompileError::Internal(
                    "Enum type for `unsafe downcast` is not an enum.",
                    exp.span.clone(),
                ));
            }
        };

        // Compile the struct expression.
        let compiled_value = self.compile_ptr_or_tmp_expression(context, md_mgr, exp)?;

        // Get the variant type.
        let variant_type = enum_type
            .get_indexed_type(context, &[1, variant.tag as u64])
            .ok_or_else(|| {
                CompileError::Internal(
                    "Failed to get variant type from enum in `unsigned downcast`.",
                    exp.span.clone(),
                )
            })?;

        // Get the offset to the variant.
        Ok(self.current_block.ins(context).get_elem_ptr_with_idcs(
            compiled_value,
            variant_type,
            &[1, variant.tag as u64],
        ))
    }

    fn compile_enum_tag(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        exp: Box<ty::TyExpression>,
    ) -> Result<Value, CompileError> {
        let tag_span_md_idx = md_mgr.span_to_md(context, &exp.span);
        let struct_val = self.compile_ptr_or_tmp_expression(context, md_mgr, &exp)?;

        let u64_ty = Type::get_uint64(context);
        Ok(self
            .current_block
            .ins(context)
            .get_elem_ptr_with_idx(struct_val, u64_ty, 0)
            .add_metadatum(context, tag_span_md_idx))
    }

    fn compile_while_loop(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        body: &ty::TyCodeBlock,
        condition: &ty::TyExpression,
        span_md_idx: Option<MetadataIndex>,
        span: Span,
    ) -> Result<Value, CompileError> {
        // Throw an error if the while loop is used in a predicate module.
        let module = context.module_iter().next().unwrap();
        if module.get_kind(context) == Kind::Predicate {
            return Err(CompileError::DisallowedWhileInPredicate { span });
        }

        // We're dancing around a bit here to make the blocks sit in the right order.  Ideally we
        // have the cond block, followed by the body block which may contain other blocks, and the
        // final block comes after any body block(s).
        //
        // NOTE: This is currently very important!  There is a limitation in the register allocator
        // which requires that all value uses are after the value definitions, where 'after' means
        // later in the list of instructions, as opposed to in the control flow sense.
        //
        // Hence the need for a 'break' block which does nothing more than jump to the final block,
        // as we need to construct the final block after the body block, but we need somewhere to
        // break to during the body block construction.

        // Jump to the while cond block.
        let cond_block = self.function.create_block(context, Some("while".into()));
        if !self.current_block.is_terminated(context) {
            self.current_block.ins(context).branch(cond_block, vec![]);
        }

        // Create the break block.
        let break_block = self
            .function
            .create_block(context, Some("while_break".into()));

        // Keep track of the previous blocks we have to jump to in case of a break or a continue.
        // This should be `None` if we're not in a loop already or the previous break or continue
        // destinations for the outer loop that contains the current loop.
        let prev_block_to_break_to = self.block_to_break_to;
        let prev_block_to_continue_to = self.block_to_continue_to;

        // Keep track of the current blocks to jump to in case of a break or continue.
        self.block_to_break_to = Some(break_block);
        self.block_to_continue_to = Some(cond_block);

        // Fill in the body block now, jump unconditionally to the cond block at its end.
        let body_block = self
            .function
            .create_block(context, Some("while_body".into()));
        self.current_block = body_block;
        self.compile_code_block(context, md_mgr, body)?;
        if !self.current_block.is_terminated(context) {
            self.current_block.ins(context).branch(cond_block, vec![]);
        }

        // Restore the blocks to jump to now that we're done with the current loop
        self.block_to_break_to = prev_block_to_break_to;
        self.block_to_continue_to = prev_block_to_continue_to;

        // Create the final block now we're finished with the body.
        let final_block = self
            .function
            .create_block(context, Some("end_while".into()));

        // Add an unconditional jump from the break block to the final block.
        break_block.ins(context).branch(final_block, vec![]);

        // Add the conditional in the cond block which jumps into the body or out to the final
        // block.
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

    pub(crate) fn get_function_var(&self, context: &mut Context, name: &str) -> Option<LocalVar> {
        self.lexical_map
            .get(name)
            .and_then(|local_name| self.function.get_local_var(context, local_name))
    }

    pub(crate) fn get_function_arg(&self, context: &mut Context, name: &str) -> Option<Value> {
        self.function.get_arg(context, name)
    }

    fn compile_var_expr(
        &mut self,
        context: &mut Context,
        call_path: &Option<CallPath>,
        name: &Ident,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let call_path = call_path
            .clone()
            .unwrap_or_else(|| CallPath::from(name.clone()));

        // We need to check the symbol map first, in case locals are shadowing the args, other
        // locals or even constants.
        if let Some(var) = self.get_function_var(context, name.as_str()) {
            Ok(self
                .current_block
                .ins(context)
                .get_local(var)
                .add_metadatum(context, span_md_idx))
        } else if let Some(val) = self.function.get_arg(context, name.as_str()) {
            Ok(val)
        } else if let Some(const_val) = self
            .module
            .get_global_constant(context, &call_path.as_vec_string())
        {
            Ok(const_val)
        } else if let Some(config_val) = self
            .module
            .get_global_configurable(context, &call_path.as_vec_string())
        {
            Ok(config_val)
        } else {
            Err(CompileError::InternalOwned(
                format!("Unable to resolve variable '{}'.", name.as_str()),
                Span::dummy(),
            ))
        }
    }

    fn compile_var_decl(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_var_decl: &ty::TyVariableDecl,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Option<Value>, CompileError> {
        let ty::TyVariableDecl { name, body, .. } = ast_var_decl;
        // Nothing to do for an abi cast declarations. The address specified in them is already
        // provided in each contract call node in the AST.
        if matches!(
            &self.type_engine.get_unaliased(body.return_type),
            TypeInfo::ContractCaller { .. }
        ) {
            return Ok(None);
        }

        // Grab this before we move body into compilation.
        let return_type = convert_resolved_typeid(
            self.type_engine,
            self.decl_engine,
            context,
            &body.return_type,
            &body.span,
        )?;

        // We must compile the RHS before checking for shadowing, as it will still be in the
        // previous scope.
        let body_deterministically_aborts = body.deterministically_aborts(self.decl_engine, false);
        let init_val = self.compile_expression(context, md_mgr, body)?;
        if init_val.is_diverging(context) || body_deterministically_aborts {
            return Ok(Some(init_val));
        }
        let local_name = self.lexical_map.insert(name.as_str().to_owned());
        let local_var = self
            .function
            .new_local_var(context, local_name, return_type, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        // We can have empty aggregates, especially arrays, which shouldn't be initialised, but
        // otherwise use a store.
        let var_ty = local_var.get_type(context);
        if ir_type_size_in_bytes(context, &var_ty) > 0 {
            let local_ptr = self
                .current_block
                .ins(context)
                .get_local(local_var)
                .add_metadatum(context, span_md_idx);
            self.current_block
                .ins(context)
                .store(local_ptr, init_val)
                .add_metadatum(context, span_md_idx);
        }
        Ok(None)
    }

    fn compile_const_decl(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_const_decl: ty::TyConstantDecl,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<(), CompileError> {
        // This is local to the function, so we add it to the locals, rather than the module
        // globals like other const decls.
        // `is_configurable` should be `false` here.
        let ty::TyConstantDecl {
            call_path,
            value,
            is_configurable,
            ..
        } = ast_const_decl;
        if let Some(value) = value {
            let const_expr_val = compile_constant_expression(
                Engines::new(self.type_engine, self.decl_engine),
                context,
                md_mgr,
                self.module,
                None,
                Some(self),
                &call_path,
                &value,
                is_configurable,
            )?;
            let local_name = self
                .lexical_map
                .insert(call_path.suffix.as_str().to_owned());
            let return_type = convert_resolved_typeid(
                self.type_engine,
                self.decl_engine,
                context,
                &value.return_type,
                &value.span,
            )?;

            // We compile consts the same as vars are compiled. This is because ASM generation
            // cannot handle
            //    1. initializing aggregates
            //    2. get_ptr()
            // into the data section.
            let local_var = self
                .function
                .new_local_var(context, local_name, return_type, None)
                .map_err(|ir_error| {
                    CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                })?;

            // We can have empty aggregates, especially arrays, which shouldn't be initialised, but
            // otherwise use a store.
            let var_ty = local_var.get_type(context);
            if ir_type_size_in_bytes(context, &var_ty) > 0 {
                let local_val = self
                    .current_block
                    .ins(context)
                    .get_local(local_var)
                    .add_metadatum(context, span_md_idx);
                self.current_block
                    .ins(context)
                    .store(local_val, const_expr_val)
                    .add_metadatum(context, span_md_idx);
            }
        }
        Ok(())
    }

    fn compile_reassignment(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_reassignment: &ty::TyReassignment,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let name = self
            .lexical_map
            .get(ast_reassignment.lhs_base_name.as_str())
            .expect("All local symbols must be in the lexical symbol map.");

        // First look for a local variable with the required name
        let lhs_val = self
            .function
            .get_local_var(context, name)
            .map(|var| {
                self.current_block
                    .ins(context)
                    .get_local(var)
                    .add_metadatum(context, span_md_idx)
            })
            .or_else(||
                // Now look for an argument with the required name
                self.function
                    .args_iter(context)
                    .find_map(|(arg_name, arg_val)| (arg_name == name).then_some(*arg_val)))
            .ok_or_else(|| {
                CompileError::InternalOwned(
                    format!("variable not found: {name}"),
                    ast_reassignment.lhs_base_name.span(),
                )
            })?;

        let reassign_val = self.compile_expression(context, md_mgr, &ast_reassignment.rhs)?;
        if reassign_val.is_diverging(context) {
            return Ok(reassign_val);
        }

        let lhs_ptr = if ast_reassignment.lhs_indices.is_empty() {
            // A non-aggregate; use a direct `store`.
            lhs_val
        } else {
            // Create a GEP by following the chain of LHS indices.  We use a scan which is
            // essentially a map with context, which is the parent type id for the current field.
            let gep_indices = ast_reassignment
                .lhs_indices
                .iter()
                .scan(ast_reassignment.lhs_type, |cur_type_id, idx_kind| {
                    let cur_type_info = self.type_engine.get_unaliased(*cur_type_id);
                    Some(match (idx_kind, cur_type_info) {
                        (
                            ProjectionKind::StructField { name: idx_name },
                            TypeInfo::Struct(decl_ref),
                        ) => {
                            // Get the struct type info, with field names.
                            let ty::TyStructDecl {
                                call_path: struct_call_path,
                                fields: struct_fields,
                                ..
                            } = self.decl_engine.get_struct(&decl_ref);

                            // Search for the index to the field name we're after, and its type
                            // id.
                            struct_fields
                                .iter()
                                .enumerate()
                                .find(|(_, field)| field.name == *idx_name)
                                .map(|(idx, field)| (idx as u64, field.type_argument.type_id))
                                .ok_or_else(|| {
                                    CompileError::InternalOwned(
                                        format!(
                                            "Unknown field name '{idx_name}' for struct '{}' \
                                                in reassignment.",
                                            struct_call_path.suffix.as_str(),
                                        ),
                                        ast_reassignment.lhs_base_name.span(),
                                    )
                                })
                                .map(|(field_idx, field_type_id)| {
                                    *cur_type_id = field_type_id;
                                    Constant::get_uint(context, 64, field_idx)
                                })
                        }
                        (ProjectionKind::TupleField { index, .. }, TypeInfo::Tuple(field_tys)) => {
                            *cur_type_id = field_tys[*index].type_id;
                            Ok(Constant::get_uint(context, 64, *index as u64))
                        }
                        (ProjectionKind::ArrayIndex { index, .. }, TypeInfo::Array(elem_ty, _)) => {
                            *cur_type_id = elem_ty.type_id;
                            self.compile_expression(context, md_mgr, index)
                        }
                        _ => Err(CompileError::Internal(
                            "Unknown field in reassignment.",
                            idx_kind.span(),
                        )),
                    })
                })
                .collect::<Result<Vec<Value>, _>>()?;

            // Using the type of the RHS for the GEP, rather than the final inner type of the
            // aggregate, but getting the later is a bit of a pain, though the `scan` above knew it.
            // Realistically the program is type checked and they should be the same.
            let field_type = reassign_val.get_type(context).ok_or_else(|| {
                CompileError::Internal(
                    "Failed to determine type of reassignment.",
                    ast_reassignment.lhs_base_name.span(),
                )
            })?;

            // Create the GEP.
            self.current_block
                .ins(context)
                .get_elem_ptr(lhs_val, field_type, gep_indices)
                .add_metadatum(context, span_md_idx)
        };

        self.current_block
            .ins(context)
            .store(lhs_ptr, reassign_val)
            .add_metadatum(context, span_md_idx);

        Ok(Constant::get_unit(context).add_metadatum(context, span_md_idx))
    }

    fn compile_storage_reassignment(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        fields: &[ty::TyStorageReassignDescriptor],
        ix: &StateIndex,
        rhs: &ty::TyExpression,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // Compile the RHS into a value
        let rhs = self.compile_expression(context, md_mgr, rhs)?;
        if rhs.is_diverging(context) {
            return Ok(rhs);
        }

        // Get the type of the access which can be a subfield
        let access_type = convert_resolved_typeid_no_span(
            self.type_engine,
            self.decl_engine,
            context,
            &fields.last().expect("guaranteed by grammar").type_id,
        )?;

        // Get the list of indices used to access the storage field. This will be empty
        // if the storage field type is not a struct.
        let base_type = fields[0].type_id;
        let field_idcs = get_indices_for_struct_access(
            self.type_engine,
            self.decl_engine,
            base_type,
            &fields[1..],
        )?;

        // Do the actual work. This is a recursive function because we want to drill down
        // to store each primitive type in the storage field in its own storage slot.
        self.compile_storage_write(context, ix, &field_idcs, &access_type, rhs, span_md_idx)?;
        Ok(Constant::get_unit(context).add_metadatum(context, span_md_idx))
    }

    fn compile_array_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        elem_type: &TypeId,
        contents: &[ty::TyExpression],
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let elem_type = convert_resolved_typeid_no_span(
            self.type_engine,
            self.decl_engine,
            context,
            elem_type,
        )?;

        let array_type = Type::new_array(context, elem_type, contents.len() as u64);

        let temp_name = self.lexical_map.insert_anon();
        let array_var = self
            .function
            .new_local_var(context, temp_name, array_type, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;
        let array_value = self
            .current_block
            .ins(context)
            .get_local(array_var)
            .add_metadatum(context, span_md_idx);

        // Compile each element and insert it immediately.
        for (idx, elem_expr) in contents.iter().enumerate() {
            let elem_value = self.compile_expression(context, md_mgr, elem_expr)?;
            if elem_value.is_diverging(context) {
                return Ok(elem_value);
            }
            let gep_val = self.current_block.ins(context).get_elem_ptr_with_idx(
                array_value,
                elem_type,
                idx as u64,
            );
            self.current_block
                .ins(context)
                .store(gep_val, elem_value)
                .add_metadatum(context, span_md_idx);
        }
        Ok(array_value)
    }

    fn compile_array_index(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        array_expr: &ty::TyExpression,
        index_expr: &ty::TyExpression,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let array_val = self.compile_ptr_or_tmp_expression(context, md_mgr, array_expr)?;
        if array_val.is_diverging(context) {
            return Ok(array_val);
        }

        // Get the array type and confirm it's an array.
        let array_type = array_val
            .get_type(context)
            .and_then(|ty| ty.get_inner_type(context))
            .and_then(|ty| ty.is_array(context).then_some(ty))
            .ok_or_else(|| {
                CompileError::Internal(
                    "Unsupported array value for index expression.",
                    array_expr.span.clone(),
                )
            })?;

        let index_expr_span = index_expr.span.clone();

        // Perform a bounds check if the array index is a constant int.
        if let Ok(Constant {
            value: ConstantValue::Uint(constant_value),
            ..
        }) = compile_constant_expression_to_constant(
            Engines::new(self.type_engine, self.decl_engine),
            context,
            md_mgr,
            self.module,
            None,
            Some(self),
            index_expr,
        ) {
            let count = array_type.get_array_len(context).unwrap();
            if constant_value >= count {
                return Err(CompileError::ArrayOutOfBounds {
                    index: constant_value,
                    count,
                    span: index_expr_span,
                });
            }
        }

        let index_val = self.compile_expression(context, md_mgr, index_expr)?;
        if index_val.is_diverging(context) {
            return Ok(index_val);
        }

        let elem_type = array_type.get_array_elem_type(context).ok_or_else(|| {
            CompileError::Internal(
                "Array type has alread confirmed to be an array.  Getting elem type can't fail.",
                array_expr.span.clone(),
            )
        })?;

        Ok(self
            .current_block
            .ins(context)
            .get_elem_ptr(array_val, elem_type, vec![index_val])
            .add_metadatum(context, span_md_idx))
    }

    fn compile_struct_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        fields: &[ty::TyStructExpressionField],
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // NOTE: This is a struct instantiation with initialisers for each field of a named struct.
        // We don't know the actual type of the struct, but the AST guarantees that the fields are
        // in the declared order (regardless of how they are initialised in source) so we can
        // create a struct with the field types.

        // Compile each of the values for field initialisers, calculate their indices and also
        // gather their types with which to make an aggregate.

        let mut insert_values = Vec::with_capacity(fields.len());
        let mut field_types = Vec::with_capacity(fields.len());
        for struct_field in fields.iter() {
            let insert_val = self.compile_expression(context, md_mgr, &struct_field.value)?;
            if insert_val.is_diverging(context) {
                return Ok(insert_val);
            }
            insert_values.push(insert_val);

            field_types.push(convert_resolved_typeid_no_span(
                self.type_engine,
                self.decl_engine,
                context,
                &struct_field.value.return_type,
            )?);
        }

        // Create the struct.
        let struct_type = Type::new_struct(context, field_types.clone());
        let temp_name = self.lexical_map.insert_anon();
        let struct_var = self
            .function
            .new_local_var(context, temp_name, struct_type, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;
        let struct_val = self
            .current_block
            .ins(context)
            .get_local(struct_var)
            .add_metadatum(context, span_md_idx);

        // Fill it in.
        insert_values
            .into_iter()
            .zip(field_types.into_iter())
            .enumerate()
            .for_each(|(insert_idx, (insert_val, field_type))| {
                let gep_val = self.current_block.ins(context).get_elem_ptr_with_idx(
                    struct_val,
                    field_type,
                    insert_idx as u64,
                );

                self.current_block
                    .ins(context)
                    .store(gep_val, insert_val)
                    .add_metadatum(context, span_md_idx);
            });

        // Return the pointer.
        Ok(struct_val)
    }

    fn compile_struct_field_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_struct_expr: &ty::TyExpression,
        struct_type_id: TypeId,
        ast_field: &ty::TyStructField,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let struct_val = self.compile_ptr_or_tmp_expression(context, md_mgr, ast_struct_expr)?;

        // Get the struct type info, with field names.
        let TypeInfo::Struct(decl_ref) = self.type_engine.get_unaliased(struct_type_id) else {
            return Err(CompileError::Internal(
                "Unknown struct in field expression.",
                ast_field.span.clone(),
            ));
        };
        let crate::language::ty::TyStructDecl {
            call_path: struct_call_path,
            fields: struct_fields,
            ..
        } = self.decl_engine.get_struct(&decl_ref);

        // Search for the index to the field name we're after, and its type id.
        let (field_idx, field_type_id) = struct_fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.name == ast_field.name)
            .map(|(idx, field)| (idx as u64, field.type_argument.type_id))
            .ok_or_else(|| {
                CompileError::InternalOwned(
                    format!(
                        "Unknown field name '{}' for struct '{}' in field expression.",
                        struct_call_path.suffix.as_str(),
                        ast_field.name
                    ),
                    ast_field.span.clone(),
                )
            })?;

        let field_type = convert_resolved_typeid(
            self.type_engine,
            self.decl_engine,
            context,
            &field_type_id,
            &ast_field.span,
        )?;

        Ok(self
            .current_block
            .ins(context)
            .get_elem_ptr_with_idx(struct_val, field_type, field_idx)
            .add_metadatum(context, span_md_idx))
    }

    fn compile_enum_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        enum_decl: &ty::TyEnumDecl,
        tag: usize,
        contents: Option<&ty::TyExpression>,
    ) -> Result<Value, CompileError> {
        // XXX The enum instantiation AST node includes the full declaration.  If the enum was
        // declared in a different module then it seems for now there's no easy way to pre-analyse
        // it and add its type/aggregate to the context.  We can re-use them here if we recognise
        // the name, and if not add a new aggregate... OTOH the naming seems a little fragile and
        // we could potentially use the wrong aggregate with the same name, different module...
        // dunno.
        let span_md_idx = md_mgr.span_to_md(context, &enum_decl.span);
        let enum_type = create_tagged_union_type(
            self.type_engine,
            self.decl_engine,
            context,
            &enum_decl.variants,
        )?;
        let tag_value =
            Constant::get_uint(context, 64, tag as u64).add_metadatum(context, span_md_idx);

        // Start with a temporary local struct and insert the tag.
        let temp_name = self.lexical_map.insert_anon();
        let enum_var = self
            .function
            .new_local_var(context, temp_name, enum_type, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;
        let enum_ptr = self
            .current_block
            .ins(context)
            .get_local(enum_var)
            .add_metadatum(context, span_md_idx);
        let u64_ty = Type::get_uint64(context);
        let tag_gep_val = self
            .current_block
            .ins(context)
            .get_elem_ptr_with_idx(enum_ptr, u64_ty, 0)
            .add_metadatum(context, span_md_idx);
        self.current_block
            .ins(context)
            .store(tag_gep_val, tag_value)
            .add_metadatum(context, span_md_idx);

        // If the struct representing the enum has only one field, then that field is the tag and
        // all the variants must have unit types, hence the absence of the union. Therefore, there
        // is no need for another `store` instruction here.
        let field_tys = enum_type.get_field_types(context);
        if field_tys.len() != 1 && contents.is_some() {
            // Insert the value too.
            let contents_value = self.compile_expression(context, md_mgr, contents.unwrap())?;
            let contents_type = contents_value.get_type(context).ok_or_else(|| {
                CompileError::Internal(
                    "Unable to get type for enum contents.",
                    enum_decl.span.clone(),
                )
            })?;
            let gep_val = self
                .current_block
                .ins(context)
                .get_elem_ptr_with_idcs(enum_ptr, contents_type, &[1, tag as u64])
                .add_metadatum(context, span_md_idx);
            self.current_block
                .ins(context)
                .store(gep_val, contents_value)
                .add_metadatum(context, span_md_idx);
        }

        // Return the pointer.
        Ok(enum_ptr)
    }

    fn compile_tuple_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        fields: &[ty::TyExpression],
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
                let init_type = convert_resolved_typeid_no_span(
                    self.type_engine,
                    self.decl_engine,
                    context,
                    &field_expr.return_type,
                )?;
                let init_value = self.compile_expression(context, md_mgr, field_expr)?;
                if init_value.is_diverging(context) {
                    return Ok(init_value);
                }
                init_values.push(init_value);
                init_types.push(init_type);
            }

            let tuple_type = Type::new_struct(context, init_types.clone());
            let temp_name = self.lexical_map.insert_anon();
            let tuple_var = self
                .function
                .new_local_var(context, temp_name, tuple_type, None)
                .map_err(|ir_error| {
                    CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                })?;
            let tuple_val = self
                .current_block
                .ins(context)
                .get_local(tuple_var)
                .add_metadatum(context, span_md_idx);

            init_values
                .into_iter()
                .zip(init_types.into_iter())
                .enumerate()
                .for_each(|(insert_idx, (field_val, field_type))| {
                    let gep_val = self
                        .current_block
                        .ins(context)
                        .get_elem_ptr_with_idx(tuple_val, field_type, insert_idx as u64)
                        .add_metadatum(context, span_md_idx);
                    self.current_block
                        .ins(context)
                        .store(gep_val, field_val)
                        .add_metadatum(context, span_md_idx);
                });

            Ok(tuple_val)
        }
    }

    fn compile_tuple_elem_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        tuple: &ty::TyExpression,
        tuple_type: TypeId,
        idx: usize,
        span: Span,
    ) -> Result<Value, CompileError> {
        let tuple_value = self.compile_ptr_or_tmp_expression(context, md_mgr, tuple)?;
        let tuple_type = convert_resolved_typeid(
            self.type_engine,
            self.decl_engine,
            context,
            &tuple_type,
            &span,
        )?;
        tuple_type
            .get_field_type(context, idx as u64)
            .map(|field_type| {
                let span_md_idx = md_mgr.span_to_md(context, &span);
                self.current_block
                    .ins(context)
                    .get_elem_ptr_with_idx(tuple_value, field_type, idx as u64)
                    .add_metadatum(context, span_md_idx)
            })
            .ok_or_else(|| {
                CompileError::Internal(
                    "Invalid (non-aggregate?) tuple type for TupleElemAccess.",
                    span,
                )
            })
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
            self.type_engine,
            self.decl_engine,
            context,
            &fields.last().expect("guaranteed by grammar").type_id,
        )?;

        // Get the list of indices used to access the storage field. This will be empty
        // if the storage field type is not a struct.
        // FIXME: shouldn't have to extract the first field like this.
        let base_type = fields[0].type_id;
        let field_idcs = get_indices_for_struct_access(
            self.type_engine,
            self.decl_engine,
            base_type,
            &fields[1..],
        )?;

        // Do the actual work. This is a recursive function because we want to drill down
        // to load each primitive type in the storage field in its own storage slot.
        self.compile_storage_read(context, md_mgr, ix, &field_idcs, &access_type, span_md_idx)
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_asm_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        registers: &[ty::TyAsmRegisterDeclaration],
        body: &[AsmOp],
        return_type: TypeId,
        returns: Option<&(AsmRegister, Span)>,
        whole_block_span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let registers = registers
            .iter()
            .map(
                |ty::TyAsmRegisterDeclaration {
                     initializer, name, ..
                 }| {
                    // Take the optional initialiser, map it to an Option<Result<Value>>,
                    // transpose that to Result<Option<Value>> and map that to an AsmArg.
                    //
                    // Here we need to compile based on the Sway 'copy-type' vs 'ref-type' since
                    // ASM args aren't explicitly typed, and if we send in a temporary it might
                    // be mutated and then discarded.  It *must* be a ptr to *the* ref-type value,
                    // *or* the value of the copy-type value.
                    initializer
                        .as_ref()
                        .map(|init_expr| {
                            self.compile_ptr_expression(context, md_mgr, init_expr).map(
                                |init_val| {
                                    if init_val
                                        .get_type(context)
                                        .map_or(false, |ty| ty.is_ptr(context))
                                        && self
                                            .type_engine
                                            .get_unaliased(init_expr.return_type)
                                            .is_copy_type()
                                    {
                                        // It's a pointer to a copy type.  We need to derefence it.
                                        self.current_block.ins(context).load(init_val)
                                    } else {
                                        init_val
                                    }
                                },
                            )
                        })
                        .transpose()
                        .map(|init| AsmArg {
                            name: name.clone(),
                            initializer: init,
                        })
                },
            )
            .collect::<Result<Vec<AsmArg>, CompileError>>()?;
        let body = body
            .iter()
            .map(
                |AsmOp {
                     op_name,
                     op_args,
                     immediate,
                     span,
                 }| AsmInstruction {
                    name: op_name.clone(),
                    args: op_args.clone(),
                    immediate: immediate.clone(),
                    metadata: md_mgr.span_to_md(context, span),
                },
            )
            .collect();
        let returns = returns
            .as_ref()
            .map(|(_, asm_reg_span)| Ident::new(asm_reg_span.clone()));
        let return_type = convert_resolved_typeid_no_span(
            self.type_engine,
            self.decl_engine,
            context,
            &return_type,
        )?;
        Ok(self
            .current_block
            .ins(context)
            .asm_block(registers, body, return_type, returns)
            .add_metadatum(context, whole_block_span_md_idx))
    }

    fn compile_storage_read(
        &mut self,
        context: &mut Context,
        _md_mgr: &mut MetadataManager,
        ix: &StateIndex,
        indices: &[u64],
        ty: &Type,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        if ty.is_struct(context) {
            let temp_name = self.lexical_map.insert_anon();
            let struct_var = self
                .function
                .new_local_var(context, temp_name, *ty, None)
                .map_err(|ir_error| {
                    CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                })?;
            let struct_val = self
                .current_block
                .ins(context)
                .get_local(struct_var)
                .add_metadatum(context, span_md_idx);

            let fields = ty.get_field_types(context);
            for (field_idx, field_type) in fields.into_iter().enumerate() {
                let field_idx = field_idx as u64;

                // Recurse. The base case is for primitive types that fit in a single storage slot.
                let mut new_indices = indices.to_owned();
                new_indices.push(field_idx);

                let val_to_insert = self.compile_storage_read(
                    context,
                    _md_mgr,
                    ix,
                    &new_indices,
                    &field_type,
                    span_md_idx,
                )?;

                //  Insert the loaded value to the aggregate at the given index
                let gep_val = self
                    .current_block
                    .ins(context)
                    .get_elem_ptr_with_idx(struct_val, field_type, field_idx)
                    .add_metadatum(context, span_md_idx);
                self.current_block
                    .ins(context)
                    .store(gep_val, val_to_insert)
                    .add_metadatum(context, span_md_idx);
            }

            Ok(self
                .current_block
                .ins(context)
                .load(struct_val)
                .add_metadatum(context, span_md_idx))
        } else {
            let storage_key = get_storage_key(ix, indices);

            // New name for the key
            let mut key_name = format!("{}{}", "key_for_", ix.to_usize());
            for ix in indices {
                key_name = format!("{key_name}_{ix}");
            }
            let alias_key_name = self.lexical_map.insert(key_name.as_str().to_owned());

            // Local pointer for the key
            let key_var = self
                .function
                .new_local_var(context, alias_key_name, Type::get_b256(context), None)
                .map_err(|ir_error| {
                    CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                })?;

            // Const value for the key from the hash
            let const_key = convert_literal_to_value(context, &Literal::B256(storage_key.into()))
                .add_metadatum(context, span_md_idx);

            let key_ptr = self
                .current_block
                .ins(context)
                .get_local(key_var)
                .add_metadatum(context, span_md_idx);

            // Store the const hash value to the key pointer value
            self.current_block
                .ins(context)
                .store(key_ptr, const_key)
                .add_metadatum(context, span_md_idx);

            match ty.get_content(context) {
                TypeContent::Array(..) => Err(CompileError::Internal(
                    "Arrays in storage have not been implemented yet.",
                    Span::dummy(),
                )),
                TypeContent::Slice => Err(CompileError::Internal(
                    "Slices in storage are not valid.",
                    Span::dummy(),
                )),
                TypeContent::Pointer(_) => Err(CompileError::Internal(
                    "Pointers in storage are not valid.",
                    Span::dummy(),
                )),
                TypeContent::B256 => {
                    self.compile_b256_storage_read(context, ix, indices, &key_ptr, span_md_idx)
                }
                TypeContent::Bool | TypeContent::Uint(_) => {
                    self.compile_uint_or_bool_storage_read(context, &key_ptr, ty, span_md_idx)
                }
                TypeContent::String(_) | TypeContent::Union(_) => self
                    .compile_union_or_string_storage_read(
                        context,
                        ix,
                        indices,
                        &key_ptr,
                        ty,
                        span_md_idx,
                    ),
                TypeContent::Struct(_) => unreachable!("structs are already handled!"),
                TypeContent::Unit => {
                    Ok(Constant::get_unit(context).add_metadatum(context, span_md_idx))
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_storage_write(
        &mut self,
        context: &mut Context,
        ix: &StateIndex,
        indices: &[u64],
        ty: &Type,
        rhs: Value,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<(), CompileError> {
        match ty {
            ty if ty.is_struct(context) => {
                // Create a temporary local struct, store the RHS to it, and read each field from
                // there.
                let temp_name = self.lexical_map.insert_anon();
                let tmp_struct_var = self
                    .function
                    .new_local_var(context, temp_name, *ty, None)
                    .map_err(|ir_error| {
                        CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                    })?;
                let tmp_struct_val = self
                    .current_block
                    .ins(context)
                    .get_local(tmp_struct_var)
                    .add_metadatum(context, span_md_idx);
                self.current_block.ins(context).store(tmp_struct_val, rhs);

                let fields = ty.get_field_types(context);
                for (field_idx, field_type) in fields.into_iter().enumerate() {
                    let field_idx = field_idx as u64;

                    // Recurse. The base case is for primitive types that fit in a single storage slot.
                    let mut new_indices = indices.to_owned();
                    new_indices.push(field_idx);

                    // Extract the value from the aggregate at the given index
                    let gep_val = self
                        .current_block
                        .ins(context)
                        .get_elem_ptr_with_idx(tmp_struct_val, field_type, field_idx)
                        .add_metadatum(context, span_md_idx);
                    let rhs = self
                        .current_block
                        .ins(context)
                        .load(gep_val)
                        .add_metadatum(context, span_md_idx);

                    self.compile_storage_write(
                        context,
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
                    key_name = format!("{key_name}_{ix}");
                }
                let alias_key_name = self.lexical_map.insert(key_name.as_str().to_owned());

                // Local pointer for the key
                let key_var = self
                    .function
                    .new_local_var(context, alias_key_name, Type::get_b256(context), None)
                    .map_err(|ir_error| {
                        CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                    })?;

                // Const value for the key from the hash
                let const_key =
                    convert_literal_to_value(context, &Literal::B256(storage_key.into()))
                        .add_metadatum(context, span_md_idx);

                // Convert the key pointer to a value using get_ptr
                let key_val = self
                    .current_block
                    .ins(context)
                    .get_local(key_var)
                    .add_metadatum(context, span_md_idx);

                // Store the const hash value to the key pointer value
                self.current_block
                    .ins(context)
                    .store(key_val, const_key)
                    .add_metadatum(context, span_md_idx);

                match ty.get_content(context) {
                    TypeContent::Array(..) => Err(CompileError::Internal(
                        "Arrays in storage have not been implemented yet.",
                        Span::dummy(),
                    )),
                    TypeContent::Slice => Err(CompileError::Internal(
                        "Slices in storage are not valid.",
                        Span::dummy(),
                    )),
                    TypeContent::Pointer(_) => Err(CompileError::Internal(
                        "Pointers in storage are not valid.",
                        Span::dummy(),
                    )),
                    TypeContent::B256 => self.compile_b256_storage_write(
                        context,
                        ix,
                        indices,
                        &key_val,
                        rhs,
                        span_md_idx,
                    ),
                    TypeContent::Bool | TypeContent::Uint(_) => {
                        self.compile_uint_or_bool_storage_write(context, &key_val, rhs, span_md_idx)
                    }
                    TypeContent::String(_) | TypeContent::Union(_) => self
                        .compile_union_or_string_storage_write(
                            context,
                            ix,
                            indices,
                            &key_val,
                            ty,
                            rhs,
                            span_md_idx,
                        ),
                    TypeContent::Struct(_) => unreachable!("structs are already handled!"),
                    TypeContent::Unit => Ok(()),
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
        let u64_ty = Type::get_uint64(context);
        // `state_store_word` requires a `u64`. Cast the value to store to
        // `u64` first before actually storing.
        let rhs_u64 = self
            .current_block
            .ins(context)
            .bitcast(rhs, u64_ty)
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
            value_name = format!("{value_name}_{ix}");
        }
        let alias_value_name = self.lexical_map.insert(value_name.as_str().to_owned());

        // Local pointer to hold the B256
        let local_var = self
            .function
            .new_local_var(context, alias_value_name, Type::get_b256(context), None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        // Convert the local pointer created to a value using get_ptr
        let local_ptr = self
            .current_block
            .ins(context)
            .get_local(local_var)
            .add_metadatum(context, span_md_idx);

        let one_value = convert_literal_to_value(context, &Literal::U64(1));
        self.current_block
            .ins(context)
            .state_load_quad_word(local_ptr, *key_ptr_val, one_value)
            .add_metadatum(context, span_md_idx);
        let local_val = self
            .current_block
            .ins(context)
            .load(local_ptr)
            .add_metadatum(context, span_md_idx);

        Ok(local_val)
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
            value_name = format!("{value_name}_{ix}");
        }
        let alias_value_name = self.lexical_map.insert(value_name.as_str().to_owned());

        // Local pointer to hold the B256
        let local_var = self
            .function
            .new_local_var(context, alias_value_name, Type::get_b256(context), None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        // Convert the local pointer created to a value using get_ptr
        let local_val = self
            .current_block
            .ins(context)
            .get_local(local_var)
            .add_metadatum(context, span_md_idx);

        // Store the value to the local pointer created for rhs
        self.current_block
            .ins(context)
            .store(local_val, rhs)
            .add_metadatum(context, span_md_idx);

        // Finally, just call state_load_quad_word/state_store_quad_word
        let one_value = convert_literal_to_value(context, &Literal::U64(1));
        self.current_block
            .ins(context)
            .state_store_quad_word(local_val, *key_ptr_val, one_value)
            .add_metadatum(context, span_md_idx);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_union_or_string_storage_read(
        &mut self,
        context: &mut Context,
        ix: &StateIndex,
        indices: &[u64],
        key_val: &Value,
        ty: &Type,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
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
        let local_value_name = self.lexical_map.insert(value_name);

        // Create an array of `b256` that will hold the value to store into storage
        // or the value loaded from storage. The array has to fit the whole type.
        let number_of_elements = (ir_type_size_in_bytes(context, ty) + 31) / 32;
        let b256_array_type = Type::new_array(context, Type::get_b256(context), number_of_elements);

        // Local pointer to hold the array of b256s
        let local_var = self
            .function
            .new_local_var(context, local_value_name, b256_array_type, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        // Convert the local pointer created to a value of the original type using cast_ptr.
        let local_val = self
            .current_block
            .ins(context)
            .get_local(local_var)
            .add_metadatum(context, span_md_idx);
        let ptr_ty = Type::new_ptr(context, *ty);
        let final_val = self
            .current_block
            .ins(context)
            .cast_ptr(local_val, ptr_ty)
            .add_metadatum(context, span_md_idx);
        let b256_ptr_ty = Type::new_ptr(context, Type::get_b256(context));

        if number_of_elements > 0 {
            // Get the b256 from the array at index iter
            let value_val_b256 = self
                .current_block
                .ins(context)
                .get_local(local_var)
                .add_metadatum(context, span_md_idx);
            let indexed_value_val_b256 = self
                .current_block
                .ins(context)
                .cast_ptr(value_val_b256, b256_ptr_ty)
                .add_metadatum(context, span_md_idx);

            let count_value = convert_literal_to_value(context, &Literal::U64(number_of_elements));
            self.current_block
                .ins(context)
                .state_load_quad_word(indexed_value_val_b256, *key_val, count_value)
                .add_metadatum(context, span_md_idx);
        }

        Ok(self
            .current_block
            .ins(context)
            .load(final_val)
            .add_metadatum(context, span_md_idx))
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_union_or_string_storage_write(
        &mut self,
        context: &mut Context,
        ix: &StateIndex,
        indices: &[u64],
        key_val: &Value,
        ty: &Type,
        rhs: Value,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<(), CompileError> {
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
        let local_value_name = self.lexical_map.insert(value_name);

        // Create an array of `b256` that will hold the value to store into storage
        // or the value loaded from storage. The array has to fit the whole type.
        let number_of_elements = (ir_type_size_in_bytes(context, ty) + 31) / 32;
        let b256_array_type = Type::new_array(context, Type::get_b256(context), number_of_elements);

        // Local pointer to hold the array of b256s
        let local_var = self
            .function
            .new_local_var(context, local_value_name, b256_array_type, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        // Convert the local pointer created to a value of the original type using
        // get_ptr.
        let local_val = self
            .current_block
            .ins(context)
            .get_local(local_var)
            .add_metadatum(context, span_md_idx);
        let ptr_ty = Type::new_ptr(context, *ty);
        let final_val = self
            .current_block
            .ins(context)
            .cast_ptr(local_val, ptr_ty)
            .add_metadatum(context, span_md_idx);

        // Store the value to the local pointer created for rhs
        self.current_block
            .ins(context)
            .store(final_val, rhs)
            .add_metadatum(context, span_md_idx);

        let b256_ptr_ty = Type::new_ptr(context, Type::get_b256(context));
        if number_of_elements > 0 {
            // Get the b256 from the array at index iter
            let value_ptr_val_b256 = self
                .current_block
                .ins(context)
                .get_local(local_var)
                .add_metadatum(context, span_md_idx);
            let indexed_value_ptr_val_b256 = self
                .current_block
                .ins(context)
                .cast_ptr(value_ptr_val_b256, b256_ptr_ty)
                .add_metadatum(context, span_md_idx);

            // Finally, just call state_load_quad_word/state_store_quad_word
            let count_value = convert_literal_to_value(context, &Literal::U64(number_of_elements));
            self.current_block
                .ins(context)
                .state_store_quad_word(indexed_value_ptr_val_b256, *key_val, count_value)
                .add_metadatum(context, span_md_idx);
        }

        Ok(())
    }
}
