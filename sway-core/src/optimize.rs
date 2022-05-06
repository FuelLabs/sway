use fuel_crypto::Hasher;
use std::{collections::HashMap, sync::Arc};

use crate::{
    asm_generation::from_ir::ir_type_size_in_bytes,
    constants,
    error::CompileError,
    parse_tree::{AsmOp, AsmRegister, BuiltinProperty, LazyOp, Literal, Visibility},
    semantic_analysis::{ast_node::*, *},
    type_engine::*,
};

use sway_types::{ident::Ident, span::Span, state::StateIndex};

use sway_ir::*;

// -------------------------------------------------------------------------------------------------

pub(crate) fn compile_ast(ast: TypedParseTree) -> Result<Context, CompileError> {
    let mut ctx = Context::default();
    match ast {
        TypedParseTree::Script {
            namespace,
            main_function,
            declarations,
            all_nodes: _,
        } => compile_script(&mut ctx, main_function, namespace, declarations),
        TypedParseTree::Predicate {
            namespace: _,
            main_function: _,
            declarations: _,
            all_nodes: _,
        } => unimplemented!("compile predicate to ir"),
        TypedParseTree::Contract {
            abi_entries,
            namespace,
            declarations,
            all_nodes: _,
        } => compile_contract(&mut ctx, abi_entries, namespace, declarations),
        TypedParseTree::Library {
            namespace: _,
            all_nodes: _,
        } => unimplemented!("compile library to ir"),
    }?;
    ctx.verify()
        .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))
}

// -------------------------------------------------------------------------------------------------

fn compile_script(
    context: &mut Context,
    main_function: TypedFunctionDeclaration,
    namespace: NamespaceRef,
    declarations: Vec<TypedDeclaration>,
) -> Result<Module, CompileError> {
    let module = Module::new(context, Kind::Script);

    compile_constants(context, module, namespace, false)?;
    compile_declarations(context, module, declarations)?;
    compile_function(context, module, main_function)?;

    Ok(module)
}

fn compile_contract(
    context: &mut Context,
    abi_entries: Vec<TypedFunctionDeclaration>,
    namespace: NamespaceRef,
    declarations: Vec<TypedDeclaration>,
) -> Result<Module, CompileError> {
    let module = Module::new(context, Kind::Contract);

    compile_constants(context, module, namespace, false)?;
    compile_declarations(context, module, declarations)?;
    for decl in abi_entries {
        compile_abi_method(context, module, decl)?;
    }

    Ok(module)
}

// -------------------------------------------------------------------------------------------------

fn compile_constants(
    context: &mut Context,
    module: Module,
    namespace: NamespaceRef,
    public_only: bool,
) -> Result<(), CompileError> {
    read_module(
        |ns| -> Result<(), CompileError> {
            for decl in ns.get_all_declared_symbols() {
                let decl_name_value = match decl {
                    TypedDeclaration::ConstantDeclaration(TypedConstantDeclaration {
                        name,
                        value,
                        visibility,
                    }) => {
                        // XXX Do we really only add public constants?
                        if !public_only || matches!(visibility, Visibility::Public) {
                            Some((name, value))
                        } else {
                            None
                        }
                    }

                    TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                        name,
                        body,
                        const_decl_origin,
                        ..
                    }) if *const_decl_origin => Some((name, body)),

                    _otherwise => None,
                };

                if let Some((name, value)) = decl_name_value {
                    let const_val = compile_constant_expression(context, value)?;
                    module.add_global_constant(context, name.as_str().to_owned(), const_val);
                }
            }

            for ns_ix in ns.get_all_imported_modules().filter(|x| **x != namespace) {
                compile_constants(context, module, *ns_ix, true)?;
            }
            Ok(())
        },
        namespace,
    )?;

    Ok(())
}

fn compile_constant_expression(
    context: &mut Context,
    const_expr: &TypedExpression,
) -> Result<Value, CompileError> {
    if let TypedExpressionVariant::Literal(literal) = &const_expr.expression {
        let span_md_idx = MetadataIndex::from_span(context, &const_expr.span);
        Ok(convert_literal_to_value(context, literal, span_md_idx))
    } else {
        Err(CompileError::Internal(
            "Unsupported constant expression type.",
            const_expr.span.clone(),
        ))
    }
}

// -------------------------------------------------------------------------------------------------
// We don't really need to compile these declarations other than `const`s since:
// a) function decls are inlined into their call site and can be (re)created there, though ideally
//    we'd give them their proper name by compiling them here.
// b) struct decls are also inlined at their instantiation site.
// c) ditto for enums.
//
// And for structs and enums in particular, we must ignore those with embedded generic types as
// they are monomorphised only at the instantation site.  We must ignore the generic declarations
// altogether anyway.

fn compile_declarations(
    context: &mut Context,
    module: Module,
    declarations: Vec<TypedDeclaration>,
) -> Result<(), CompileError> {
    for declaration in declarations {
        match declaration {
            TypedDeclaration::ConstantDeclaration(decl) => {
                // These are in the global scope for the module, so they can be added there.
                let const_val = compile_constant_expression(context, &decl.value)?;
                module.add_global_constant(context, decl.name.as_str().to_owned(), const_val);
            }

            TypedDeclaration::FunctionDeclaration(_decl) => {
                // We no longer compile functions other than `main()` until we can improve the name
                // resolution.  Currently there isn't enough information in the AST to fully
                // distinguish similarly named functions and especially trait methods.
                //
                //compile_function(context, module, decl).map(|_| ())?
            }
            TypedDeclaration::ImplTrait {
                methods: _,
                type_implementing_for: _,
                ..
            } => {
                // And for the same reason we don't need to compile impls at all.
                //
                // compile_impl(
                //    context,
                //    module,
                //    type_implementing_for,
                //    methods,
                //)?,
            }

            TypedDeclaration::StructDeclaration(_)
            | TypedDeclaration::EnumDeclaration(_)
            | TypedDeclaration::TraitDeclaration(_)
            | TypedDeclaration::VariableDeclaration(_)
            | TypedDeclaration::Reassignment(_)
            | TypedDeclaration::StorageReassignment(_)
            | TypedDeclaration::AbiDeclaration(_)
            | TypedDeclaration::GenericTypeForFunctionScope { .. }
            | TypedDeclaration::StorageDeclaration(_)
            | TypedDeclaration::ErrorRecovery => (),
        }
    }
    Ok(())
}

