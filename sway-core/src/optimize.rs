use std::collections::HashMap;
use std::iter::FromIterator;

use crate::{
    parse_tree::{AsmOp, AsmRegister, LazyOp, Literal, Visibility},
    semantic_analysis::{ast_node::TypedCodeBlock, ast_node::*, *},
    type_engine::*,
};

use sway_types::{ident::Ident, span::Span};

use sway_ir::*;

// -------------------------------------------------------------------------------------------------
// XXX This needs to return a CompileResult.

pub(crate) fn compile_ast(ast: TypedParseTree) -> Result<Context, String> {
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
    ctx.verify()?;
    Ok(ctx)
}

// -------------------------------------------------------------------------------------------------

fn compile_script(
    context: &mut Context,
    main_function: TypedFunctionDeclaration,
    namespace: NamespaceRef,
    declarations: Vec<TypedDeclaration>,
) -> Result<Module, String> {
    let module = Module::new(context, Kind::Script, "script");

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
) -> Result<Module, String> {
    let module = Module::new(context, Kind::Contract, "contract");

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
) -> Result<(), String> {
    read_module(
        |ns| -> Result<(), String> {
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
) -> Result<Value, String> {
    if let TypedExpressionVariant::Literal(literal) = &const_expr.expression {
        let span_md_idx = MetadataIndex::from_span(context, &const_expr.span);
        Ok(convert_literal_to_value(context, literal, span_md_idx))
    } else {
        Err("Unsupported constant expression type.".into())
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
) -> Result<(), String> {
    for declaration in declarations {
        match declaration {
            TypedDeclaration::ConstantDeclaration(decl) => {
                // These are in the global scope for the module, so they can be added there.
                let const_val = compile_constant_expression(context, &decl.value)?;
                module.add_global_constant(context, decl.name.as_str().to_owned(), const_val);
            }

            TypedDeclaration::FunctionDeclaration(decl) => compile_function(context, module, decl)?,
            TypedDeclaration::ImplTrait {
                methods,
                type_implementing_for,
                ..
            } => compile_impl(context, module, type_implementing_for, methods)?,

            TypedDeclaration::StructDeclaration(_)
            | TypedDeclaration::TraitDeclaration(_)
            | TypedDeclaration::EnumDeclaration(_)
            | TypedDeclaration::VariableDeclaration(_)
            | TypedDeclaration::Reassignment(_)
            | TypedDeclaration::AbiDeclaration(_)
            | TypedDeclaration::GenericTypeForFunctionScope { .. }
            | TypedDeclaration::ErrorRecovery => (),
        }
    }
    Ok(())
}

// -------------------------------------------------------------------------------------------------

fn create_struct_aggregate(
    context: &mut Context,
    name: String,
    fields: Vec<OwnedTypedStructField>,
) -> Result<Aggregate, String> {
    let (field_types, syms): (Vec<_>, Vec<_>) = fields
        .into_iter()
        .map(|tsf| {
            (
                convert_resolved_typeid_no_span(context, &tsf.r#type),
                tsf.name,
            )
        })
        .unzip();

    let field_types = field_types
        .into_iter()
        .collect::<Result<Vec<_>, String>>()?;

    let aggregate = Aggregate::new_struct(context, Some(name), field_types);
    context.add_aggregate_symbols(
        aggregate,
        HashMap::from_iter(syms.into_iter().enumerate().map(|(n, sym)| (sym, n as u64))),
    )?;

    Ok(aggregate)
}

// -------------------------------------------------------------------------------------------------

fn compile_enum_decl(
    context: &mut Context,
    enum_decl: TypedEnumDeclaration,
) -> Result<Aggregate, String> {
    let TypedEnumDeclaration {
        name,
        type_parameters,
        variants,
        .. //span,
    } = enum_decl;

    if !type_parameters.is_empty() {
        return Err("Unable to compile generic enums.".into());
    }

    create_enum_aggregate(
        context,
        name.as_str().to_owned(),
        variants
            .into_iter()
            .map(|tev| tev.as_owned_typed_enum_variant())
            .collect(),
    )
}

fn create_enum_aggregate(
    context: &mut Context,
    name: String,
    variants: Vec<OwnedTypedEnumVariant>,
) -> Result<Aggregate, String> {
    // Create the enum aggregate first.
    let (field_types, syms): (Vec<_>, Vec<_>) = variants
        .into_iter()
        .map(|tev| {
            (
                convert_resolved_typeid_no_span(context, &tev.r#type),
                tev.name,
            )
        })
        .unzip();

    let field_types = field_types
        .into_iter()
        .collect::<Result<Vec<_>, String>>()?;

    let enum_aggregate = Aggregate::new_struct(context, Some(name.clone() + "_union"), field_types);
    // Not sure if we should do this..?  The 'field' names aren't used for enums?
    context.add_aggregate_symbols(
        enum_aggregate,
        HashMap::from_iter(syms.into_iter().enumerate().map(|(n, sym)| (sym, n as u64))),
    )?;

    // Create the tagged union struct next.  Just by creating it here with the name it'll be added
    // to the context and can be looked up.  It isn't obvious from the name, maybe it should
    // change... to create()?  insert()?  Anonymous aggregates aren't added though, so... maybe it
    // should be separate calls to create it and then insert it by name.
    Ok(Aggregate::new_struct(
        context,
        Some(name),
        vec![Type::Uint(64), Type::Union(enum_aggregate)],
    ))
}

// -------------------------------------------------------------------------------------------------

fn create_tuple_aggregate(context: &mut Context, fields: Vec<TypeId>) -> Result<Aggregate, String> {
    let field_types = fields
        .into_iter()
        .map(|ty_id| convert_resolved_typeid_no_span(context, &ty_id))
        .collect::<Result<Vec<_>, String>>()?;

    Ok(Aggregate::new_struct(context, None, field_types))
}

// -------------------------------------------------------------------------------------------------

fn compile_function(
    context: &mut Context,
    module: Module,
    ast_fn_decl: TypedFunctionDeclaration,
) -> Result<(), String> {
    // Currently monomorphisation of generics is inlined into main() and the functions with generic
    // args are still present in the AST declarations, but they can be ignored.
    if !ast_fn_decl.type_parameters.is_empty() {
        Ok(())
    } else {
        let args = ast_fn_decl
            .parameters
            .iter()
            .map(|param| {
                convert_resolved_typeid(context, &param.r#type, &param.type_span)
                    .map(|ty| (param.name.as_str().into(), ty, param.name.span().clone()))
            })
            .collect::<Result<Vec<(String, Type, Span)>, String>>()?;

        compile_fn_with_args(context, module, ast_fn_decl, args, None)
    }
}

// -------------------------------------------------------------------------------------------------

fn compile_fn_with_args(
    context: &mut Context,
    module: Module,
    ast_fn_decl: TypedFunctionDeclaration,
    args: Vec<(String, Type, Span)>,
    selector: Option<[u8; 4]>,
) -> Result<(), String> {
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

    let mut compiler = FnCompiler::new(context, module, func);

    let ret_val = compiler.compile_code_block(context, body)?;
    compiler
        .current_block
        .ins(context)
        .ret(ret_val, ret_type, None);
    Ok(())
}

// -------------------------------------------------------------------------------------------------

fn compile_impl(
    context: &mut Context,
    module: Module,
    self_type: TypeInfo,
    ast_methods: Vec<TypedFunctionDeclaration>,
) -> Result<(), String> {
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
            .collect::<Result<Vec<(String, Type, Span)>, String>>()?;

        compile_fn_with_args(context, module, method, args, None)?;
    }
    Ok(())
}

// -------------------------------------------------------------------------------------------------

fn compile_abi_method(
    context: &mut Context,
    module: Module,
    ast_fn_decl: TypedFunctionDeclaration,
) -> Result<(), String> {
    let selector = ast_fn_decl.to_fn_selector_value().value.ok_or(format!(
        "Cannot generate selector for ABI method: {}",
        ast_fn_decl.name.as_str()
    ))?;

    let args = ast_fn_decl
        .parameters
        .iter()
        .map(|param| {
            convert_resolved_typeid(context, &param.r#type, &param.type_span)
                .map(|ty| (param.name.as_str().into(), ty, param.name.span().clone()))
        })
        .collect::<Result<Vec<(String, Type, Span)>, String>>()?;

    compile_fn_with_args(context, module, ast_fn_decl, args, Some(selector))
}

// -------------------------------------------------------------------------------------------------

struct FnCompiler {
    module: Module,
    function: Function,
    current_block: Block,
    symbol_map: HashMap<String, String>,
}

impl FnCompiler {
    fn new(context: &mut Context, module: Module, function: Function) -> Self {
        let symbol_map = HashMap::from_iter(
            function
                .args_iter(context)
                .map(|(name, _value)| (name.clone(), name.clone())),
        );
        FnCompiler {
            module,
            function,
            current_block: function.get_entry_block(context),
            symbol_map,
        }
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_code_block(
        &mut self,
        context: &mut Context,
        ast_block: TypedCodeBlock,
    ) -> Result<Value, String> {
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
                        TypedDeclaration::FunctionDeclaration(_) => Err("func decl".into()),
                        TypedDeclaration::TraitDeclaration(_) => Err("trait decl".into()),
                        TypedDeclaration::StructDeclaration(_) => Err("struct decl".into()),
                        TypedDeclaration::EnumDeclaration(ted) => {
                            let span_md_idx = MetadataIndex::from_span(context, &ted.span);
                            compile_enum_decl(context, ted).map(|_| ())?;
                            Ok(Constant::get_unit(context, span_md_idx))
                        }
                        TypedDeclaration::Reassignment(tr) => {
                            self.compile_reassignment(context, tr, span_md_idx)
                        }
                        TypedDeclaration::ImplTrait { span, .. } => {
                            // XXX What if I ignore the trait implementation???  Potentially since
                            // we currently inline everything and below we 'recreate' the functions
                            // lazily as they are called, nothing needs to be done here.  BUT!
                            // This is obviously not really correct, and eventually we want to
                            // compile and then call these properly.
                            let span_md_idx = MetadataIndex::from_span(context, &span);
                            Ok(Constant::get_unit(context, span_md_idx))
                        }
                        TypedDeclaration::AbiDeclaration(_) => Err("abi decl".into()),
                        TypedDeclaration::GenericTypeForFunctionScope { .. } => {
                            Err("gen ty for fn scope".into())
                        }
                        TypedDeclaration::ErrorRecovery { .. } => Err("error recovery".into()),
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
                    TypedAstNodeContent::SideEffect => Err("code block side effect".into()),
                }
            })
            .collect::<Result<Vec<_>, String>>()
            .map(|vals| vals.last().cloned())
            .transpose()
            .unwrap_or_else(|| Ok(Constant::get_unit(context, None)))
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_expression(
        &mut self,
        context: &mut Context,
        ast_expr: TypedExpression,
    ) -> Result<Value, String> {
        let span_md_idx = MetadataIndex::from_span(context, &ast_expr.span);
        match ast_expr.expression {
            TypedExpressionVariant::Literal(l) => Ok(convert_literal_to_value(context, &l, span_md_idx)),
            TypedExpressionVariant::FunctionApplication {
                name,
                arguments,
                function_body,
                ..
            } => self.compile_fn_call(
                context,
                name.suffix.as_str(),
                arguments,
                Some(function_body),
                span_md_idx,
            ),
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
            TypedExpressionVariant::StructExpression {
                struct_name,
                fields,
            } => self.compile_struct_expr(context, struct_name.as_str(), fields, span_md_idx),
            TypedExpressionVariant::CodeBlock(cb) => self.compile_code_block(context, cb),
            TypedExpressionVariant::FunctionParameter => Err("expr func param".into()),
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
                self.compile_asm_expr(context, registers, body, returns, span_md_idx)
            }
            TypedExpressionVariant::StructFieldAccess {
                prefix,
                field_to_access,
                resolved_type_of_parent,
                field_to_access_span,
                ..
            } => {
                let span_md_idx = MetadataIndex::from_span(context, &field_to_access_span);
                self.compile_struct_field_expr(
                    context,
                    *prefix,
                    field_to_access,
                    resolved_type_of_parent,
                    span_md_idx,
                )
            }
            TypedExpressionVariant::EnumInstantiation {
                enum_decl,
                tag,
                contents,
                ..
            } => self.compile_enum_expr(context, enum_decl, tag, contents),
            TypedExpressionVariant::EnumArgAccess {
                //Prefix: Box<TypedExpression>,
                //Arg_num_to_access: usize,
                //Resolved_type_of_parent: TypeId,
                ..
            } => Err("enum arg access".into()),
            TypedExpressionVariant::Tuple {
               fields
            } => self.compile_tuple_expr(context, fields, span_md_idx),
            TypedExpressionVariant::TupleElemAccess {
                prefix,
                elem_to_access_num: idx,
                elem_to_access_span: span,
                resolved_type_of_parent: tuple_type,
            } => self.compile_tuple_elem_expr( context, *prefix, tuple_type, idx, span),
            // XXX IGNORE FOR NOW?
            TypedExpressionVariant::AbiCast { span, .. } => {
                let span_md_idx = MetadataIndex::from_span(context, &span);
                Ok(Constant::get_unit(context, span_md_idx))
            }
        }
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_return_statement(
        &mut self,
        context: &mut Context,
        ast_expr: TypedExpression,
    ) -> Result<Value, String> {
        let span_md_idx = MetadataIndex::from_span(context, &ast_expr.span);
        let ret_value = self.compile_expression(context, ast_expr)?;
        match ret_value.get_type(context) {
            None => Err("Unable to determine type for return statement expression.".into()),
            Some(ret_ty) => {
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
    ) -> Result<Value, String> {
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

    fn compile_fn_call(
        &mut self,
        context: &mut Context,
        ast_name: &str,
        ast_args: Vec<(Ident, TypedExpression)>,
        callee_body: Option<TypedCodeBlock>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, String> {
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

        // Edge case: take note as to whether the called function has the same name as this
        // function.  If so we'll get confused and try and recurse.  This is only a problem while
        // we don't have absolute paths to callees and while function bodies are inlined at call
        // sites.
        let has_same_name = self.function.get_name(context) == ast_name;

        match context
            .module_iter()
            .flat_map(|module| module.function_iter(context))
            .find(|function| !has_same_name && function.get_name(context) == ast_name)
        {
            Some(callee) => {
                let args = ast_args
                    .into_iter()
                    .map(|(_, expr)| self.compile_expression(context, expr))
                    .collect::<Result<Vec<Value>, String>>()?;
                Ok(self
                    .current_block
                    .ins(context)
                    .call(callee, &args, span_md_idx))
            }

            None if callee_body.is_none() => Err(format!("function not found: {}", ast_name)),

            None => {
                // Firstly create the single-use callee by fudging an AST declaration.
                let callee_name = context.get_unique_name();
                let callee_name_len = callee_name.len();
                let callee_ident = Ident::new(crate::span::Span {
                    span: pest::Span::new(
                        std::sync::Arc::from(callee_name.clone()),
                        0,
                        callee_name_len,
                    )
                    .unwrap(),
                    path: None,
                });

                let parameters = ast_args
                    .iter()
                    .map(|(name, expr)| TypedFunctionParameter {
                        name: name.clone(),
                        r#type: expr.return_type,
                        type_span: crate::span::Span {
                            span: pest::Span::new(" ".into(), 0, 0).unwrap(),
                            path: None,
                        },
                    })
                    .collect();

                let callee_body = callee_body.unwrap();

                // We're going to have to reverse engineer the return type.
                let return_type =
                    Self::get_codeblock_return_type(&callee_body).unwrap_or_else(||
                    // This code block is missing a return or implicit return.  The only time I've
                    // seen it happen (whether it's 'valid' or not) is in std::storage::store(),
                    // which has a single asm block which also returns nothing.  In this case, it
                    // actually is Unit.
                    insert_type(TypeInfo::Tuple(Vec::new())));

                let callee_fn_decl = TypedFunctionDeclaration {
                    name: callee_ident,
                    body: callee_body,
                    parameters,
                    span: crate::span::Span {
                        span: pest::Span::new(" ".into(), 0, 0).unwrap(),
                        path: None,
                    },
                    return_type,
                    type_parameters: Vec::new(),
                    return_type_span: crate::span::Span {
                        span: pest::Span::new(" ".into(), 0, 0).unwrap(),
                        path: None,
                    },
                    visibility: Visibility::Private,
                    is_contract_call: false,
                    purity: Default::default(),
                };

                compile_function(context, self.module, callee_fn_decl)?;

                // Then recursively create a call to it.
                self.compile_fn_call(context, &callee_name, ast_args, None, span_md_idx)
            }
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
    ) -> Result<Value, String> {
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

    fn compile_while_loop(
        &mut self,
        context: &mut Context,
        ast_while_loop: TypedWhileLoop,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, String> {
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
    ) -> Result<Value, String> {
        // We need to check the symbol map first, in case locals are shadowing the args, other
        // locals or even constants.
        if let Some(ptr) = self
            .symbol_map
            .get(name)
            .and_then(|local_name| self.function.get_local_ptr(context, local_name))
        {
            Ok(if ptr.is_struct_ptr(context) {
                self.current_block.ins(context).get_ptr(ptr, span_md_idx)
            } else {
                self.current_block.ins(context).load(ptr, span_md_idx)
            })
        } else if let Some(val) = self.function.get_arg(context, name) {
            Ok(val)
        } else if let Some(const_val) = self.module.get_global_constant(context, name) {
            Ok(const_val)
        } else {
            Err(format!("Unable to resolve variable '{}'.", name))
        }
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_var_decl(
        &mut self,
        context: &mut Context,
        ast_var_decl: TypedVariableDeclaration,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, String> {
        let TypedVariableDeclaration {
            name,
            body,
            is_mutable,
            ..
        } = ast_var_decl;

        // We must compile the RHS before checking for shadowing, as it will still be in the
        // previous scope.
        let return_type = convert_resolved_typeid(context, &body.return_type, &body.span)?;
        let init_val = self.compile_expression(context, body)?;

        let local_name = match self.symbol_map.get(name.as_str()) {
            None => {
                // Haven't seen this name before.  Use it as-is.
                name.as_str().to_owned()
            }
            Some(shadowed_name) => {
                // Seen before, and this is shadowing the old one.  Update to a new name.
                format!("{}_", shadowed_name)
            }
        };
        self.symbol_map
            .insert(name.as_str().to_owned(), local_name.clone());

        let ptr = self.function.new_local_ptr(
            context,
            local_name,
            return_type,
            is_mutable.into(),
            None,
        )?;

        self.current_block
            .ins(context)
            .store(ptr, init_val, span_md_idx);
        Ok(init_val)
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_const_decl(
        &mut self,
        context: &mut Context,
        ast_const_decl: TypedConstantDeclaration,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, String> {
        // This is local to the function, so we add it to the locals, rather than the module
        // globals like other const decls.
        let TypedConstantDeclaration { name, value, .. } = ast_const_decl;

        if let TypedExpressionVariant::Literal(literal) = &value.expression {
            let initialiser = convert_literal_to_constant(literal);
            let return_type = convert_resolved_typeid(context, &value.return_type, &value.span)?;
            let name = name.as_str().to_owned();
            self.function.new_local_ptr(
                context,
                name.clone(),
                return_type,
                false,
                Some(initialiser),
            )?;

            // We still insert this into the symbol table, as itself... can they be shadowed?
            // (Hrmm, name resolution in the variable expression code could be smarter about var
            // decls vs const decls, for now they're essentially the same...)
            self.symbol_map.insert(name.clone(), name);

            Ok(Constant::get_unit(context, span_md_idx))
        } else {
            Err("Unsupported constant declaration type.".into())
        }
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_reassignment(
        &mut self,
        context: &mut Context,
        ast_reassignment: TypedReassignment,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, String> {
        let name = ast_reassignment.lhs[0].name.as_str();
        let ptr_val = self
            .function
            .get_local_ptr(context, name)
            .ok_or(format!("variable not found: {}", name))?;

        let reassign_val = self.compile_expression(context, ast_reassignment.rhs)?;

        if ast_reassignment.lhs.len() == 1 {
            // A non-aggregate; use a `store`.
            self.current_block
                .ins(context)
                .store(ptr_val, reassign_val, span_md_idx);
        } else {
            // An aggregate.  Iterate over the field names from the left hand side and collect
            // field indices.
            let field_idcs = ast_reassignment.lhs[1..]
                .iter()
                .fold(
                    Ok((Vec::new(), *ptr_val.get_type(context))),
                    |acc, field_name| {
                        // Make sure we have an aggregate to index into.
                        acc.and_then(|(mut fld_idcs, ty)| match ty {
                            Type::Struct(aggregate) => {
                                // Get the field index and also its type for the next iteration.
                                match context
                                    .get_aggregate_index(&aggregate, field_name.name.as_str())
                                {
                                    None => Err(format!(
                                        "Unknown field name {} for struct ???",
                                        field_name.name.as_str()
                                    )),
                                    Some(field_idx) => {
                                        let field_type = context.aggregates[aggregate.0]
                                            .field_types()
                                            [field_idx as usize];

                                        // Save the field index.
                                        fld_idcs.push(field_idx);
                                        Ok((fld_idcs, field_type))
                                    }
                                }
                            }
                            _otherwise => {
                                Err("Reassignment with multiple accessors to non-aggregate.".into())
                            }
                        })
                    },
                )?
                .0;

            let ty = match ptr_val.get_type(context) {
                Type::Struct(aggregate) => *aggregate,
                _otherwise => {
                    return Err("Reassignment with multiple accessors to non-aggregate.".into())
                }
            };

            let get_ptr_val = self
                .current_block
                .ins(context)
                .get_ptr(ptr_val, span_md_idx);
            self.current_block.ins(context).insert_value(
                get_ptr_val,
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

    fn compile_array_expr(
        &mut self,
        context: &mut Context,
        contents: Vec<TypedExpression>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, String> {
        if contents.is_empty() {
            return Err("Unable to create zero sized static arrays.".into());
        }

        // Create a new aggregate, since they're not named.
        let elem_type = convert_resolved_typeid_no_span(context, &contents[0].return_type)?;
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
    ) -> Result<Value, String> {
        let array_val = self.compile_expression(context, array_expr)?;
        let aggregate = match &context.values[array_val.0].value {
            ValueDatum::Instruction(instruction) => {
                instruction.get_aggregate(context).ok_or_else(|| {
                    format!(
                        "Unsupported instruction as array value for index expression. {:?}",
                        instruction
                    )
                })
            }
            ValueDatum::Argument(Type::Array(aggregate)) => Ok(*aggregate),
            otherwise => Err(format!(
                "Unsupported array value for index expression: {:?}",
                otherwise
            )),
        }?;

        // Check for out of bounds if we have a literal index.
        let (_, count) = context.aggregates[aggregate.0].array_type();
        if let TypedExpressionVariant::Literal(Literal::U64(index)) = index_expr.expression {
            if index >= *count {
                // XXX Here is a very specific case where we want to return an Error enum
                // specifically, if not an actual CompileError.  This should be a
                // CompileError::ArrayOutOfBounds, or at least converted to one.
                return Err(format!(
                    "Array index out of bounds; the length is {} but the index is {}.",
                    *count, index
                ));
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
        struct_name: &str,
        fields: Vec<TypedStructExpressionField>,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, String> {
        let aggregate = context
            .get_aggregate_by_name(struct_name)
            .ok_or_else(|| format!("Unknown aggregate {}", struct_name))?;

        // Compile each of the values for field initialisers and calculate their indices.
        let inserted_values_indices = fields
            .into_iter()
            .map(|field_value| {
                let name = field_value.name.as_str();
                self.compile_expression(context, field_value.value)
                    .and_then(|insert_val| {
                        context
                            .get_aggregate_index(&aggregate, name)
                            .ok_or_else(|| {
                                format!("Unknown field name {} for aggregate {}", name, struct_name)
                            })
                            .map(|insert_idx| (insert_val, insert_idx))
                    })
            })
            .collect::<Result<Vec<_>, String>>()?;

        // Start with a constant empty struct and then fill in the values.
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
        ast_field: OwnedTypedStructField,
        _ast_parent_type: TypeId,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, String> {
        let struct_val = self.compile_expression(context, ast_struct_expr)?;
        let aggregate = match &context.values[struct_val.0].value {
            ValueDatum::Instruction(instruction) => {
                instruction.get_aggregate(context).ok_or_else(|| {
                    format!(
                        "Unsupported instruction as struct value for field expression. {:?}",
                        instruction
                    )
                })
            }
            ValueDatum::Argument(Type::Struct(aggregate)) => Ok(*aggregate),
            otherwise => Err(format!(
                "Unsupported struct value for field expression: {:?}",
                otherwise
            )),
        }?;

        let field_idx = context
            .get_aggregate_index(&aggregate, &ast_field.name)
            .ok_or_else(|| format!("Unknown field name {} in struct ???", ast_field.name))?;

        Ok(self.current_block.ins(context).extract_value(
            struct_val,
            aggregate,
            vec![field_idx],
            span_md_idx,
        ))
    }

    // ---------------------------------------------------------------------------------------------
    // As per compile_enum_decl(), these are tagged unions.

    fn compile_enum_expr(
        &mut self,
        context: &mut Context,
        enum_decl: TypedEnumDeclaration,
        tag: usize,
        contents: Option<Box<TypedExpression>>,
    ) -> Result<Value, String> {
        // XXX The enum instantiation AST node includes the full declaration.  If the enum was
        // declared in a different module then it seems for now there's no easy way to pre-analyse
        // it and add its type/aggregate to the context.  We can re-use them here if we recognise
        // the name, and if not add a new aggregate... OTOH the naming seems a little fragile and
        // we could potentially use the wrong aggregate with the same name, different module...
        // dunno.
        let span_md_idx = MetadataIndex::from_span(context, &enum_decl.span);
        let aggregate = match context.get_aggregate_by_name(enum_decl.name.as_str()) {
            Some(agg) => Ok(agg),
            None => compile_enum_decl(context, enum_decl),
        }?;
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
    ) -> Result<Value, String> {
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
                .collect::<Result<Vec<_>, String>>()?
                .into_iter()
                .unzip();

            let aggregate = Aggregate::new_struct(context, None, init_types);
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
    ) -> Result<Value, String> {
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
            Err("Invalid (non-aggregate?) tuple type for TupleElemAccess?".into())
        }
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_asm_expr(
        &mut self,
        context: &mut Context,
        registers: Vec<TypedAsmRegisterDeclaration>,
        body: Vec<AsmOp>,
        returns: Option<(AsmRegister, Span)>,
        whole_block_span_md_idx: Option<MetadataIndex>,
    ) -> Result<Value, String> {
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
            .collect::<Result<Vec<AsmArg>, String>>()?;
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
        let returns = returns.as_ref().map(|(asm_reg, _)| {
            Ident::new(Span {
                span: pest::Span::new(asm_reg.name.as_str().into(), 0, asm_reg.name.len()).unwrap(),
                path: None,
            })
        });
        Ok(self.current_block.ins(context).asm_block(
            registers,
            body,
            returns,
            whole_block_span_md_idx,
        ))
    }
}

// -------------------------------------------------------------------------------------------------

fn convert_literal_to_value(
    context: &mut Context,
    ast_literal: &Literal,
    span_id_idx: Option<MetadataIndex>,
) -> Value {
    match ast_literal {
        Literal::U8(n) | Literal::Byte(n) => Constant::get_uint(context, 8, *n as u64, span_id_idx),
        Literal::U16(n) => Constant::get_uint(context, 16, *n as u64, span_id_idx),
        Literal::U32(n) => Constant::get_uint(context, 32, *n as u64, span_id_idx),
        Literal::U64(n) => Constant::get_uint(context, 64, *n, span_id_idx),
        Literal::Numeric(n) => Constant::get_uint(context, 64, *n, span_id_idx),
        Literal::String(s) => Constant::get_string(context, s.as_str().to_owned(), span_id_idx),
        Literal::Boolean(b) => Constant::get_bool(context, *b, span_id_idx),
        Literal::B256(bs) => Constant::get_b256(context, *bs, span_id_idx),
    }
}

fn convert_literal_to_constant(ast_literal: &Literal) -> Constant {
    match ast_literal {
        Literal::U8(n) | Literal::Byte(n) => Constant::new_uint(8, *n as u64),
        Literal::U16(n) => Constant::new_uint(16, *n as u64),
        Literal::U32(n) => Constant::new_uint(32, *n as u64),
        Literal::U64(n) => Constant::new_uint(64, *n),
        Literal::Numeric(n) => Constant::new_uint(64, *n),
        Literal::String(s) => Constant::new_string(s.as_str().to_owned()),
        Literal::Boolean(b) => Constant::new_bool(*b),
        Literal::B256(bs) => Constant::new_b256(*bs),
    }
}

fn convert_resolved_typeid(
    context: &mut Context,
    ast_type: &TypeId,
    span: &Span,
) -> Result<Type, String> {
    // There's probably a better way to convert TypeError to String, but... we'll use something
    // other than String eventually?  IrError?
    convert_resolved_type(
        context,
        &resolve_type(*ast_type, span).map_err(|ty_err| format!("{:?}", ty_err))?,
    )
}

fn convert_resolved_typeid_no_span(
    context: &mut Context,
    ast_type: &TypeId,
) -> Result<Type, String> {
    let span = crate::span::Span {
        span: pest::Span::new(" ".into(), 0, 0).unwrap(),
        path: None,
    };
    convert_resolved_typeid(context, ast_type, &span)
}

fn convert_resolved_type(context: &mut Context, ast_type: &TypeInfo) -> Result<Type, String> {
    Ok(match ast_type {
        TypeInfo::UnsignedInteger(nbits) => {
            // We need impl IntegerBits { fn num_bits() -> u64 { ... } }
            let nbits = match nbits {
                IntegerBits::Eight => 8,
                IntegerBits::Sixteen => 16,
                IntegerBits::ThirtyTwo => 32,
                IntegerBits::SixtyFour => 64,
            };
            Type::Uint(nbits)
        }
        TypeInfo::Numeric => Type::Uint(64),
        TypeInfo::Boolean => Type::Bool,
        TypeInfo::Byte => Type::Uint(8), // XXX?
        TypeInfo::B256 => Type::B256,
        TypeInfo::Str(n) => Type::String(*n),
        TypeInfo::Struct { name, fields } => match context.get_aggregate_by_name(name) {
            Some(existing_aggregate) => Type::Struct(existing_aggregate),
            None => {
                // Let's create a new aggregate from the TypeInfo.
                create_struct_aggregate(context, name.clone(), fields.clone()).map(&Type::Struct)?
            }
        },
        TypeInfo::Enum {
            name,
            variant_types,
        } => {
            match context.get_aggregate_by_name(name) {
                Some(existing_aggregate) => Type::Struct(existing_aggregate),
                None => {
                    // Let's create a new aggregate from the TypeInfo.
                    create_enum_aggregate(context, name.clone(), variant_types.clone())
                        .map(&Type::Struct)?
                }
            }
        }
        TypeInfo::Array(elem_type_id, count) => {
            let elem_type = convert_resolved_typeid_no_span(context, elem_type_id)?;
            Type::Array(Aggregate::new_array(context, elem_type, *count as u64))
        }
        TypeInfo::Tuple(fields) => {
            if fields.is_empty() {
                // XXX We've removed Unit from the core compiler, replaced with an empty Tuple.
                // Perhaps the same should be done for the IR, although it would use an empty
                // aggregate which might not make as much sense as a dedicated Unit type.
                Type::Unit
            } else {
                create_tuple_aggregate(context, fields.clone()).map(Type::Struct)?
            }
        }
        TypeInfo::Custom { .. } => return Err("can't do custom types yet".into()),
        TypeInfo::SelfType { .. } => return Err("can't do self types yet".into()),
        TypeInfo::Contract => Type::Contract,
        TypeInfo::ContractCaller { abi_name, address } => Type::ContractCaller(AbiInstance::new(
            context,
            abi_name.prefixes.clone(),
            abi_name.suffix.clone(),
            address.clone(),
        )),
        TypeInfo::Unknown => return Err("unknown type found in AST..?".into()),
        TypeInfo::UnknownGeneric { .. } => return Err("unknowngeneric type found in AST..?".into()),
        TypeInfo::Ref(_) => return Err("ref type found in AST..?".into()),
        TypeInfo::ErrorRecovery => return Err("error recovery type found in AST..?".into()),
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
                    path.file_name().unwrap_or_else(|| path.as_os_str())
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

        let typed_ast = parse_to_typed_ast(sw_path, &input);
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
        }
        assert_eq!(output, expected);
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
                    path.file_name().unwrap_or_else(|| path.as_os_str())
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
        }
        assert_eq!(input, printed);
    }

    // -------------------------------------------------------------------------------------------------

    fn parse_to_typed_ast(path: PathBuf, input: &str) -> TypedParseTree {
        let mut parsed =
            SwayParser::parse(Rule::program, std::sync::Arc::from(input)).expect("parse_tree");

        let dir_of_code = std::sync::Arc::new(path.parent().unwrap().into());
        let file_name = std::sync::Arc::new(path);

        let build_config = crate::build_config::BuildConfig {
            file_name,
            dir_of_code,
            manifest_path: std::sync::Arc::new(".".into()),
            use_ir: false,
            print_intermediate_asm: false,
            print_finalized_asm: false,
            print_ir: false,
            generated_names: std::sync::Arc::new(std::sync::Mutex::new(vec![])),
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
            &TreeType::Script,
            &build_config,
            &mut dead_code_graph,
            &mut std::collections::HashMap::new(),
        )
        .unwrap(&mut warnings, &mut errors)
    }
}

// -------------------------------------------------------------------------------------------------
