use crate::{
    asm_generation::from_ir::ir_type_size_in_bytes,
    constants,
    error::CompileError,
    parse_tree::{AsmOp, AsmRegister, LazyOp, Literal, Purity, Visibility},
    semantic_analysis::*,
    type_engine::{insert_type, resolve_type, TypeId, TypeInfo},
};

use super::{compile::compile_function, convert::*, lexical_map::LexicalMap, types::*};

use fuel_crypto::Hasher;
use sway_ir::{Context, *};
use sway_types::{
    ident::Ident,
    span::{Span, Spanned},
    state::StateIndex,
};

use std::{collections::HashMap, sync::Arc};

pub(super) struct FnCompiler {
    module: Module,
    pub(super) function: Function,
    pub(super) current_block: Block,
    lexical_map: LexicalMap,
}

pub(super) enum StateAccessType {
    Read,
    Write,
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
            lexical_map,
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
        ast_block: TypedCodeBlock,
    ) -> Result<Value, CompileError> {
        self.compile_with_new_scope(|fn_compiler| {
            fn_compiler.compile_code_block_inner(context, ast_block)
        })
    }

    fn compile_code_block_inner(
        &mut self,
        context: &mut Context,
        ast_block: TypedCodeBlock,
    ) -> Result<Value, CompileError> {
        self.lexical_map.enter_scope();
        let value = ast_block
            .contents
            .into_iter()
            .map(|ast_node| {
                let span_md_idx = MetadataIndex::from_span(context, &ast_node.span);
                match ast_node.content {
                    TypedAstNodeContent::ReturnStatement(trs) => {
                        self.compile_return_statement(context, trs.expr)
                    }
                    TypedAstNodeContent::Declaration(td) => match td {
                        TypedDeclaration::VariableDeclaration(tvd) => {
                            self.compile_var_decl(context, tvd, span_md_idx)
                        }
                        TypedDeclaration::ConstantDeclaration(tcd) => {
                            self.compile_const_decl(context, tcd, span_md_idx)
                        }
                        TypedDeclaration::FunctionDeclaration(_) => {
                            Err(CompileError::UnexpectedDeclaration {
                                decl_type: "function",
                                span: ast_node.span,
                            })
                        }
                        TypedDeclaration::TraitDeclaration(_) => {
                            Err(CompileError::UnexpectedDeclaration {
                                decl_type: "trait",
                                span: ast_node.span,
                            })
                        }
                        TypedDeclaration::StructDeclaration(_) => {
                            Err(CompileError::UnexpectedDeclaration {
                                decl_type: "struct",
                                span: ast_node.span,
                            })
                        }
                        TypedDeclaration::EnumDeclaration(ted) => {
                            let span_md_idx = MetadataIndex::from_span(context, &ted.span);
                            create_enum_aggregate(context, ted.variants).map(|_| ())?;
                            Ok(Constant::get_unit(context, span_md_idx))
                        }
                        TypedDeclaration::Reassignment(tr) => {
                            self.compile_reassignment(context, tr, span_md_idx)
                        }
                        TypedDeclaration::StorageReassignment(tr) => self
                            .compile_storage_reassignment(
                                context,
                                &tr.fields,
                                &tr.ix,
                                &tr.rhs,
                                span_md_idx,
                            ),
                        TypedDeclaration::ImplTrait(TypedImplTrait { span, .. }) => {
                            // XXX What if we ignore the trait implementation???  Potentially since
                            // we currently inline everything and below we 'recreate' the functions
                            // lazily as they are called, nothing needs to be done here.  BUT!
                            // This is obviously not really correct, and eventually we want to
                            // compile and then call these properly.
                            let span_md_idx = MetadataIndex::from_span(context, &span);
                            Ok(Constant::get_unit(context, span_md_idx))
                        }
                        TypedDeclaration::AbiDeclaration(_) => {
                            Err(CompileError::UnexpectedDeclaration {
                                decl_type: "abi",
                                span: ast_node.span,
                            })
                        }
                        TypedDeclaration::GenericTypeForFunctionScope { .. } => {
                            Err(CompileError::UnexpectedDeclaration {
                                decl_type: "abi",
                                span: ast_node.span,
                            })
                        }
                        TypedDeclaration::ErrorRecovery { .. } => {
                            Err(CompileError::UnexpectedDeclaration {
                                decl_type: "error recovery",
                                span: ast_node.span,
                            })
                        }
                        TypedDeclaration::StorageDeclaration(_) => {
                            Err(CompileError::UnexpectedDeclaration {
                                decl_type: "storage",
                                span: ast_node.span,
                            })
                        }
                    },
                    TypedAstNodeContent::Expression(te) => {
                        // An expression with an ignored return value... I assume.
                        self.compile_expression(context, te)
                    }
                    TypedAstNodeContent::ImplicitReturnExpression(te) => {
                        self.compile_expression(context, te)
                    }
                    TypedAstNodeContent::WhileLoop(twl) => {
                        self.compile_while_loop(context, twl, span_md_idx)
                    }
                    // a side effect can be () because it just impacts the type system/namespacing.
                    // There should be no new IR generated.
                    TypedAstNodeContent::SideEffect => Ok(Constant::get_unit(context, None)),
                }
            })
            .collect::<Result<Vec<_>, CompileError>>()
            .map(|vals| vals.last().cloned())
            .transpose()
            .unwrap_or_else(|| Ok(Constant::get_unit(context, None)));
        self.lexical_map.leave_scope();
        value
    }

    fn compile_expression(
        &mut self,
        context: &mut Context,
        ast_expr: TypedExpression,
    ) -> Result<Value, CompileError> {
        let span_md_idx = MetadataIndex::from_span(context, &ast_expr.span);
        match ast_expr.expression {
            TypedExpressionVariant::Literal(l) => {
                Ok(convert_literal_to_value(context, &l, span_md_idx))
            }
            TypedExpressionVariant::FunctionApplication {
                call_path: name,
                contract_call_params,
                arguments,
                function_body,
                function_body_name_span,
                function_body_purity,
                self_state_idx,
                selector,
            } => {
                if let Some(metadata) = selector {
                    self.compile_contract_call(
                        &metadata,
                        &contract_call_params,
                        context,
                        name.suffix.as_str(),
                        arguments,
                        ast_expr.return_type,
                        span_md_idx,
                    )
                } else {
                    self.compile_fn_call(
                        context,
                        arguments,
                        function_body,
                        function_body_name_span,
                        function_body_purity,
                        self_state_idx,
                        span_md_idx,
                    )
                }
            }
            TypedExpressionVariant::LazyOperator { op, lhs, rhs } => {
                self.compile_lazy_op(context, op, *lhs, *rhs, span_md_idx)
            }
            TypedExpressionVariant::VariableExpression { name } => {
                self.compile_var_expr(context, name.as_str(), span_md_idx)
            }
            TypedExpressionVariant::Array { contents } => {
                self.compile_array_expr(context, contents, span_md_idx)
            }
            TypedExpressionVariant::ArrayIndex { prefix, index } => {
                self.compile_array_index(context, *prefix, *index, span_md_idx)
            }
            TypedExpressionVariant::StructExpression { fields, .. } => {
                self.compile_struct_expr(context, fields, span_md_idx)
            }
            TypedExpressionVariant::CodeBlock(cb) => self.compile_code_block(context, cb),
            TypedExpressionVariant::FunctionParameter => Err(CompileError::Internal(
                "Unexpected function parameter declaration.",
                ast_expr.span,
            )),
            TypedExpressionVariant::IfExp {
                condition,
                then,
                r#else,
            } => self.compile_if(context, *condition, *then, r#else),
            TypedExpressionVariant::AsmExpression {
                registers,
                body,
                returns,
                whole_block_span,
            } => {
                let span_md_idx = MetadataIndex::from_span(context, &whole_block_span);
                self.compile_asm_expr(
                    context,
                    registers,
                    body,
                    ast_expr.return_type,
                    returns,
                    span_md_idx,
                )
            }
            TypedExpressionVariant::StructFieldAccess {
                prefix,
                field_to_access,
                resolved_type_of_parent,
            } => {
                let span_md_idx = MetadataIndex::from_span(context, &field_to_access.span);
                self.compile_struct_field_expr(
                    context,
                    *prefix,
                    resolved_type_of_parent,
                    field_to_access,
                    span_md_idx,
                )
            }
            TypedExpressionVariant::EnumInstantiation {
                enum_decl,
                tag,
                contents,
                ..
            } => self.compile_enum_expr(context, enum_decl, tag, contents),
            TypedExpressionVariant::Tuple { fields } => {
                self.compile_tuple_expr(context, fields, span_md_idx)
            }
            TypedExpressionVariant::TupleElemAccess {
                prefix,
                elem_to_access_num: idx,
                elem_to_access_span: span,
                resolved_type_of_parent: tuple_type,
            } => self.compile_tuple_elem_expr(context, *prefix, tuple_type, idx, span),
            TypedExpressionVariant::AbiCast { span, .. } => {
                let span_md_idx = MetadataIndex::from_span(context, &span);
                Ok(Constant::get_unit(context, span_md_idx))
            }
            TypedExpressionVariant::StorageAccess(access) => {
                let span_md_idx = MetadataIndex::from_span(context, &access.span());
                self.compile_storage_access(context, &access.fields, &access.ix, span_md_idx)
            }
            TypedExpressionVariant::IntrinsicFunction(kind) => {
                self.compile_intrinsic_function(context, kind, ast_expr.span)
            }
            TypedExpressionVariant::AbiName(_) => {
                Ok(Value::new_constant(context, Constant::new_unit(), None))
            }
            TypedExpressionVariant::UnsafeDowncast { exp, variant } => {
                self.compile_unsafe_downcast(context, exp, variant)
            }
            TypedExpressionVariant::EnumTag { exp } => self.compile_enum_tag(context, exp),
        }
    }

    fn compile_intrinsic_function(
        &mut self,
        context: &mut Context,
        kind: TypedIntrinsicFunctionKind,
        span: Span,
    ) -> Result<Value, CompileError> {
        match kind {
            TypedIntrinsicFunctionKind::SizeOfVal { exp } => {
                // Compile the expression in case of side-effects but ignore its value.
                let ir_type = convert_resolved_typeid(context, &exp.return_type, &exp.span)?;
                self.compile_expression(context, *exp)?;
                Ok(Constant::get_uint(
                    context,
                    64,
                    ir_type_size_in_bytes(context, &ir_type),
                    None,
                ))
            }
            TypedIntrinsicFunctionKind::SizeOfType { type_id, type_span } => {
                let ir_type = convert_resolved_typeid(context, &type_id, &type_span)?;
                Ok(Constant::get_uint(
                    context,
                    64,
                    ir_type_size_in_bytes(context, &ir_type),
                    None,
                ))
            }
            TypedIntrinsicFunctionKind::IsRefType { type_id, type_span } => {
                let ir_type = convert_resolved_typeid(context, &type_id, &type_span)?;
                Ok(Constant::get_bool(context, !ir_type.is_copy_type(), None))
            }
            TypedIntrinsicFunctionKind::GetStorageKey => {
                let span_md_idx = MetadataIndex::from_span(context, &span);
                Ok(self
                    .current_block
                    .ins(context)
                    .get_storage_key(span_md_idx, None))
            }
        }
    }

    fn compile_return_statement(
        &mut self,
        context: &mut Context,
        ast_expr: TypedExpression,
    ) -> Result<Value, CompileError> {
        let ret_value = self.compile_expression(context, ast_expr.clone())?;
        match ret_value.get_type(context) {
            None => Err(CompileError::Internal(
                "Unable to determine type for return statement expression.",
                ast_expr.span,
            )),
            Some(ret_ty) => {
                let span_md_idx = MetadataIndex::from_span(context, &ast_expr.span);
                self.current_block
                    .ins(context)
                    .ret(ret_value, ret_ty, span_md_idx);
                Ok(Constant::get_unit(context, span_md_idx))
            }
        }
    }

    fn compile_lazy_op(
        &mut self,
        context: &mut Context,
        ast_op: LazyOp,
        ast_lhs: TypedExpression,
        ast_rhs: TypedExpression,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // Short-circuit: if LHS is true for AND we still must eval the RHS block; for OR we can
        // skip the RHS block, and vice-versa.
        let lhs_val = self.compile_expression(context, ast_lhs)?;
        let rhs_block = self.function.create_block(context, None);
        let final_block = self.function.create_block(context, None);
        let cond_builder = self.current_block.ins(context);
        match ast_op {
            LazyOp::And => cond_builder.conditional_branch(
                lhs_val,
                rhs_block,
                final_block,
                Some(lhs_val),
                span_md_idx,
            ),
            LazyOp::Or => cond_builder.conditional_branch(
                lhs_val,
                final_block,
                rhs_block,
                Some(lhs_val),
                span_md_idx,
            ),
        };

        self.current_block = rhs_block;
        let rhs_val = self.compile_expression(context, ast_rhs)?;
        self.current_block
            .ins(context)
            .branch(final_block, Some(rhs_val), span_md_idx);

        self.current_block = final_block;
        Ok(final_block.get_phi(context))
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_contract_call(
        &mut self,
        metadata: &ContractCallMetadata,
        contract_call_parameters: &HashMap<String, TypedExpression>,
        context: &mut Context,
        ast_name: &str,
        ast_args: Vec<(Ident, TypedExpression)>,
        return_type: TypeId,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // Compile each user argument
        let compiled_args = ast_args
            .into_iter()
            .map(|(_, expr)| self.compile_expression(context, expr))
            .collect::<Result<Vec<Value>, CompileError>>()?;

        let user_args_val = match compiled_args.len() {
            0 => Constant::get_uint(context, 64, 0, None),
            1 => {
                // The single arg doesn't need to be put into a struct.
                let arg0 = compiled_args[0];

                // We're still undecided as to whether this should be decided by type or size.
                // Going with type for now.
                let arg0_type = arg0.get_type(context).unwrap();
                if arg0_type.is_copy_type() {
                    self.current_block
                        .ins(context)
                        .bitcast(arg0, Type::Uint(64), span_md_idx)
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

                    let arg0_ptr = self.current_block.ins(context).get_ptr(
                        by_reference_arg,
                        arg0_type,
                        0,
                        None,
                    );
                    self.current_block.ins(context).store(arg0_ptr, arg0, None);

                    // NOTE: Here we're fetching the original stack pointer, cast to u64.
                    // TODO: Instead of casting here, we should use an `ptrtoint` instruction.
                    self.current_block.ins(context).get_ptr(
                        by_reference_arg,
                        Type::Uint(64),
                        0,
                        span_md_idx,
                    )
                }
            }
            _ => {
                // New struct type to hold the user arguments bundled together.
                let field_types = compiled_args
                    .iter()
                    .map(|val| val.get_type(context).unwrap())
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
                    self.current_block.ins(context).get_ptr(
                        user_args_struct_ptr,
                        Type::Struct(user_args_struct_aggregate),
                        0,
                        span_md_idx,
                    ),
                    |user_args_struct_ptr_val, (insert_idx, insert_val)| {
                        self.current_block.ins(context).insert_value(
                            user_args_struct_ptr_val,
                            user_args_struct_aggregate,
                            insert_val,
                            vec![insert_idx as u64],
                            span_md_idx,
                        )
                    },
                );

                // NOTE: Here we're fetching the original stack pointer, cast to u64.
                self.current_block.ins(context).get_ptr(
                    user_args_struct_ptr,
                    Type::Uint(64),
                    0,
                    span_md_idx,
                )
            }
        };

        // Now handle the contract address and the selector. The contract address is just
        // as B256 while the selector is a [u8; 4] which we have to convert to a U64.
        let ra_struct_aggregate = Aggregate::new_struct(
            context,
            [Type::B256, Type::Uint(64), Type::Uint(64)].to_vec(),
        );

        let addr = self.compile_expression(context, *metadata.contract_address.clone())?;
        let mut ra_struct_val =
            Constant::get_undef(context, Type::Struct(ra_struct_aggregate), span_md_idx);

        // Insert the contract address
        ra_struct_val = self.current_block.ins(context).insert_value(
            ra_struct_val,
            ra_struct_aggregate,
            addr,
            vec![0],
            span_md_idx,
        );

        // Convert selector to U64 and then insert it
        let sel = metadata.func_selector;
        let sel_val = convert_literal_to_value(
            context,
            &Literal::U64(
                sel[3] as u64 + 256 * (sel[2] as u64 + 256 * (sel[1] as u64 + 256 * sel[0] as u64)),
            ),
            span_md_idx,
        );
        ra_struct_val = self.current_block.ins(context).insert_value(
            ra_struct_val,
            ra_struct_aggregate,
            sel_val,
            vec![1],
            span_md_idx,
        );

        // Insert the user args value.

        ra_struct_val = self.current_block.ins(context).insert_value(
            ra_struct_val,
            ra_struct_aggregate,
            user_args_val,
            vec![2],
            span_md_idx,
        );

        // Compile all other metadata parameters
        let coins = match contract_call_parameters
            .get(&constants::CONTRACT_CALL_COINS_PARAMETER_NAME.to_string())
        {
            Some(coins_expr) => self.compile_expression(context, coins_expr.clone())?,
            None => convert_literal_to_value(
                context,
                &Literal::U64(constants::CONTRACT_CALL_COINS_PARAMETER_DEFAULT_VALUE),
                span_md_idx,
            ),
        };

        let asset_id = match contract_call_parameters
            .get(&constants::CONTRACT_CALL_ASSET_ID_PARAMETER_NAME.to_string())
        {
            Some(asset_id_expr) => self.compile_expression(context, asset_id_expr.clone())?,
            None => convert_literal_to_value(
                context,
                &Literal::B256(constants::CONTRACT_CALL_ASSET_ID_PARAMETER_DEFAULT_VALUE),
                span_md_idx,
            ),
        };

        let gas = match contract_call_parameters
            .get(&constants::CONTRACT_CALL_GAS_PARAMETER_NAME.to_string())
        {
            Some(gas_expr) => self.compile_expression(context, gas_expr.clone())?,
            None => self
                .current_block
                .ins(context)
                .read_register(sway_ir::Register::Cgas, span_md_idx),
        };

        let return_type = convert_resolved_typeid_no_span(context, &return_type)?;

        // Insert the contract_call instruction
        Ok(self.current_block.ins(context).contract_call(
            return_type,
            ast_name.to_string(),
            ra_struct_val,
            coins,
            asset_id,
            gas,
            span_md_idx,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_fn_call(
        &mut self,
        context: &mut Context,
        ast_args: Vec<(Ident, TypedExpression)>,
        callee_body: TypedCodeBlock,
        callee_span: Span,
        callee_purity: Purity,
        self_state_idx: Option<StateIndex>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // XXX OK, now, the old compiler inlines everything very lazily.  Function calls include
        // the body of the callee (i.e., the callee_body arg above) and so codegen just pulled it
        // straight in, no questions asked.  Library functions are provided in an initial namespace
        // from Forc and when the parser builds the AST (or is it during type checking?) these
        // function bodies are embedded.
        //
        // We're going to build little single-use instantiations of the callee and then call them.
        // For now if they're called in multiple places they'll be redundantly recreated, but also
        // at present we are still inlining everything so it actually makes little difference.
        //
        // Eventually we need to Do It Properly and inline only when necessary, and compile the
        // standard library to an actual module.

        {
            // Firstly create the single-use callee by fudging an AST declaration.
            let callee_name = context.get_unique_name();
            let callee_name_len = callee_name.len();
            let callee_ident = Ident::new(
                crate::span::Span::new(Arc::from(callee_name), 0, callee_name_len, None).unwrap(),
            );

            // TODO: `is_mutable` below is set to `false` regardless of the actual mutability of
            // each arg. This is hacky but not too important at the moment. Mutability is only
            // relevant (currently) during type checking and so this just works. Long term, we need
            // to propagate mutability for arguments in IR and make sure that the verifier takes it
            // into account.
            let parameters = ast_args
                .iter()
                .map(|(name, expr)| TypedFunctionParameter {
                    name: name.clone(),
                    is_mutable: false,
                    type_id: expr.return_type,
                    type_span: crate::span::Span::new(" ".into(), 0, 0, None).unwrap(),
                })
                .collect();

            // We're going to have to reverse engineer the return type.
            let return_type = Self::get_codeblock_return_type(&callee_body).unwrap_or_else(||
                    // This code block is missing a return or implicit return.  The only time I've
                    // seen it happen (whether it's 'valid' or not) is in std::storage::store(),
                    // which has a single asm block which also returns nothing.  In this case, it
                    // actually is Unit.
                    insert_type(TypeInfo::Tuple(Vec::new())));

            let callee_fn_decl = TypedFunctionDeclaration {
                name: callee_ident,
                body: callee_body,
                parameters,
                span: callee_span,
                return_type,
                type_parameters: Vec::new(),
                return_type_span: crate::span::Span::new(" ".into(), 0, 0, None).unwrap(),
                visibility: Visibility::Private,
                is_contract_call: false,
                purity: callee_purity,
            };

            let callee = compile_function(context, self.module, callee_fn_decl)?;

            // Now actually call the new function.
            let args = ast_args
                .into_iter()
                .map(|(_, expr)| self.compile_expression(context, expr))
                .collect::<Result<Vec<Value>, CompileError>>()?;
            let state_idx_md_idx = match self_state_idx {
                Some(self_state_idx) => {
                    MetadataIndex::from_state_idx(context, self_state_idx.to_usize())
                }
                None => None,
            };
            Ok(self.current_block.ins(context).call(
                callee.unwrap(),
                &args,
                span_md_idx,
                state_idx_md_idx,
            ))
        }
    }

    fn get_codeblock_return_type(codeblock: &TypedCodeBlock) -> Option<TypeId> {
        if codeblock.contents.is_empty() {
            Some(insert_type(TypeInfo::Tuple(Vec::new())))
        } else {
            codeblock.contents.iter().find_map(|node| {
                match node.gather_return_statements().first() {
                    Some(TypedReturnStatement { expr }) => Some(expr.return_type),
                    None => match &node.content {
                        TypedAstNodeContent::ImplicitReturnExpression(te) => Some(te.return_type),
                        _otherwise => None,
                    },
                }
            })
        }
    }

    fn compile_if(
        &mut self,
        context: &mut Context,
        ast_condition: TypedExpression,
        ast_then: TypedExpression,
        ast_else: Option<Box<TypedExpression>>,
    ) -> Result<Value, CompileError> {
        // Compile the condition expression in the entry block.  Then save the current block so we
        // can jump to the true and false blocks after we've created them.
        let cond_span_md_idx = MetadataIndex::from_span(context, &ast_condition.span);
        let cond_value = self.compile_expression(context, ast_condition)?;
        let entry_block = self.current_block;

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
        let true_value = self.compile_expression(context, ast_then)?;
        let true_block_end = self.current_block;
        let then_returns = true_block_end.is_terminated_by_ret(context);

        let false_block_begin = self.function.create_block(context, None);
        self.current_block = false_block_begin;
        let false_value = match ast_else {
            None => Constant::get_unit(context, None),
            Some(expr) => self.compile_expression(context, *expr)?,
        };
        let false_block_end = self.current_block;
        let else_returns = false_block_end.is_terminated_by_ret(context);

        entry_block.ins(context).conditional_branch(
            cond_value,
            true_block_begin,
            false_block_begin,
            None,
            cond_span_md_idx,
        );

        if then_returns && else_returns {
            return Ok(Constant::get_unit(context, None));
        }

        let merge_block = self.function.create_block(context, None);
        if !then_returns {
            true_block_end
                .ins(context)
                .branch(merge_block, Some(true_value), None);
        }
        if !else_returns {
            false_block_end
                .ins(context)
                .branch(merge_block, Some(false_value), None);
        }

        self.current_block = merge_block;
        if !then_returns || !else_returns {
            Ok(merge_block.get_phi(context))
        } else {
            Ok(Constant::get_unit(context, None))
        }
    }

    fn compile_unsafe_downcast(
        &mut self,
        context: &mut Context,
        exp: Box<TypedExpression>,
        variant: TypedEnumVariant,
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
        let compiled_value = self.compile_expression(context, *exp)?;
        // retrieve the value minus the tag
        Ok(self.current_block.ins(context).extract_value(
            compiled_value,
            enum_aggregate,
            vec![1, variant.tag as u64],
            None,
        ))
    }

    fn compile_enum_tag(
        &mut self,
        context: &mut Context,
        exp: Box<TypedExpression>,
    ) -> Result<Value, CompileError> {
        let tag_span_md_idx = MetadataIndex::from_span(context, &exp.span);
        let enum_aggregate = match convert_resolved_typeid(context, &exp.return_type, &exp.span)? {
            Type::Struct(aggregate) => aggregate,
            _ => {
                return Err(CompileError::Internal("Expected enum type here.", exp.span));
            }
        };
        let exp = self.compile_expression(context, *exp)?;
        Ok(self.current_block.ins(context).extract_value(
            exp,
            enum_aggregate,
            vec![0],
            tag_span_md_idx,
        ))
    }

    fn compile_while_loop(
        &mut self,
        context: &mut Context,
        ast_while_loop: TypedWhileLoop,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // We're dancing around a bit here to make the blocks sit in the right order.  Ideally we
        // have the cond block, followed by the body block which may contain other blocks, and the
        // final block comes after any body block(s).

        // Jump to the while cond block.
        let cond_block = self.function.create_block(context, Some("while".into()));
        self.current_block
            .ins(context)
            .branch(cond_block, None, None);

        // Fill in the body block now, jump unconditionally to the cond block at its end.
        let body_block = self
            .function
            .create_block(context, Some("while_body".into()));
        self.current_block = body_block;
        self.compile_code_block(context, ast_while_loop.body)?;
        self.current_block
            .ins(context)
            .branch(cond_block, None, None);

        // Create the final block after we're finished with the body.
        let final_block = self
            .function
            .create_block(context, Some("end_while".into()));

        // Add the conditional which jumps into the body or out to the final block.
        self.current_block = cond_block;
        let cond_value = self.compile_expression(context, ast_while_loop.condition)?;
        self.current_block.ins(context).conditional_branch(
            cond_value,
            body_block,
            final_block,
            None,
            None,
        );

        self.current_block = final_block;
        Ok(Constant::get_unit(context, span_md_idx))
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
                .get_ptr(ptr, ptr_ty, 0, span_md_idx);
            Ok(if ptr.is_aggregate_ptr(context) {
                ptr_val
            } else {
                self.current_block.ins(context).load(ptr_val, span_md_idx)
            })
        } else if let Some(val) = self.function.get_arg(context, name) {
            Ok(val)
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
        ast_var_decl: TypedVariableDeclaration,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let TypedVariableDeclaration {
            name,
            body,
            is_mutable,
            ..
        } = ast_var_decl;
        // Nothing to do for an abi cast declarations. The address specified in them is already
        // provided in each contract call node in the AST.
        if matches!(
            &resolve_type(body.return_type, &body.span).map_err(|ty_err| {
                CompileError::InternalOwned(format!("{:?}", ty_err), body.span.clone())
            })?,
            TypeInfo::ContractCaller { .. }
        ) {
            return Ok(Constant::get_unit(context, span_md_idx));
        }

        // Grab these before we move body into compilation.
        let return_type = convert_resolved_typeid(context, &body.return_type, &body.span)?;

        // We must compile the RHS before checking for shadowing, as it will still be in the
        // previous scope.
        let init_val = self.compile_expression(context, body)?;
        let local_name = self.lexical_map.insert(name.as_str().to_owned());
        let ptr = self
            .function
            .new_local_ptr(context, local_name, return_type, is_mutable.into(), None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        // We can have empty aggregates, especially arrays, which shouldn't be initialised, but
        // otherwise use a store.
        let ptr_ty = *ptr.get_type(context);
        if ir_type_size_in_bytes(context, &ptr_ty) > 0 {
            let ptr_val = self
                .current_block
                .ins(context)
                .get_ptr(ptr, ptr_ty, 0, span_md_idx);
            self.current_block
                .ins(context)
                .store(ptr_val, init_val, span_md_idx);
        }
        Ok(init_val)
    }

    fn compile_const_decl(
        &mut self,
        context: &mut Context,
        ast_const_decl: TypedConstantDeclaration,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // This is local to the function, so we add it to the locals, rather than the module
        // globals like other const decls.
        let TypedConstantDeclaration { name, value, .. } = ast_const_decl;

        if let TypedExpressionVariant::Literal(literal) = &value.expression {
            let initialiser = convert_literal_to_constant(literal);
            let return_type = convert_resolved_typeid(context, &value.return_type, &value.span)?;
            let name = name.as_str().to_owned();
            self.function
                .new_local_ptr(context, name.clone(), return_type, false, Some(initialiser))
                .map_err(|ir_error| {
                    CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                })?;

            // We still insert this into the symbol table, as itself... can they be shadowed?
            // (Hrmm, name resolution in the variable expression code could be smarter about var
            // decls vs const decls, for now they're essentially the same...)
            self.lexical_map.insert(name);

            Ok(Constant::get_unit(context, span_md_idx))
        } else {
            Err(CompileError::Internal(
                "Unsupported constant declaration type, expecting a literal.",
                name.span(),
            ))
        }
    }

    fn compile_reassignment(
        &mut self,
        context: &mut Context,
        ast_reassignment: TypedReassignment,
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
                    .get_ptr(ptr, ptr_ty, 0, span_md_idx)
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

        let reassign_val = self.compile_expression(context, ast_reassignment.rhs)?;

        if ast_reassignment.lhs_indices.is_empty() {
            // A non-aggregate; use a `store`.
            self.current_block
                .ins(context)
                .store(val, reassign_val, span_md_idx);
        } else {
            // An aggregate.  Iterate over the field names from the left hand side and collect
            // field indices.  The struct type from the previous iteration is used to determine the
            // field type for the current iteration.
            let field_idcs = get_indices_for_struct_access(
                ast_reassignment.lhs_type,
                &ast_reassignment.lhs_indices,
            )?;

            let ty = match val.get_type(context).unwrap() {
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

            self.current_block.ins(context).insert_value(
                val,
                ty,
                reassign_val,
                field_idcs,
                span_md_idx,
            );
        }

        // This shouldn't really return a value, it doesn't make sense to return the `store` or
        // `insert_value` instruction, but we need to return something at this stage.
        Ok(reassign_val)
    }

    fn compile_storage_reassignment(
        &mut self,
        context: &mut Context,
        fields: &[TypeCheckedStorageReassignDescriptor],
        ix: &StateIndex,
        rhs: &TypedExpression,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // Compile the RHS into a value
        let rhs = self.compile_expression(context, rhs.clone())?;

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
        self.compile_storage_read_or_write(
            context,
            &StateAccessType::Write,
            ix,
            field_idcs,
            &access_type,
            &Some(rhs),
            span_md_idx,
        )
    }

    fn compile_array_expr(
        &mut self,
        context: &mut Context,
        contents: Vec<TypedExpression>,
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
        let array_value = Constant::get_undef(context, Type::Array(aggregate), span_md_idx);
        contents
            .into_iter()
            .enumerate()
            .fold(Ok(array_value), |array_value, (idx, elem_expr)| {
                // Result::flatten() is currently nightly only.
                match array_value {
                    Err(_) => array_value,
                    Ok(array_value) => {
                        let index_val = Constant::get_uint(context, 64, idx as u64, span_md_idx);
                        self.compile_expression(context, elem_expr)
                            .map(|elem_value| {
                                self.current_block.ins(context).insert_element(
                                    array_value,
                                    aggregate,
                                    elem_value,
                                    index_val,
                                    span_md_idx,
                                )
                            })
                    }
                }
            })
    }

    fn compile_array_index(
        &mut self,
        context: &mut Context,
        array_expr: TypedExpression,
        index_expr: TypedExpression,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let array_expr_span = array_expr.span.clone();
        let array_val = self.compile_expression(context, array_expr)?;
        let aggregate = match &context.values[array_val.0].value {
            ValueDatum::Instruction(instruction) => {
                instruction.get_aggregate(context).ok_or_else(|| {
                    CompileError::InternalOwned(format!(
                        "Unsupported instruction as array value for index expression. {instruction:?}"),
                        array_expr_span)
                })
            }
            ValueDatum::Argument(Type::Array(aggregate))
            | ValueDatum::Constant(Constant { ty : Type::Array(aggregate), ..}) => Ok (*aggregate),
            otherwise => Err(CompileError::InternalOwned(
                format!("Unsupported array value for index expression: {otherwise:?}"),
                array_expr_span,
            )),
        }?;

        // Check for out of bounds if we have a literal index.
        let (_, count) = context.aggregates[aggregate.0].array_type();
        if let TypedExpressionVariant::Literal(Literal::U64(index)) = index_expr.expression {
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

        let index_val = self.compile_expression(context, index_expr)?;

        Ok(self.current_block.ins(context).extract_element(
            array_val,
            aggregate,
            index_val,
            span_md_idx,
        ))
    }

    fn compile_struct_expr(
        &mut self,
        context: &mut Context,
        fields: Vec<TypedStructExpressionField>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        // NOTE: This is a struct instantiation with initialisers for each field of a named struct.
        // We don't know the actual type of the struct, but the AST guarantees that the fields are
        // in the declared order (regardless of how they are initialised in source) so we can
        // create an aggregate with the field types to construct the struct value.

        // Compile each of the values for field initialisers, calculate their indices and also
        // gather their types with which to make an aggregate.
        let field_descrs = fields
            .into_iter()
            .enumerate()
            .map(|(insert_idx, struct_field)| {
                let field_ty = struct_field.value.return_type;
                self.compile_expression(context, struct_field.value)
                    .map(|insert_val| ((insert_val, insert_idx as u64), field_ty))
            })
            .collect::<Result<Vec<_>, CompileError>>()?;
        let (inserted_values_indices, field_types): (Vec<(Value, u64)>, Vec<TypeId>) =
            field_descrs.into_iter().unzip();

        // Start with a constant empty struct and then fill in the values.
        let aggregate = get_aggregate_for_types(context, &field_types)?;
        let agg_value = Constant::get_undef(context, Type::Struct(aggregate), span_md_idx);
        Ok(inserted_values_indices.into_iter().fold(
            agg_value,
            |agg_value, (insert_val, insert_idx)| {
                self.current_block.ins(context).insert_value(
                    agg_value,
                    aggregate,
                    insert_val,
                    vec![insert_idx],
                    span_md_idx,
                )
            },
        ))
    }

    fn compile_struct_field_expr(
        &mut self,
        context: &mut Context,
        ast_struct_expr: TypedExpression,
        struct_type_id: TypeId,
        ast_field: TypedStructField,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let ast_struct_expr_span = ast_struct_expr.span.clone();
        let struct_val = self.compile_expression(context, ast_struct_expr)?;
        let aggregate = match &context.values[struct_val.0].value {
            ValueDatum::Instruction(instruction) => {
                instruction.get_aggregate(context).ok_or_else(|| {
                    CompileError::InternalOwned(
                        format!(
                            "Unsupported instruction as struct value for \
                            field expression: {instruction:?}",
                        ),
                        ast_struct_expr_span,
                    )
                })
            }
            ValueDatum::Argument(Type::Struct(aggregate))
            | ValueDatum::Constant(Constant {
                ty: Type::Struct(aggregate),
                ..
            }) => Ok(*aggregate),
            otherwise => Err(CompileError::InternalOwned(
                format!("Unsupported struct value for field expression: {otherwise:?}",),
                ast_struct_expr_span,
            )),
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

        Ok(self.current_block.ins(context).extract_value(
            struct_val,
            aggregate,
            vec![field_idx],
            span_md_idx,
        ))
    }

    fn compile_enum_expr(
        &mut self,
        context: &mut Context,
        enum_decl: TypedEnumDeclaration,
        tag: usize,
        contents: Option<Box<TypedExpression>>,
    ) -> Result<Value, CompileError> {
        // XXX The enum instantiation AST node includes the full declaration.  If the enum was
        // declared in a different module then it seems for now there's no easy way to pre-analyse
        // it and add its type/aggregate to the context.  We can re-use them here if we recognise
        // the name, and if not add a new aggregate... OTOH the naming seems a little fragile and
        // we could potentially use the wrong aggregate with the same name, different module...
        // dunno.
        let span_md_idx = MetadataIndex::from_span(context, &enum_decl.span);
        let aggregate = create_enum_aggregate(context, enum_decl.variants)?;
        let tag_value = Constant::get_uint(context, 64, tag as u64, span_md_idx);

        // Start with the undef and insert the tag.
        let agg_value = Constant::get_undef(context, Type::Struct(aggregate), span_md_idx);
        let agg_value = self.current_block.ins(context).insert_value(
            agg_value,
            aggregate,
            tag_value,
            vec![0],
            span_md_idx,
        );

        // If the struct representing the enum has only one field, then that field is basically the
        // tag and all the variants must have unit types, hence the absence of the union.
        // Therefore, there is no need for another `insert_value` instruction here.
        match &context.aggregates[aggregate.0] {
            AggregateContent::FieldTypes(field_tys) => {
                Ok(if field_tys.len() == 1 {
                    agg_value
                } else {
                    match contents {
                        None => agg_value,
                        Some(te) => {
                            // Insert the value too.
                            let contents_value = self.compile_expression(context, *te)?;
                            self.current_block.ins(context).insert_value(
                                agg_value,
                                aggregate,
                                contents_value,
                                vec![1],
                                span_md_idx,
                            )
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
        fields: Vec<TypedExpression>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        if fields.is_empty() {
            // This is a Unit.  We're still debating whether Unit should just be an empty tuple in
            // the IR or not... it is a special case for now.
            Ok(Constant::get_unit(context, span_md_idx))
        } else {
            let (init_values, init_types): (Vec<Value>, Vec<Type>) = fields
                .into_iter()
                .map(|field_expr| {
                    convert_resolved_typeid_no_span(context, &field_expr.return_type).and_then(
                        |init_type| {
                            self.compile_expression(context, field_expr)
                                .map(|init_value| (init_value, init_type))
                        },
                    )
                })
                .collect::<Result<Vec<_>, CompileError>>()?
                .into_iter()
                .unzip();

            let aggregate = Aggregate::new_struct(context, init_types);
            let agg_value = Constant::get_undef(context, Type::Struct(aggregate), span_md_idx);

            Ok(init_values.into_iter().enumerate().fold(
                agg_value,
                |agg_value, (insert_idx, insert_val)| {
                    self.current_block.ins(context).insert_value(
                        agg_value,
                        aggregate,
                        insert_val,
                        vec![insert_idx as u64],
                        span_md_idx,
                    )
                },
            ))
        }
    }

    fn compile_tuple_elem_expr(
        &mut self,
        context: &mut Context,
        tuple: TypedExpression,
        tuple_type: TypeId,
        idx: usize,
        span: Span,
    ) -> Result<Value, CompileError> {
        let tuple_value = self.compile_expression(context, tuple)?;
        if let Type::Struct(aggregate) = convert_resolved_typeid(context, &tuple_type, &span)? {
            let span_md_idx = MetadataIndex::from_span(context, &span);
            Ok(self.current_block.ins(context).extract_value(
                tuple_value,
                aggregate,
                vec![idx as u64],
                span_md_idx,
            ))
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
        fields: &[TypeCheckedStorageAccessDescriptor],
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
        self.compile_storage_read_or_write(
            context,
            &StateAccessType::Read,
            ix,
            field_idcs,
            &access_type,
            &None,
            span_md_idx,
        )
    }

    fn compile_asm_expr(
        &mut self,
        context: &mut Context,
        registers: Vec<TypedAsmRegisterDeclaration>,
        body: Vec<AsmOp>,
        return_type: TypeId,
        returns: Option<(AsmRegister, Span)>,
        whole_block_span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let registers = registers
            .into_iter()
            .map(
                |TypedAsmRegisterDeclaration {
                     initializer, name, ..
                 }| {
                    // Take the optional initialiser, map it to an Option<Result<Value>>,
                    // transpose that to Result<Option<Value>> and map that to an AsmArg.
                    initializer
                        .map(|init_expr| self.compile_expression(context, init_expr))
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
                    span_md_idx: MetadataIndex::from_span(context, &span),
                },
            )
            .collect();
        let returns = returns
            .as_ref()
            .map(|(_, asm_reg_span)| Ident::new(asm_reg_span.clone()));
        let return_type = convert_resolved_typeid_no_span(context, &return_type)?;
        Ok(self.current_block.ins(context).asm_block(
            registers,
            body,
            return_type,
            returns,
            whole_block_span_md_idx,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_storage_read_or_write(
        &mut self,
        context: &mut Context,
        access_type: &StateAccessType,
        ix: &StateIndex,
        indices: Vec<u64>,
        r#type: &Type,
        rhs: &Option<Value>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        match r#type {
            Type::Struct(aggregate) => {
                let mut struct_val =
                    Constant::get_undef(context, Type::Struct(*aggregate), span_md_idx);

                let fields = context.aggregates[aggregate.0].field_types().clone();
                for (field_idx, field_type) in fields.into_iter().enumerate() {
                    let field_idx = field_idx as u64;

                    // Recurse. The base case is for primitive types that fit in a single storage slot.
                    let mut new_indices = indices.clone();
                    new_indices.push(field_idx);

                    match access_type {
                        StateAccessType::Read => {
                            let val_to_insert = self.compile_storage_read_or_write(
                                context,
                                access_type,
                                ix,
                                new_indices,
                                &field_type,
                                rhs,
                                span_md_idx,
                            )?;

                            //  Insert the loaded value to the aggregate at the given index
                            struct_val = self.current_block.ins(context).insert_value(
                                struct_val,
                                *aggregate,
                                val_to_insert,
                                vec![field_idx],
                                span_md_idx,
                            );
                        }
                        StateAccessType::Write => {
                            // Extract the value from the aggregate at the given index
                            let rhs = self.current_block.ins(context).extract_value(
                                rhs.expect("expecting a rhs for write"),
                                *aggregate,
                                vec![field_idx],
                                span_md_idx,
                            );

                            self.compile_storage_read_or_write(
                                context,
                                access_type,
                                ix,
                                new_indices,
                                &field_type,
                                &Some(rhs),
                                span_md_idx,
                            )?;
                        }
                    }
                }
                Ok(struct_val)
            }
            _ => {
                // Calculate the storage location hash for the given field
                let mut storage_slot_to_hash = format!(
                    "{}{}",
                    sway_utils::constants::STORAGE_DOMAIN_SEPARATOR,
                    ix.to_usize()
                );
                for ix in &indices {
                    storage_slot_to_hash = format!("{}_{}", storage_slot_to_hash, ix);
                }
                let hashed_storage_slot = Hasher::hash(storage_slot_to_hash);

                // New name for the key
                let mut key_name = format!("{}{}", "key_for_", ix.to_usize());
                for ix in &indices {
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
                let const_key = convert_literal_to_value(
                    context,
                    &Literal::B256(hashed_storage_slot.into()),
                    span_md_idx,
                );

                // Convert the key pointer to a value using get_ptr
                let key_ptr_ty = *key_ptr.get_type(context);
                let mut key_ptr_val =
                    self.current_block
                        .ins(context)
                        .get_ptr(key_ptr, key_ptr_ty, 0, span_md_idx);

                // Store the const hash value to the key pointer value
                self.current_block
                    .ins(context)
                    .store(key_ptr_val, const_key, span_md_idx);

                match r#type {
                    Type::Array(_) => Err(CompileError::Internal(
                        "Arrays in storage have not been implemented yet.",
                        Span::dummy(),
                    )),
                    Type::B256 => self.compile_b256_storage(
                        context,
                        access_type,
                        ix,
                        &indices,
                        &key_ptr_val,
                        r#type,
                        rhs,
                        span_md_idx,
                    ),
                    Type::Bool | Type::Uint(_) => self.compile_uint_or_bool_storage(
                        context,
                        access_type,
                        &key_ptr_val,
                        r#type,
                        rhs,
                        span_md_idx,
                    ),
                    Type::String(_) | Type::Union(_) => self.compile_union_or_string_storage(
                        context,
                        access_type,
                        ix,
                        &indices,
                        &mut key_ptr_val,
                        &key_ptr,
                        &hashed_storage_slot,
                        r#type,
                        rhs,
                        span_md_idx,
                    ),
                    Type::Struct(_) => unreachable!("structs are already handled!"),
                    Type::Unit => Ok(Constant::get_unit(context, span_md_idx)),
                }
            }
        }
    }

    fn compile_uint_or_bool_storage(
        &mut self,
        context: &mut Context,
        access_type: &StateAccessType,
        key_ptr_val: &Value,
        r#type: &Type,
        rhs: &Option<Value>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        Ok(match access_type {
            StateAccessType::Read => {
                // `state_load_word` always returns a `u64`. Cast the result back
                // to the right type before returning
                let load_val = self
                    .current_block
                    .ins(context)
                    .state_load_word(*key_ptr_val, span_md_idx);
                self.current_block
                    .ins(context)
                    .bitcast(load_val, *r#type, span_md_idx)
            }
            StateAccessType::Write => {
                // `state_store_word` requires a `u64`. Cast the value to store to
                // `u64` first before actually storing.
                let rhs_u64 = self.current_block.ins(context).bitcast(
                    rhs.expect("expecting a rhs for write"),
                    Type::Uint(64),
                    span_md_idx,
                );
                self.current_block.ins(context).state_store_word(
                    rhs_u64,
                    *key_ptr_val,
                    span_md_idx,
                );
                rhs.expect("expecting a rhs for write")
            }
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_b256_storage(
        &mut self,
        context: &mut Context,
        access_type: &StateAccessType,
        ix: &StateIndex,
        indices: &Vec<u64>,
        key_ptr_val: &Value,
        r#type: &Type,
        rhs: &Option<Value>,
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
            .new_local_ptr(context, alias_value_name, *r#type, true, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        // Convert the local pointer created to a value using get_ptr
        let value_ptr_val =
            self.current_block
                .ins(context)
                .get_ptr(value_ptr, *r#type, 0, span_md_idx);

        match access_type {
            StateAccessType::Read => {
                self.current_block.ins(context).state_load_quad_word(
                    value_ptr_val,
                    *key_ptr_val,
                    span_md_idx,
                );
                Ok(value_ptr_val)
            }
            StateAccessType::Write => {
                // Store the value to the local pointer created for rhs
                self.current_block.ins(context).store(
                    value_ptr_val,
                    rhs.expect("expecting a rhs for write"),
                    span_md_idx,
                );

                // Finally, just call state_load_quad_word/state_store_quad_word
                self.current_block.ins(context).state_store_quad_word(
                    value_ptr_val,
                    *key_ptr_val,
                    span_md_idx,
                );
                Ok(rhs.expect("expecting a rhs for write"))
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_union_or_string_storage(
        &mut self,
        context: &mut Context,
        access_type: &StateAccessType,
        ix: &StateIndex,
        indices: &[u64],
        key_ptr_val: &mut Value,
        key_ptr: &Pointer,
        hashed_storage_slot: &fuel_types::Bytes32,
        r#type: &Type,
        rhs: &Option<Value>,
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
        let value_ptr_val =
            self.current_block
                .ins(context)
                .get_ptr(value_ptr, *r#type, 0, span_md_idx);

        if rhs.is_some() {
            // Store the value to the local pointer created for rhs
            self.current_block.ins(context).store(
                value_ptr_val,
                rhs.expect("expecting a rhs for write"),
                span_md_idx,
            );
        }

        for array_index in 0..number_of_elements {
            if array_index > 0 {
                // Prepare key for the next iteration but not for array index 0
                // because the first key was generated earlier.
                // Const value for the key from the initial hash + array_index
                let const_key = convert_literal_to_value(
                    context,
                    &Literal::B256(*add_to_b256(*hashed_storage_slot, array_index)),
                    span_md_idx,
                );

                // Convert the key pointer to a value using get_ptr
                let key_ptr_ty = *key_ptr.get_type(context);
                *key_ptr_val =
                    self.current_block
                        .ins(context)
                        .get_ptr(*key_ptr, key_ptr_ty, 0, span_md_idx);

                // Store the const hash value to the key pointer value
                self.current_block
                    .ins(context)
                    .store(*key_ptr_val, const_key, span_md_idx);
            }

            // Get the b256 from the array at index iter
            let value_ptr_val_b256 = self.current_block.ins(context).get_ptr(
                value_ptr,
                Type::B256,
                array_index,
                span_md_idx,
            );

            match access_type {
                StateAccessType::Read => {
                    self.current_block.ins(context).state_load_quad_word(
                        value_ptr_val_b256,
                        *key_ptr_val,
                        span_md_idx,
                    );
                }
                StateAccessType::Write => {
                    // Finally, just call state_load_quad_word/state_store_quad_word
                    self.current_block.ins(context).state_store_quad_word(
                        value_ptr_val_b256,
                        *key_ptr_val,
                        span_md_idx,
                    );
                }
            }
        }

        Ok(match access_type {
            StateAccessType::Read => value_ptr_val,
            StateAccessType::Write => rhs.expect("expecting a rhs for write"),
        })
    }
}