// -------------------------------------------------------------------------------------------------

fn get_aggregate_for_types(
    context: &mut Context,
    type_ids: &[TypeId],
) -> Result<Aggregate, CompileError> {
    let types = type_ids
        .iter()
        .map(|ty_id| convert_resolved_typeid_no_span(context, ty_id))
        .collect::<Result<Vec<_>, CompileError>>()?;
    Ok(Aggregate::new_struct(context, types))
}

// -------------------------------------------------------------------------------------------------

fn create_enum_aggregate(
    context: &mut Context,
    variants: Vec<TypedEnumVariant>,
) -> Result<Aggregate, CompileError> {
    // Create the enum aggregate first.  NOTE: single variant enums don't need an aggregate but are
    // getting one here anyway.  They don't need to be a tagged union either.
    let field_types: Vec<_> = variants
        .into_iter()
        .map(|tev| convert_resolved_typeid_no_span(context, &tev.r#type))
        .collect::<Result<Vec<_>, CompileError>>()?;
    let enum_aggregate = Aggregate::new_struct(context, field_types);

    // Create the tagged union struct next.
    Ok(Aggregate::new_struct(
        context,
        vec![Type::Uint(64), Type::Union(enum_aggregate)],
    ))
}

// -------------------------------------------------------------------------------------------------

fn create_tuple_aggregate(
    context: &mut Context,
    fields: Vec<TypeId>,
) -> Result<Aggregate, CompileError> {
    let field_types = fields
        .into_iter()
        .map(|ty_id| convert_resolved_typeid_no_span(context, &ty_id))
        .collect::<Result<Vec<_>, CompileError>>()?;

    Ok(Aggregate::new_struct(context, field_types))
}

// -------------------------------------------------------------------------------------------------

fn compile_function(
    context: &mut Context,
    module: Module,
    ast_fn_decl: TypedFunctionDeclaration,
) -> Result<Option<Function>, CompileError> {
    // Currently monomorphisation of generics is inlined into main() and the functions with generic
    // args are still present in the AST declarations, but they can be ignored.
    if !ast_fn_decl.type_parameters.is_empty() {
        Ok(None)
    } else {
        let args = ast_fn_decl
            .parameters
            .iter()
            .map(|param| {
                convert_resolved_typeid(context, &param.r#type, &param.type_span)
                    .map(|ty| (param.name.as_str().into(), ty, param.name.span().clone()))
            })
            .collect::<Result<Vec<(String, Type, Span)>, CompileError>>()?;

        compile_fn_with_args(context, module, ast_fn_decl, args, None).map(&Some)
    }
}

// -------------------------------------------------------------------------------------------------

fn compile_fn_with_args(
    context: &mut Context,
    module: Module,
    ast_fn_decl: TypedFunctionDeclaration,
    args: Vec<(String, Type, Span)>,
    selector: Option<[u8; 4]>,
) -> Result<Function, CompileError> {
    let TypedFunctionDeclaration {
        name,
        body,
        return_type,
        return_type_span,
        visibility,
        ..
    } = ast_fn_decl;

    let args = args
        .into_iter()
        .map(|(name, ty, span)| (name, ty, MetadataIndex::from_span(context, &span)))
        .collect();
    let ret_type = convert_resolved_typeid(context, &return_type, &return_type_span)?;
    let func = Function::new(
        context,
        module,
        name.as_str().to_owned(),
        args,
        ret_type,
        selector,
        visibility == Visibility::Public,
    );

    // We clone the struct symbols here, as they contain the globals; any new local declarations
    // may remain within the function scope.
    let mut compiler = FnCompiler::new(context, module, func);

    let mut ret_val = compiler.compile_code_block(context, body)?;

    // Special case: if the return type is unit but the return value type is not, then we have an
    // implicit return from the last expression in the code block having a semi-colon.  This isn't
    // codified in the AST explicitly so we need to make a unit to return here.
    if ret_type.eq(context, &Type::Unit) && !matches!(ret_val.get_type(context), Some(Type::Unit)) {
        // NOTE: We're replacing the ret_val and throwing away whatever it used to be, as it is
        // never actually used anyway.
        ret_val = Constant::get_unit(context, None);
    }

    // Another special case: if the last expression in a function is a return then we don't want to
    // add another implicit return instruction here, as `ret_val` will be unit regardless of the
    // function return type actually is.  This new RET will be going into an unreachable block
    // which is valid, but pointless and we should avoid it due to the aforementioned potential
    // type conflict.
    //
    // To tell if this is the case we can check that the current block is empty and has no
    // predecessors (and isn't the entry block which has none by definition), implying the most
    // recent instruction was a RET.
    if compiler.current_block.num_instructions(context) > 0
        || compiler.current_block == compiler.function.get_entry_block(context)
        || compiler.current_block.num_predecessors(context) > 0
    {
        if ret_type.eq(context, &Type::Unit) {
            ret_val = Constant::get_unit(context, None);
        }
        compiler
            .current_block
            .ins(context)
            .ret(ret_val, ret_type, None);
    }
    Ok(func)
}

// -------------------------------------------------------------------------------------------------

/* Disabled until we can improve symbol resolution.  See comments above in compile_declarations().

fn compile_impl(
    context: &mut Context,
    module: Module,
    self_type: TypeInfo,
    ast_methods: Vec<TypedFunctionDeclaration>,
) -> Result<(), CompileError> {
    for method in ast_methods {
        let args = method
            .parameters
            .iter()
            .map(|param| {
                if param.name.as_str() == "self" {
                    convert_resolved_type(context, &self_type)
                } else {
                    convert_resolved_typeid(context, &param.r#type, &param.type_span)
                }
                .map(|ty| (param.name.as_str().into(), ty, param.name.span().clone()))
            })
            .collect::<Result<Vec<(String, Type, Span)>, CompileError>>()?;

        compile_fn_with_args(context, module, method, args, None)?;
    }
    Ok(())
}
*/

// -------------------------------------------------------------------------------------------------

fn compile_abi_method(
    context: &mut Context,
    module: Module,
    ast_fn_decl: TypedFunctionDeclaration,
) -> Result<Function, CompileError> {
    // Use the error from .to_fn_selector_value() if possible, else make an CompileError::Internal.
    let get_selector_result = ast_fn_decl.to_fn_selector_value();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let selector = match get_selector_result.ok(&mut warnings, &mut errors) {
        Some(selector) => selector,
        None => {
            return if !errors.is_empty() {
                Err(errors[0].clone())
            } else {
                Err(CompileError::InternalOwned(
                    format!(
                        "Cannot generate selector for ABI method: {}",
                        ast_fn_decl.name.as_str()
                    ),
                    ast_fn_decl.name.span().clone(),
                ))
            };
        }
    };

    let args = ast_fn_decl
        .parameters
        .iter()
        .map(|param| {
            convert_resolved_typeid(context, &param.r#type, &param.type_span)
                .map(|ty| (param.name.as_str().into(), ty, param.name.span().clone()))
        })
        .collect::<Result<Vec<(String, Type, Span)>, CompileError>>()?;

    compile_fn_with_args(context, module, ast_fn_decl, args, Some(selector))
}

// -------------------------------------------------------------------------------------------------

struct FnCompiler {
    module: Module,
    function: Function,
    current_block: Block,
    lexical_map: LexicalMap,
}

pub enum StateAccessType {
    Read,
    Write,
}

impl FnCompiler {
    fn new(context: &mut Context, module: Module, function: Function) -> Self {
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

    // ---------------------------------------------------------------------------------------------

    fn compile_code_block(
        &mut self,
        context: &mut Context,
        ast_block: TypedCodeBlock,
    ) -> Result<Value, CompileError> {
        ast_block
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
                        TypedDeclaration::ImplTrait { span, .. } => {
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
            .unwrap_or_else(|| Ok(Constant::get_unit(context, None)))
    }

    // ---------------------------------------------------------------------------------------------

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
                name,
                contract_call_params,
                arguments,
                function_body,
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
                        name.suffix.as_str(),
                        arguments,
                        Some(function_body),
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
            TypedExpressionVariant::IfLet {
                enum_type,
                expr,
                variant,
                variable_to_assign,
                then,
                r#else,
            } => self.compile_if_let(
                context,
                enum_type,
                expr,
                variant,
                variable_to_assign,
                then,
                r#else,
            ),
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
            TypedExpressionVariant::TypeProperty {
                property,
                type_id,
                span,
            } => {
                let ir_type = convert_resolved_typeid(context, &type_id, &span)?;
                match property {
                    BuiltinProperty::SizeOfType => Ok(Constant::get_uint(
                        context,
                        64,
                        ir_type_size_in_bytes(context, &ir_type),
                        None,
                    )),
                    BuiltinProperty::IsRefType => {
                        Ok(Constant::get_bool(context, !ir_type.is_copy_type(), None))
                    }
                }
            }
            TypedExpressionVariant::SizeOfValue { expr } => {
                // Compile the expression in case of side-effects but ignore its value.
                let ir_type = convert_resolved_typeid(context, &expr.return_type, &expr.span)?;
                self.compile_expression(context, *expr)?;
                Ok(Constant::get_uint(
                    context,
                    64,
                    ir_type_size_in_bytes(context, &ir_type),
                    None,
                ))
            }
            TypedExpressionVariant::AbiName(_) => {
                Ok(Value::new_constant(context, Constant::new_unit(), None))
            }
        }
    }

    // ---------------------------------------------------------------------------------------------

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
                // RET is a terminator so we must create a new block here.  If anything is added to
                // it then it'll almost certainly be dead code.
                self.current_block = self.function.create_block(context, None);
                Ok(Constant::get_unit(context, span_md_idx))
            }
        }
    }

    // ---------------------------------------------------------------------------------------------

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

    // ---------------------------------------------------------------------------------------------

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

    // ---------------------------------------------------------------------------------------------

    fn compile_fn_call(
        &mut self,
        context: &mut Context,
        _ast_name: &str,
        ast_args: Vec<(Ident, TypedExpression)>,
        callee_body: Option<TypedCodeBlock>,
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

            let parameters = ast_args
                .iter()
                .map(|(name, expr)| TypedFunctionParameter {
                    name: name.clone(),
                    r#type: expr.return_type,
                    type_span: crate::span::Span::new(" ".into(), 0, 0, None).unwrap(),
                })
                .collect();

            let callee_body = callee_body.unwrap();

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
                span: crate::span::Span::new(" ".into(), 0, 0, None).unwrap(),
                return_type,
                type_parameters: Vec::new(),
                return_type_span: crate::span::Span::new(" ".into(), 0, 0, None).unwrap(),
                visibility: Visibility::Private,
                is_contract_call: false,
                purity: Default::default(),
            };

            let callee = compile_function(context, self.module, callee_fn_decl)?;

            // Now actually call the new function.
            let args = ast_args
                .into_iter()
                .map(|(_, expr)| self.compile_expression(context, expr))
                .collect::<Result<Vec<Value>, CompileError>>()?;
            Ok(self
                .current_block
                .ins(context)
                .call(callee.unwrap(), &args, span_md_idx))
        }
    }

    fn get_codeblock_return_type(codeblock: &TypedCodeBlock) -> Option<TypeId> {
        if codeblock.contents.is_empty() {
            Some(insert_type(TypeInfo::Tuple(Vec::new())))
        } else {
            codeblock
                .contents
                .iter()
                .find_map(|node| match &node.content {
                    TypedAstNodeContent::ReturnStatement(trs) => Some(trs.expr.return_type),
                    TypedAstNodeContent::ImplicitReturnExpression(te) => Some(te.return_type),
                    _otherwise => None,
                })
        }
    }

    // ---------------------------------------------------------------------------------------------

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

        let false_block_begin = self.function.create_block(context, None);
        self.current_block = false_block_begin;
        let false_value = match ast_else {
            None => Constant::get_unit(context, None),
            Some(expr) => self.compile_expression(context, *expr)?,
        };
        let false_block_end = self.current_block;

        entry_block.ins(context).conditional_branch(
            cond_value,
            true_block_begin,
            false_block_begin,
            None,
            cond_span_md_idx,
        );

        let merge_block = self.function.create_block(context, None);
        true_block_end
            .ins(context)
            .branch(merge_block, Some(true_value), None);
        false_block_end
            .ins(context)
            .branch(merge_block, Some(false_value), None);

        self.current_block = merge_block;
        Ok(merge_block.get_phi(context))
    }

    // ---------------------------------------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    fn compile_if_let(
        &mut self,
        context: &mut Context,
        enum_type: TypeId,
        ast_expr: Box<TypedExpression>,
        variant: TypedEnumVariant,
        variable_to_assign: Ident,
        ast_then: TypedCodeBlock,
        ast_else: Option<Box<TypedExpression>>,
    ) -> Result<Value, CompileError> {
        // Similar to a regular `if` expression we create the different blocks in order, being
        // careful to track the 'current' block as it evolves.
        //
        // Instead of a condition we have to match the expression result with the enum variant (by
        // comparing the tags), and then assign the variant to a local variable, which is scoped to
        // the `then' block.
        let cond_span_md_idx = MetadataIndex::from_span(context, &ast_expr.span);
        let enum_aggregate = if let Type::Struct(aggregate) =
            convert_resolved_typeid(context, &enum_type, &ast_expr.span)?
        {
            aggregate
        } else {
            return Err(CompileError::Internal(
                "Enum type for `if let` is not an enum.",
                ast_expr.span,
            ));
        };
        let matched_value = self.compile_expression(context, *ast_expr)?;
        let matched_tag_value = self.current_block.ins(context).extract_value(
            matched_value,
            enum_aggregate,
            vec![0],
            cond_span_md_idx,
        );
        let variant_tag = variant.tag as u64;
        let variant_tag_value = Constant::get_uint(context, 64, variant_tag, cond_span_md_idx);
        let cond_value = self.current_block.ins(context).cmp(
            Predicate::Equal,
            matched_tag_value,
            variant_tag_value,
            cond_span_md_idx,
        );
        let entry_block = self.current_block;

        // The true/then block, with a variable referring to the matched value.
        let true_block_begin = self.function.create_block(context, None);
        self.current_block = true_block_begin;

        // See compile_var_decl() for details.  Copied from there, essentially.  We're still making
        // a copy of the value into a local variable, which is probably wasteful.  But an
        // optimisation pass is probably the best way to fix that.
        let var_span_md_idx = MetadataIndex::from_span(context, variable_to_assign.span());
        let variable_type = enum_aggregate
            .get_field_type(context, &[1, variant_tag])
            .ok_or_else(|| {
                CompileError::Internal(
                    "Unable to get type of enum variant from its tag.",
                    variable_to_assign.span().clone(),
                )
            })?;
        let var_init_value = self.current_block.ins(context).extract_value(
            matched_value,
            enum_aggregate,
            vec![1, variant_tag],
            var_span_md_idx,
        );
        let local_name = self
            .lexical_map
            .enter_scope()
            .insert(variable_to_assign.as_str().to_owned());
        let variable_ptr = self
            .function
            .new_local_ptr(context, local_name, variable_type, false, None)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;
        let variable_ptr_ty = *variable_ptr.get_type(context);
        let variable_ptr_val = self.current_block.ins(context).get_ptr(
            variable_ptr,
            variable_ptr_ty,
            0,
            var_span_md_idx,
        );
        self.current_block
            .ins(context)
            .store(variable_ptr_val, var_init_value, var_span_md_idx);
        let true_value = self.compile_code_block(context, ast_then)?;
        let true_block_end = self.current_block;
        self.lexical_map.leave_scope();

        // The optional false/else block.  Does not have access to the variable.
        let false_block_begin = self.function.create_block(context, None);
        self.current_block = false_block_begin;
        let false_value = match ast_else {
            None => Constant::get_unit(context, None),
            Some(expr) => self.compile_expression(context, *expr)?,
        };
        let false_block_end = self.current_block;

        // Branch from the top to each of the true/false blocks and then merge from them to a final
        // block.
        entry_block.ins(context).conditional_branch(
            cond_value,
            true_block_begin,
            false_block_begin,
            None,
            cond_span_md_idx,
        );

        let merge_block = self.function.create_block(context, None);
        true_block_end
            .ins(context)
            .branch(merge_block, Some(true_value), None);
        false_block_end
            .ins(context)
            .branch(merge_block, Some(false_value), None);

        self.current_block = merge_block;
        Ok(merge_block.get_phi(context))
    }

    // ---------------------------------------------------------------------------------------------

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

    // ---------------------------------------------------------------------------------------------

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

    // ---------------------------------------------------------------------------------------------

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

    // ---------------------------------------------------------------------------------------------

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
                name.span().clone(),
            ))
        }
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_reassignment(
        &mut self,
        context: &mut Context,
        ast_reassignment: TypedReassignment,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, CompileError> {
        let name = self
            .lexical_map
            .get(ast_reassignment.lhs[0].name.as_str())
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
                            ast_reassignment.lhs[0].name.span().clone(),
                        )
                    })?
                    .1
            }
        };

        let reassign_val = self.compile_expression(context, ast_reassignment.rhs)?;

        assert!(!ast_reassignment.lhs.is_empty());
        if ast_reassignment.lhs.len() == 1 {
            // A non-aggregate; use a `store`.
            self.current_block
                .ins(context)
                .store(val, reassign_val, span_md_idx);
        } else {
            // An aggregate.  Iterate over the field names from the left hand side and collect
            // field indices.  The struct type from the previous iteration is used to determine the
            // field type for the current iteration.
            let field_idcs = get_indices_for_struct_access(&ast_reassignment.lhs)?;

            let ty = match val.get_type(context).unwrap() {
                Type::Struct(aggregate) => aggregate,
                _otherwise => {
                    let spans = ast_reassignment
                        .lhs
                        .iter()
                        .map(|lhs| lhs.name.span().clone())
                        .reduce(Span::join)
                        .expect("Joined spans of LHS of reassignment.");
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

    // ---------------------------------------------------------------------------------------------

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
            &fields.last().expect("guaranteed by grammar").r#type,
        )?;

        // Get the list of indices used to access the storage field. This will be empty
        // if the storage field type is not a struct.
        let field_idcs = get_indices_for_struct_access(fields)?;

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

    // ---------------------------------------------------------------------------------------------

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

    // ---------------------------------------------------------------------------------------------

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
            ValueDatum::Argument(Type::Array(aggregate)) => Ok(*aggregate),
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

    // ---------------------------------------------------------------------------------------------

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

    // ---------------------------------------------------------------------------------------------

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
            ValueDatum::Argument(Type::Struct(aggregate)) => Ok(*aggregate),
            otherwise => Err(CompileError::InternalOwned(
                format!("Unsupported struct value for field expression: {otherwise:?}",),
                ast_struct_expr_span,
            )),
        }?;

        let field_idx = match get_struct_name_and_field_index(struct_type_id, &ast_field.name) {
            None => Err(CompileError::Internal(
                "Unknown struct in field expression.",
                ast_field.span,
            )),
            Some((struct_name, field_idx)) => match field_idx {
                None => Err(CompileError::InternalOwned(
                    format!(
                        "Unknown field name '{}' for struct '{struct_name}' in field expression.",
                        ast_field.name
                    ),
                    ast_field.span,
                )),
                Some(field_idx) => Ok(field_idx),
            },
        }?;

        Ok(self.current_block.ins(context).extract_value(
            struct_val,
            aggregate,
            vec![field_idx],
            span_md_idx,
        ))
    }

    // ---------------------------------------------------------------------------------------------

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

        Ok(match contents {
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
        })
    }

    // ---------------------------------------------------------------------------------------------

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

    // ---------------------------------------------------------------------------------------------

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

    // ---------------------------------------------------------------------------------------------

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
            &fields.last().expect("guaranteed by grammar").r#type,
        )?;

        // Get the list of indices used to access the storage field. This will be empty
        // if the storage field type is not a struct.
        let field_idcs = get_indices_for_struct_access(fields)?;

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

    // ---------------------------------------------------------------------------------------------

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

    // -------------------------------------------------------------------------------------------------
    // Utils

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
        Ok(match r#type {
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
                struct_val
            }
            Type::Bool | Type::Uint(_) | Type::B256 => {
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
                let key_ptr_val =
                    self.current_block
                        .ins(context)
                        .get_ptr(key_ptr, key_ptr_ty, 0, span_md_idx);

                // Store the const hash value to the key pointer value
                self.current_block
                    .ins(context)
                    .store(key_ptr_val, const_key, span_md_idx);

                match r#type {
                    Type::Uint(_) | Type::Bool => {
                        // These types fit in a word. use state_store_word/state_load_word
                        match access_type {
                            StateAccessType::Read => {
                                // `state_load_word` always returns a `u64`. Cast the result back
                                // to the right type before returning
                                let load_val = self
                                    .current_block
                                    .ins(context)
                                    .state_load_word(key_ptr_val, span_md_idx);
                                self.current_block.ins(context).bitcast(
                                    load_val,
                                    *r#type,
                                    span_md_idx,
                                )
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
                                    key_ptr_val,
                                    span_md_idx,
                                );
                                rhs.expect("expecting a rhs for write")
                            }
                        }
                    }
                    Type::B256 => {
                        // B256 requires 4 words. Use state_load_quad_word/state_store_quad_word
                        // First, create a name for the value to load from or store to
                        let mut value_name = format!("{}{}", "val_for_", ix.to_usize());
                        for ix in &indices {
                            value_name = format!("{}_{}", value_name, ix);
                        }
                        let alias_value_name =
                            self.lexical_map.insert(value_name.as_str().to_owned());

                        // Local pointer to hold the B256
                        let value_ptr = self
                            .function
                            .new_local_ptr(context, alias_value_name, *r#type, true, None)
                            .map_err(|ir_error| {
                                CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                            })?;

                        // Convert the local pointer created to a value using get_ptr
                        let value_ptr_val = self.current_block.ins(context).get_ptr(
                            value_ptr,
                            *r#type,
                            0,
                            span_md_idx,
                        );

                        match access_type {
                            StateAccessType::Read => {
                                self.current_block.ins(context).state_load_quad_word(
                                    value_ptr_val,
                                    key_ptr_val,
                                    span_md_idx,
                                );
                                value_ptr_val
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
                                    key_ptr_val,
                                    span_md_idx,
                                );
                                rhs.expect("expecting a rhs for write")
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
            _ => unimplemented!("Other types are not yet supported in storage"),
        })
    }
}

// -------------------------------------------------------------------------------------------------
// Nested mappings between symbol strings.  Allows shadowing and/or nested scopes for local
// symbols.
//
// NOTE: ALL symbols should be represented in this data structure to be sure that we
// don't accidentally ignore (i.e., neglect to shadow with) a new binding.
//
// A further complication is although we have enter_scope() and leave_scope() to potentially add
// and remove shadowing symbols, the re-use of symbol names can't be allowed, so all names are
// reserved even when they're not 'currently' valid.

struct LexicalMap {
    symbol_map: Vec<HashMap<String, String>>,
    reserved_sybols: Vec<String>,
}

impl LexicalMap {
    fn from_iter<I: IntoIterator<Item = String>>(names: I) -> Self {
        let (root_symbol_map, reserved_sybols): (HashMap<String, String>, Vec<String>) = names
            .into_iter()
            .fold((HashMap::new(), Vec::new()), |(mut m, mut r), name| {
                m.insert(name.clone(), name.clone());
                r.push(name);
                (m, r)
            });

        LexicalMap {
            symbol_map: vec![root_symbol_map],
            reserved_sybols,
        }
    }

    fn enter_scope(&mut self) -> &mut Self {
        self.symbol_map.push(HashMap::new());
        self
    }

    fn leave_scope(&mut self) -> &mut Self {
        assert!(self.symbol_map.len() > 1);
        self.symbol_map.pop();
        self
    }

    fn get(&self, symbol: &str) -> Option<&String> {
        // Only get 'valid' symbols which are currently in scope.
        self.symbol_map
            .iter()
            .rev()
            .find_map(|scope| scope.get(symbol))
    }

    fn insert(&mut self, new_symbol: String) -> String {
        // Insert this new symbol into this lexical scope.  If it has ever existed then the
        // original will be shadowed and the shadower is returned.
        fn get_new_local_symbol(reserved: &[String], candidate: String) -> String {
            match reserved.iter().find(|&reserved| reserved == &candidate) {
                None => candidate,
                Some(_) => {
                    // Try again with adjusted candidate.
                    get_new_local_symbol(reserved, format!("{candidate}_"))
                }
            }
        }
        let local_symbol = get_new_local_symbol(&self.reserved_sybols, new_symbol.clone());
        self.symbol_map
            .last_mut()
            .expect("LexicalMap should always have at least the root scope.")
            .insert(new_symbol, local_symbol.clone());
        self.reserved_sybols.push(local_symbol.clone());
        local_symbol
    }
}

// -------------------------------------------------------------------------------------------------
// Get the name of a struct and the index to a particular named field from a TypeId.

fn get_struct_name_and_field_index(
    field_type: TypeId,
    field_name: &Ident,
) -> Option<(String, Option<u64>)> {
    resolve_type(field_type, field_name.span())
        .ok()
        .and_then(|ty_info| match ty_info {
            TypeInfo::Struct { name, fields } => Some((
                name.as_str().to_owned(),
                fields
                    .iter()
                    .enumerate()
                    .find(|(_, field)| &field.name == field_name)
                    .map(|(idx, _)| idx as u64),
            )),
            _otherwise => None,
        })
}

// To gather the indices into nested structs for the struct oriented IR instructions we need to
// inspect the names and types of a vector of fields in a path.  There are several different
// representations of this in the AST but we can wrap fetching the struct type and field name in a
// trait.  And we can even wrap the implementation in a macro.

trait TypedNamedField {
    fn get_type(&self) -> TypeId;
    fn get_name(&self) -> &Ident;
}

macro_rules! impl_typed_named_field_for {
    ($field_type_name: ident) => {
        impl TypedNamedField for $field_type_name {
            fn get_type(&self) -> TypeId {
                self.r#type
            }
            fn get_name(&self) -> &Ident {
                &self.name
            }
        }
    };
}

impl_typed_named_field_for!(ReassignmentLhs);
impl_typed_named_field_for!(TypeCheckedStorageAccessDescriptor);
impl_typed_named_field_for!(TypeCheckedStorageReassignDescriptor);

fn get_indices_for_struct_access<F: TypedNamedField>(
    fields: &[F],
) -> Result<Vec<u64>, CompileError> {
    fields[1..]
        .iter()
        .fold(Ok((Vec::new(), fields[0].get_type())), |acc, field| {
            // Make sure we have an aggregate to index into.
            acc.and_then(|(mut fld_idcs, prev_type_id)| {
                // Get the field index and also its type for the next iteration.
                match get_struct_name_and_field_index(prev_type_id, field.get_name()) {
                    None => Err(CompileError::Internal(
                        "Unknown struct in in reassignment.",
                        Span::dummy(),
                    )),
                    Some((struct_name, field_idx)) => match field_idx {
                        None => Err(CompileError::InternalOwned(
                            format!(
                                "Unknown field name '{}' for struct {struct_name} in reassignment.",
                                field.get_name(),
                            ),
                            field.get_name().span().clone(),
                        )),
                        Some(field_idx) => {
                            // Save the field index.
                            fld_idcs.push(field_idx);
                            Ok((fld_idcs, field.get_type()))
                        }
                    },
                }
            })
        })
        .map(|(fld_idcs, _)| fld_idcs)
}

// -------------------------------------------------------------------------------------------------

fn convert_literal_to_value(
    context: &mut Context,
    ast_literal: &Literal,
    span_id_idx: Option<MetadataIndex>,
) -> Value {
    match ast_literal {
        // In Sway for now we don't have `as` casting and for integers which may be implicitly cast
        // between widths we just emit a warning, and essentially ignore it.  We also assume a
        // 'Numeric' integer of undetermined width is 'u64`.  The IR would like to be type
        // consistent and doesn't tolerate mising integers of different width, so for now, until we
        // do introduce explicit `as` casting, all integers are `u64` as far as the IR is
        // concerned.
        Literal::U8(n) | Literal::Byte(n) => {
            Constant::get_uint(context, 64, *n as u64, span_id_idx)
        }
        Literal::U16(n) => Constant::get_uint(context, 64, *n as u64, span_id_idx),
        Literal::U32(n) => Constant::get_uint(context, 64, *n as u64, span_id_idx),
        Literal::U64(n) => Constant::get_uint(context, 64, *n, span_id_idx),
        Literal::Numeric(n) => Constant::get_uint(context, 64, *n, span_id_idx),
        Literal::String(s) => {
            Constant::get_string(context, s.as_str().as_bytes().to_vec(), span_id_idx)
        }
        Literal::Boolean(b) => Constant::get_bool(context, *b, span_id_idx),
        Literal::B256(bs) => Constant::get_b256(context, *bs, span_id_idx),
    }
}

fn convert_literal_to_constant(ast_literal: &Literal) -> Constant {
    match ast_literal {
        // All integers are `u64`.  See comment above.
        Literal::U8(n) | Literal::Byte(n) => Constant::new_uint(64, *n as u64),
        Literal::U16(n) => Constant::new_uint(64, *n as u64),
        Literal::U32(n) => Constant::new_uint(64, *n as u64),
        Literal::U64(n) => Constant::new_uint(64, *n),
        Literal::Numeric(n) => Constant::new_uint(64, *n),
        Literal::String(s) => Constant::new_string(s.as_str().as_bytes().to_vec()),
        Literal::Boolean(b) => Constant::new_bool(*b),
        Literal::B256(bs) => Constant::new_b256(*bs),
    }
}

fn convert_resolved_typeid(
    context: &mut Context,
    ast_type: &TypeId,
    span: &Span,
) -> Result<Type, CompileError> {
    // There's probably a better way to convert TypeError to String, but... we'll use something
    // other than String eventually?  IrError?
    convert_resolved_type(
        context,
        &resolve_type(*ast_type, span)
            .map_err(|ty_err| CompileError::InternalOwned(format!("{ty_err:?}"), span.clone()))?,
        span,
    )
}

fn convert_resolved_typeid_no_span(
    context: &mut Context,
    ast_type: &TypeId,
) -> Result<Type, CompileError> {
    let msg = "unknown source location";
    let span = crate::span::Span::new(Arc::from(msg), 0, msg.len(), None).unwrap();
    convert_resolved_typeid(context, ast_type, &span)
}

fn convert_resolved_type(
    context: &mut Context,
    ast_type: &TypeInfo,
    span: &Span,
) -> Result<Type, CompileError> {
    Ok(match ast_type {
        // All integers are `u64`, see comment in convert_literal_to_value() above.
        TypeInfo::UnsignedInteger(_) => Type::Uint(64),
        TypeInfo::Numeric => Type::Uint(64),
        TypeInfo::Boolean => Type::Bool,
        TypeInfo::Byte => Type::Uint(64),
        TypeInfo::B256 => Type::B256,
        TypeInfo::Str(n) => Type::String(*n),
        TypeInfo::Struct { fields, .. } => get_aggregate_for_types(
            context,
            fields
                .iter()
                .map(|field| field.r#type)
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .map(&Type::Struct)?,
        TypeInfo::Enum { variant_types, .. } => {
            create_enum_aggregate(context, variant_types.clone()).map(&Type::Struct)?
        }
        TypeInfo::Array(elem_type_id, count) => {
            let elem_type = convert_resolved_typeid(context, elem_type_id, span)?;
            Type::Array(Aggregate::new_array(context, elem_type, *count as u64))
        }
        TypeInfo::Tuple(fields) => {
            if fields.is_empty() {
                // XXX We've removed Unit from the core compiler, replaced with an empty Tuple.
                // Perhaps the same should be done for the IR, although it would use an empty
                // aggregate which might not make as much sense as a dedicated Unit type.
                Type::Unit
            } else {
                let new_fields = fields.iter().map(|x| x.type_id).collect();
                create_tuple_aggregate(context, new_fields).map(Type::Struct)?
            }
        }

        // Unsupported types which shouldn't exist in the AST after type checking and
        // monomorphisation.
        TypeInfo::Custom { .. } => {
            return Err(CompileError::Internal(
                "Custom type cannot be resolved in IR.",
                span.clone(),
            ))
        }
        TypeInfo::SelfType { .. } => {
            return Err(CompileError::Internal(
                "Self type cannot be resolved in IR.",
                span.clone(),
            ))
        }
        TypeInfo::Contract => {
            return Err(CompileError::Internal(
                "Contract type cannot be resolved in IR.",
                span.clone(),
            ))
        }
        TypeInfo::ContractCaller { .. } => {
            return Err(CompileError::Internal(
                "ContractCaller type cannot be reoslved in IR.",
                span.clone(),
            ))
        }
        TypeInfo::Unknown => {
            return Err(CompileError::Internal(
                "Unknown type cannot be resolved in IR.",
                span.clone(),
            ))
        }
        TypeInfo::UnknownGeneric { .. } => {
            return Err(CompileError::Internal(
                "Generic type cannot be resolved in IR.",
                span.clone(),
            ))
        }
        TypeInfo::Ref(_) => {
            return Err(CompileError::Internal(
                "Ref type cannot be resolved in IR.",
                span.clone(),
            ))
        }
        TypeInfo::ErrorRecovery => {
            return Err(CompileError::Internal(
                "Error recovery type cannot be resolved in IR.",
                span.clone(),
            ))
        }
        TypeInfo::Storage { .. } => {
            return Err(CompileError::Internal(
                "Storage type cannot be resolved in IR.",
                span.clone(),
            ))
        }
    })
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use crate::{
        control_flow_analysis::{ControlFlowGraph, Graph},
        parser::{Rule, SwayParser},
        semantic_analysis::{TreeType, TypedParseTree},
    };
    use pest::Parser;

    // -------------------------------------------------------------------------------------------------

    #[test]
    fn sway_to_ir_tests() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let dir: PathBuf = format!("{}/tests/sway_to_ir", manifest_dir).into();
        for entry in std::fs::read_dir(dir).unwrap() {
            // We're only interested in the `.sw` files here.
            let path = entry.unwrap().path();
            match path.extension().unwrap().to_str() {
                Some("sw") => {
                    //
                    // Run the tests!
                    //
                    println!("---- Sway To IR: {:?} ----", path);
                    test_sway_to_ir(path);
                }
                Some("ir") | Some("disabled") => (),
                _ => panic!(
                    "File with invalid extension in tests dir: {:?}",
                    path.file_name().unwrap_or(path.as_os_str())
                ),
            }
        }
    }

    fn test_sway_to_ir(sw_path: PathBuf) {
        let input_bytes = std::fs::read(&sw_path).unwrap();
        let input = String::from_utf8_lossy(&input_bytes);

        let mut ir_path = sw_path.clone();
        ir_path.set_extension("ir");

        let expected_bytes = std::fs::read(&ir_path).unwrap();
        let expected = String::from_utf8_lossy(&expected_bytes);

        let typed_ast = parse_to_typed_ast(sw_path.clone(), &input);
        let ir = super::compile_ast(typed_ast).unwrap();
        let output = sway_ir::printer::to_string(&ir);

        // Use a tricky regex to replace the local path in the metadata with something generic.  It
        // should convert, e.g.,
        //     `!0 = filepath "/usr/home/me/sway/sway-core/tests/sway_to_ir/foo.sw"`
        //  to `!0 = filepath "/path/to/foo.sw"`
        let path_converter = regex::Regex::new(r#"(!\d = filepath ")(?:[^/]*/)*(.+)"#).unwrap();
        let output = path_converter.replace_all(output.as_str(), "$1/path/to/$2");

        if output != expected {
            println!("{}", prettydiff::diff_lines(&expected, &output));
            panic!("{} failed.", sw_path.display());
        }
    }

    // -------------------------------------------------------------------------------------------------

    #[test]
    fn ir_printer_parser_tests() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let dir: PathBuf = format!("{}/tests/sway_to_ir", manifest_dir).into();
        for entry in std::fs::read_dir(dir).unwrap() {
            // We're only interested in the `.ir` files here.
            let path = entry.unwrap().path();
            match path.extension().unwrap().to_str() {
                Some("ir") => {
                    //
                    // Run the tests!
                    //
                    println!("---- IR Print and Parse Test: {:?} ----", path);
                    test_printer_parser(path);
                }
                Some("sw") | Some("disabled") => (),
                _ => panic!(
                    "File with invalid extension in tests dir: {:?}",
                    path.file_name().unwrap_or(path.as_os_str())
                ),
            }
        }
    }

    fn test_printer_parser(path: PathBuf) {
        let input_bytes = std::fs::read(&path).unwrap();
        let input = String::from_utf8_lossy(&input_bytes);

        // Use another tricky regex to inject the proper metadata filepath back, so we can create
        // spans in the parser.  NOTE, if/when we refactor spans to not have the source string and
        // just the path these tests should pass without needing this conversion.
        let mut true_path = path.clone();
        true_path.set_extension("sw");
        let path_converter = regex::Regex::new(r#"(!\d = filepath )(?:.+)"#).unwrap();
        let input = path_converter.replace_all(&input, format!("$1\"{}\"", true_path.display()));

        let parsed_ctx = match sway_ir::parser::parse(&input) {
            Ok(p) => p,
            Err(e) => {
                println!("{}: {}", path.display(), e);
                panic!();
            }
        };
        let printed = sway_ir::printer::to_string(&parsed_ctx);
        if printed != input {
            println!("{}", prettydiff::diff_lines(&input, &printed));
            panic!("{} failed.", path.display());
        }
    }

    // -------------------------------------------------------------------------------------------------

    fn parse_to_typed_ast(path: PathBuf, input: &str) -> TypedParseTree {
        let mut parsed =
            SwayParser::parse(Rule::program, std::sync::Arc::from(input)).expect("parse_tree");

        let program_type = match parsed
            .peek()
            .unwrap()
            .into_inner()
            .peek()
            .unwrap()
            .as_rule()
        {
            Rule::script => TreeType::Script,
            Rule::contract => TreeType::Contract,
            Rule::predicate => TreeType::Predicate,
            Rule::library => todo!(),
            _ => unreachable!("unexpected program type"),
        };

        let dir_of_code = std::sync::Arc::new(path.parent().unwrap().into());
        let file_name = std::sync::Arc::new(path);

        let build_config = crate::build_config::BuildConfig {
            file_name,
            dir_of_code,
            manifest_path: std::sync::Arc::new(".".into()),
            use_orig_asm: false,
            use_orig_parser: false,
            print_intermediate_asm: false,
            print_finalized_asm: false,
            print_ir: false,
            generated_names: Default::default(),
        };

        let mut warnings = vec![];
        let mut errors = vec![];
        let parse_tree =
            crate::parse_root_from_pairs(parsed.next().unwrap().into_inner(), Some(&build_config))
                .unwrap(&mut warnings, &mut errors);

        let mut dead_code_graph = ControlFlowGraph {
            graph: Graph::new(),
            entry_points: vec![],
            namespace: Default::default(),
        };
        TypedParseTree::type_check(
            parse_tree.tree,
            crate::create_module(),
            crate::create_module(),
            &program_type,
            &build_config,
            &mut dead_code_graph,
        )
        .unwrap(&mut warnings, &mut errors)
    }
}

// -------------------------------------------------------------------------------------------------
