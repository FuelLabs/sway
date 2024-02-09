use super::{
    compile::compile_function,
    convert::*,
    lexical_map::LexicalMap,
    storage::{add_to_b256, get_storage_key},
    types::*,
};
use crate::{
    engine_threading::*,
    ir_generation::const_eval::{
        compile_constant_expression, compile_constant_expression_to_constant,
    },
    language::{
        ty::{self, ProjectionKind, TyConstantDecl, TyExpressionVariant},
        *,
    },
    metadata::MetadataManager,
    type_system::*,
    types::*,
};
use indexmap::IndexMap;
use sway_ast::intrinsics::Intrinsic;
use sway_error::error::CompileError;
use sway_ir::{Context, *};
use sway_types::{
    constants,
    ident::Ident,
    integer_bits::IntegerBits,
    span::{Span, Spanned},
    state::StateIndex,
    u256::U256,
    Named,
};

use std::collections::HashMap;

/// Engine for compiling a function and all of the AST nodes within.
///
/// This is mostly recursively compiling expressions, as Sway is fairly heavily expression based.
///
/// The rule here is to use `compile_expression_to_value()` when a value is desired, as opposed to a
/// pointer. This is most of the time, as we try to be target agnostic and not make assumptions
/// about which values must be used by reference.
///
/// `compile_expression_to_value()` will force the result to be a value, by using a temporary if
/// necessary.
///
/// `compile_expression_to_ptr()` will compile the expression and force it to be a pointer, also by
/// using a temporary if necessary. This can be slightly dangerous, if the reference is supposed
/// to be to a particular value but is accidentally made to a temporary value then mutations or
/// other side-effects might not be applied in the correct context.
///
/// `compile_expression()` will compile the expression without forcing anything. If the expression
/// has a reference type, like getting a struct or an explicit ref arg, it will return a pointer
/// value, but otherwise will return a value.
///
/// So in general the methods in [FnCompiler] will return a pointer if they can and will get it, be
/// forced, into a value if that is desired. All the temporary values are manipulated with simple
/// loads and stores, rather than anything more complicated like `mem_copy`s.

// Wrapper around Value to enforce distinction between terminating and non-terminating values.
struct TerminatorValue {
    value: Value,
    is_terminator: bool,
}

impl TerminatorValue {
    pub fn new(value: Value, context: &Context) -> Self {
        Self {
            value,
            is_terminator: value.is_terminator(context),
        }
    }
}

// If the provided TerminatorValue::is_terminator is true, then return from the current function
// immediately. Otherwise extract the embedded Value.
macro_rules! return_on_termination_or_extract {
    ($value:expr) => {{
        let val = $value;
        if val.is_terminator {
            return Ok(val);
        };
        val.value
    }};
}

pub(crate) struct FnCompiler<'eng> {
    engines: &'eng Engines,
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
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        engines: &'eng Engines,
        context: &mut Context,
        module: Module,
        function: Function,
        logged_types_map: &HashMap<TypeId, LogId>,
        messages_types_map: &HashMap<TypeId, MessageId>,
    ) -> Self {
        let lexical_map = LexicalMap::from_iter(
            function
                .args_iter(context)
                .map(|(name, _value)| name.clone()),
        );
        FnCompiler {
            engines,
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

    fn compile_with_new_scope<F, T, R>(&mut self, inner: F) -> Result<T, R>
    where
        F: FnOnce(&mut FnCompiler) -> Result<T, R>,
    {
        self.lexical_map.enter_scope();
        let result = inner(self);
        self.lexical_map.leave_scope();
        result
    }

    pub(super) fn compile_code_block_to_value(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_block: &ty::TyCodeBlock,
    ) -> Result<Value, Vec<CompileError>> {
        Ok(self.compile_code_block(context, md_mgr, ast_block)?.value)
    }

    fn compile_code_block(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_block: &ty::TyCodeBlock,
    ) -> Result<TerminatorValue, Vec<CompileError>> {
        self.compile_with_new_scope(|fn_compiler| {
            let mut errors = vec![];

            let mut ast_nodes = ast_block.contents.iter();
            let v = loop {
                let ast_node = match ast_nodes.next() {
                    Some(ast_node) => ast_node,
                    None => break TerminatorValue::new(Constant::get_unit(context), context),
                };
                match fn_compiler.compile_ast_node(context, md_mgr, ast_node) {
                    // 'Some' indicates an implicit return or a diverging expression, so break.
                    Ok(Some(val)) => break val,
                    Ok(None) => (),
                    Err(e) => {
                        errors.push(e);
                    }
                }
            };

            if !errors.is_empty() {
                Err(errors)
            } else {
                Ok(v)
            }
        })
    }

    fn compile_ast_node(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_node: &ty::TyAstNode,
    ) -> Result<Option<TerminatorValue>, CompileError> {
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
                ty::TyDecl::ConstantDecl(ty::ConstantDecl { decl_id, .. }) => {
                    let tcd = self.engines.de().get_constant(decl_id);
                    self.compile_const_decl(context, md_mgr, &tcd, span_md_idx, false)?;
                    Ok(None)
                }
                ty::TyDecl::EnumDecl(ty::EnumDecl { decl_id, .. }) => {
                    let ted = self.engines.de().get_enum(decl_id);
                    create_tagged_union_type(
                        self.engines.te(),
                        self.engines.de(),
                        context,
                        &ted.variants,
                    )
                    .map(|_| ())?;
                    Ok(None)
                }
                ty::TyDecl::TypeAliasDecl { .. } => Err(CompileError::UnexpectedDeclaration {
                    decl_type: "type alias",
                    span: ast_node.span.clone(),
                }),
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
                ty::TyDecl::GenericTypeForFunctionScope { .. } => unexpected_decl("generic type"),
                ty::TyDecl::ErrorRecovery { .. } => unexpected_decl("error recovery"),
                ty::TyDecl::StorageDecl { .. } => unexpected_decl("storage"),
                ty::TyDecl::EnumVariantDecl { .. } => unexpected_decl("enum variant"),
                ty::TyDecl::TraitTypeDecl { .. } => unexpected_decl("trait type"),
            },
            ty::TyAstNodeContent::Expression(te) => {
                match &te.expression {
                    TyExpressionVariant::ImplicitReturn(exp) => self
                        .compile_expression_to_value(context, md_mgr, exp)
                        .map(Some),
                    _ => {
                        // An expression with an ignored return value... I assume.
                        let value = self.compile_expression_to_value(context, md_mgr, te)?;
                        // Terminating values should end the compilation of the block
                        if value.is_terminator {
                            Ok(Some(value))
                        } else {
                            Ok(None)
                        }
                    }
                }
            }
            // a side effect can be () because it just impacts the type system/namespacing.
            // There should be no new IR generated.
            ty::TyAstNodeContent::SideEffect(_) => Ok(None),
            ty::TyAstNodeContent::Error(_, _) => {
                unreachable!("error node found when generating IR");
            }
        }
    }

    fn compile_expression_to_value(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_expr: &ty::TyExpression,
    ) -> Result<TerminatorValue, CompileError> {
        // Compile expression which *may* be a pointer.  We can't return a pointer value here
        // though, so add a `load` to it.
        self.compile_expression(context, md_mgr, ast_expr)
            .map(|val| {
                if val
                    .value
                    .get_type(context)
                    .map_or(false, |ty| ty.is_ptr(context))
                {
                    let load_val = self.current_block.append(context).load(val.value);
                    TerminatorValue::new(load_val, context)
                } else {
                    val
                }
            })
    }

    fn compile_expression_to_ptr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_expr: &ty::TyExpression,
    ) -> Result<TerminatorValue, CompileError> {
        // Compile expression which *may* be a pointer.  We can't return a value so create a
        // temporary here, store the value and return its pointer.
        let val =
            return_on_termination_or_extract!(self.compile_expression(context, md_mgr, ast_expr)?);
        let ty = match val.get_type(context) {
            Some(ty) if !ty.is_ptr(context) => ty,
            _ => return Ok(TerminatorValue::new(val, context)),
        };

        // Create a temporary.
        let temp_name = self.lexical_map.insert_anon();
        let tmp_var = self
            .function
            .new_local_var(context, temp_name, ty, None, false)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;
        let tmp_val = self.current_block.append(context).get_local(tmp_var);
        self.current_block.append(context).store(tmp_val, val);

        Ok(TerminatorValue::new(tmp_val, context))
    }

    fn compile_string_slice(
        &mut self,
        context: &mut Context,
        span_md_idx: Option<MetadataIndex>,
        string_data: Value,
        string_len: u64,
    ) -> Result<TerminatorValue, CompileError> {
        let int_ty = Type::get_uint64(context);

        // build field values of the slice
        let ptr_val = self
            .current_block
            .append(context)
            .ptr_to_int(string_data, int_ty)
            .add_metadatum(context, span_md_idx);
        let len_val = Constant::get_uint(context, 64, string_len);

        // a slice is a pointer and a length
        let field_types = vec![int_ty, int_ty];

        // build a struct variable to store the values
        let struct_type = Type::new_struct(context, field_types.clone());
        let struct_var = self
            .function
            .new_local_var(
                context,
                self.lexical_map.insert_anon(),
                struct_type,
                None,
                false,
            )
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;
        let struct_val = self
            .current_block
            .append(context)
            .get_local(struct_var)
            .add_metadatum(context, span_md_idx);

        // put field values inside the struct variable
        [ptr_val, len_val]
            .into_iter()
            .zip(field_types)
            .enumerate()
            .for_each(|(insert_idx, (insert_val, field_type))| {
                let gep_val = self.current_block.append(context).get_elem_ptr_with_idx(
                    struct_val,
                    field_type,
                    insert_idx as u64,
                );

                self.current_block
                    .append(context)
                    .store(gep_val, insert_val)
                    .add_metadatum(context, span_md_idx);
            });

        // build a slice variable to return
        let slice_type = Type::get_slice(context);
        let slice_var = self
            .function
            .new_local_var(
                context,
                self.lexical_map.insert_anon(),
                slice_type,
                None,
                false,
            )
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;
        let slice_val = self
            .current_block
            .append(context)
            .get_local(slice_var)
            .add_metadatum(context, span_md_idx);

        // copy the value of the struct variable into the slice
        self.current_block
            .append(context)
            .mem_copy_bytes(slice_val, struct_val, 16);

        // return the slice
        Ok(TerminatorValue::new(slice_val, context))
    }

    fn compile_expression(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_expr: &ty::TyExpression,
    ) -> Result<TerminatorValue, CompileError> {
        let span_md_idx = md_mgr.span_to_md(context, &ast_expr.span);
        match &ast_expr.expression {
            ty::TyExpressionVariant::Literal(Literal::String(s)) => {
                let string_data = Constant::get_string(context, s.as_str().as_bytes().to_vec());
                let string_len = s.as_str().len() as u64;
                self.compile_string_slice(context, span_md_idx, string_data, string_len)
            }
            ty::TyExpressionVariant::Literal(Literal::Numeric(n)) => {
                let implied_lit = match &*self.engines.te().get(ast_expr.return_type) {
                    TypeInfo::UnsignedInteger(IntegerBits::Eight) => Literal::U8(*n as u8),
                    TypeInfo::UnsignedInteger(IntegerBits::V256) => Literal::U256(U256::from(*n)),
                    _ =>
                    // Anything more than a byte needs a u64 (except U256 of course).
                    // (This is how convert_literal_to_value treats it too).
                    {
                        Literal::U64(*n)
                    }
                };
                let val = convert_literal_to_value(context, &implied_lit)
                    .add_metadatum(context, span_md_idx);
                Ok(TerminatorValue::new(val, context))
            }
            ty::TyExpressionVariant::Literal(l) => {
                let val = convert_literal_to_value(context, l).add_metadatum(context, span_md_idx);
                Ok(TerminatorValue::new(val, context))
            }
            ty::TyExpressionVariant::FunctionApplication {
                call_path: name,
                contract_call_params,
                arguments,
                fn_ref,
                selector,
                type_binding: _,
                call_path_typeid: _,
                deferred_monomorphization,
            } => {
                if *deferred_monomorphization {
                    return Err(CompileError::Internal("Trying to compile a deferred function application with deferred monomorphization", name.span()));
                }
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
                    let function_decl = self.engines.de().get_function(fn_ref);
                    self.compile_fn_call(context, md_mgr, arguments, &function_decl, span_md_idx)
                }
            }
            ty::TyExpressionVariant::LazyOperator { op, lhs, rhs } => {
                self.compile_lazy_op(context, md_mgr, op, lhs, rhs, span_md_idx)
            }
            ty::TyExpressionVariant::ConstantExpression { const_decl, .. } => {
                self.compile_const_expr(context, md_mgr, const_decl, span_md_idx)
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
            ty::TyExpressionVariant::CodeBlock(cb) => {
                //TODO return all errors
                self.compile_code_block(context, md_mgr, cb)
                    .map_err(|mut x| x.pop().unwrap())
            }
            ty::TyExpressionVariant::FunctionParameter => Err(CompileError::Internal(
                "Unexpected function parameter declaration.",
                ast_expr.span.clone(),
            )),
            ty::TyExpressionVariant::MatchExp { desugared, .. } => {
                self.compile_expression_to_value(context, md_mgr, desugared)
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
                let enum_decl = self.engines.de().get_enum(enum_ref);
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
                let val = Constant::get_unit(context).add_metadatum(context, span_md_idx);
                Ok(TerminatorValue::new(val, context))
            }
            ty::TyExpressionVariant::StorageAccess(access) => {
                let span_md_idx = md_mgr.span_to_md(context, &access.span());
                self.compile_storage_access(context, &access.fields, &access.ix, span_md_idx)
            }
            ty::TyExpressionVariant::IntrinsicFunction(kind) => {
                self.compile_intrinsic_function(context, md_mgr, kind, ast_expr.span.clone())
            }
            ty::TyExpressionVariant::AbiName(_) => {
                let val = Value::new_constant(context, Constant::new_unit(context));
                Ok(TerminatorValue::new(val, context))
            }
            ty::TyExpressionVariant::UnsafeDowncast {
                exp,
                variant,
                call_path_decl: _,
            } => self.compile_unsafe_downcast(context, md_mgr, exp, variant),
            ty::TyExpressionVariant::EnumTag { exp } => {
                self.compile_enum_tag(context, md_mgr, exp.to_owned())
            }
            ty::TyExpressionVariant::WhileLoop { body, condition } => {
                self.compile_while_loop(context, md_mgr, body, condition, span_md_idx)
            }
            ty::TyExpressionVariant::ForLoop { desugared } => {
                self.compile_expression(context, md_mgr, desugared)
            }
            ty::TyExpressionVariant::Break => {
                match self.block_to_break_to {
                    // If `self.block_to_break_to` is not None, then it has been set inside
                    // a loop and the use of `break` here is legal, so create a branch
                    // instruction. Error out otherwise.
                    Some(block_to_break_to) => {
                        let val = self
                            .current_block
                            .append(context)
                            .branch(block_to_break_to, vec![]);
                        Ok(TerminatorValue::new(val, context))
                    }
                    None => Err(CompileError::BreakOutsideLoop {
                        span: ast_expr.span.clone(),
                    }),
                }
            }
            ty::TyExpressionVariant::Continue { .. } => match self.block_to_continue_to {
                // If `self.block_to_continue_to` is not None, then it has been set inside
                // a loop and the use of `continue` here is legal, so create a branch
                // instruction. Error out otherwise.
                Some(block_to_continue_to) => {
                    let val = self
                        .current_block
                        .append(context)
                        .branch(block_to_continue_to, vec![]);
                    Ok(TerminatorValue::new(val, context))
                }
                None => Err(CompileError::ContinueOutsideLoop {
                    span: ast_expr.span.clone(),
                }),
            },
            ty::TyExpressionVariant::Reassignment(reassignment) => {
                self.compile_reassignment(context, md_mgr, reassignment, span_md_idx)
            }
            ty::TyExpressionVariant::ImplicitReturn(_exp) => {
                // This is currently handled at the top-level handler, `compile_ast_node`.
                unreachable!();
            }
            ty::TyExpressionVariant::Return(exp) => {
                self.compile_return(context, md_mgr, exp, span_md_idx)
            }
            ty::TyExpressionVariant::Ref(exp) => {
                self.compile_ref(context, md_mgr, exp, span_md_idx)
            }
            ty::TyExpressionVariant::Deref(exp) => {
                self.compile_deref(context, md_mgr, exp, span_md_idx)
            }
        }
    }

    fn compile_intrinsic_function(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        i @ ty::TyIntrinsicFunctionKind {
            kind,
            arguments,
            type_arguments,
            span: _,
        }: &ty::TyIntrinsicFunctionKind,
        span: Span,
    ) -> Result<TerminatorValue, CompileError> {
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
                .new_local_var(context, key_name, Type::get_b256(context), None, false)
                .map_err(|ir_error| {
                    CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                })?;

            // Convert the key variable to a value using get_local.
            let key_val = compiler
                .current_block
                .append(context)
                .get_local(key_var)
                .add_metadatum(context, span_md_idx);

            // Store the value to the key pointer value
            compiler
                .current_block
                .append(context)
                .store(key_val, value)
                .add_metadatum(context, span_md_idx);
            Ok(key_val)
        }

        let engines = self.engines;

        // We safely index into arguments and type_arguments arrays below
        // because the type-checker ensures that the arguments are all there.
        match kind {
            Intrinsic::SizeOfVal => {
                let exp = &arguments[0];
                // Compile the expression in case of side-effects but ignore its value.
                let ir_type = convert_resolved_typeid(
                    engines.te(),
                    engines.de(),
                    context,
                    &exp.return_type,
                    &exp.span,
                )?;
                self.compile_expression_to_value(context, md_mgr, exp)?;
                let val = Constant::get_uint(context, 64, ir_type.size(context).in_bytes());
                Ok(TerminatorValue::new(val, context))
            }
            Intrinsic::SizeOfType => {
                let targ = type_arguments[0].clone();
                let ir_type = convert_resolved_typeid(
                    engines.te(),
                    engines.de(),
                    context,
                    &targ.type_id,
                    &targ.span,
                )?;
                let val = Constant::get_uint(context, 64, ir_type.size(context).in_bytes());
                Ok(TerminatorValue::new(val, context))
            }
            Intrinsic::SizeOfStr => {
                let targ = type_arguments[0].clone();
                let ir_type = convert_resolved_typeid(
                    engines.te(),
                    engines.de(),
                    context,
                    &targ.type_id,
                    &targ.span,
                )?;
                let val = Constant::get_uint(
                    context,
                    64,
                    ir_type.get_string_len(context).unwrap_or_default(),
                );
                Ok(TerminatorValue::new(val, context))
            }
            Intrinsic::IsReferenceType => {
                let targ = type_arguments[0].clone();
                let is_val = !engines.te().get_unaliased(targ.type_id).is_copy_type();
                let val = Constant::get_bool(context, is_val);
                Ok(TerminatorValue::new(val, context))
            }
            Intrinsic::IsStrArray => {
                let targ = type_arguments[0].clone();
                let is_val = matches!(
                    &*engines.te().get_unaliased(targ.type_id),
                    TypeInfo::StringArray(_) | TypeInfo::StringSlice
                );
                let val = Constant::get_bool(context, is_val);
                Ok(TerminatorValue::new(val, context))
            }
            Intrinsic::AssertIsStrArray => {
                let targ = type_arguments[0].clone();
                let ir_type = convert_resolved_typeid(
                    engines.te(),
                    engines.de(),
                    context,
                    &targ.type_id,
                    &targ.span,
                )?;
                match ir_type.get_content(context) {
                    TypeContent::StringSlice | TypeContent::StringArray(_) => {
                        let val = Constant::get_unit(context);
                        Ok(TerminatorValue::new(val, context))
                    }
                    _ => Err(CompileError::NonStrGenericType {
                        span: targ.span.clone(),
                    }),
                }
            }
            Intrinsic::ToStrArray => match arguments[0].expression.extract_literal_value() {
                Some(Literal::String(span)) => {
                    let val = Constant::get_string(context, span.as_str().as_bytes().to_vec());
                    Ok(TerminatorValue::new(val, context))
                }
                _ => unreachable!(),
            },
            Intrinsic::Eq | Intrinsic::Gt | Intrinsic::Lt => {
                let lhs = &arguments[0];
                let rhs = &arguments[1];
                let lhs_value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, lhs)?
                );
                let rhs_value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, rhs)?
                );
                let pred = match kind {
                    Intrinsic::Eq => Predicate::Equal,
                    Intrinsic::Gt => Predicate::GreaterThan,
                    Intrinsic::Lt => Predicate::LessThan,
                    _ => unreachable!(),
                };
                let val = self
                    .current_block
                    .append(context)
                    .cmp(pred, lhs_value, rhs_value);
                Ok(TerminatorValue::new(val, context))
            }
            Intrinsic::Gtf => {
                // The index is just a Value
                let index = return_on_termination_or_extract!(self.compile_expression_to_value(
                    context,
                    md_mgr,
                    &arguments[0]
                )?);

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
                    false,
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
                    engines.te(),
                    engines.de(),
                    context,
                    &target_type.type_id,
                    &target_type.span,
                )?;

                let span_md_idx = md_mgr.span_to_md(context, &span);

                // The `gtf` instruction
                let gtf_reg = self
                    .current_block
                    .append(context)
                    .gtf(index, tx_field_id)
                    .add_metadatum(context, span_md_idx);

                // Reinterpret the result of the `gtf` instruction (which is always `u64`) as type
                // `T`. This requires an `int_to_ptr` instruction if `T` is a reference type.
                if engines
                    .te()
                    .get_unaliased(target_type.type_id)
                    .is_copy_type()
                {
                    let val = self
                        .current_block
                        .append(context)
                        .bitcast(gtf_reg, target_ir_type)
                        .add_metadatum(context, span_md_idx);
                    Ok(TerminatorValue::new(val, context))
                } else {
                    let ptr_ty = Type::new_ptr(context, target_ir_type);
                    let val = self
                        .current_block
                        .append(context)
                        .int_to_ptr(gtf_reg, ptr_ty)
                        .add_metadatum(context, span_md_idx);
                    Ok(TerminatorValue::new(val, context))
                }
            }
            Intrinsic::AddrOf => {
                let exp = &arguments[0];
                let value = return_on_termination_or_extract!(
                    self.compile_expression(context, md_mgr, exp)?
                );
                let int_ty = Type::new_uint(context, 64);
                let span_md_idx = md_mgr.span_to_md(context, &span);
                let val = self
                    .current_block
                    .append(context)
                    .ptr_to_int(value, int_ty)
                    .add_metadatum(context, span_md_idx);
                Ok(TerminatorValue::new(val, context))
            }
            Intrinsic::StateClear => {
                let key_exp = arguments[0].clone();
                let number_of_slots_exp = arguments[1].clone();
                let key_value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, &key_exp)?
                );
                let number_of_slots_value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, &number_of_slots_exp)?
                );
                let span_md_idx = md_mgr.span_to_md(context, &span);
                let key_var = store_key_in_local_mem(self, context, key_value, span_md_idx)?;
                let val = self
                    .current_block
                    .append(context)
                    .state_clear(key_var, number_of_slots_value)
                    .add_metadatum(context, span_md_idx);
                Ok(TerminatorValue::new(val, context))
            }
            Intrinsic::StateLoadWord => {
                let exp = &arguments[0];
                let value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, exp)?
                );
                let span_md_idx = md_mgr.span_to_md(context, &span);
                let key_var = store_key_in_local_mem(self, context, value, span_md_idx)?;
                let val = self
                    .current_block
                    .append(context)
                    .state_load_word(key_var)
                    .add_metadatum(context, span_md_idx);
                Ok(TerminatorValue::new(val, context))
            }
            Intrinsic::StateStoreWord => {
                let key_exp = &arguments[0];
                let val_exp = &arguments[1];
                // Validate that the val_exp is of the right type. We couldn't do it
                // earlier during type checking as the type arguments may not have been resolved.
                let val_ty = engines.te().get_unaliased(val_exp.return_type);
                if !val_ty.is_copy_type() {
                    return Err(CompileError::IntrinsicUnsupportedArgType {
                        name: kind.to_string(),
                        span,
                        hint: "This argument must be a copy type".to_string(),
                    });
                }
                let key_value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, key_exp)?
                );
                let val_value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, val_exp)?
                );
                let span_md_idx = md_mgr.span_to_md(context, &span);
                let key_var = store_key_in_local_mem(self, context, key_value, span_md_idx)?;
                let val = self
                    .current_block
                    .append(context)
                    .state_store_word(val_value, key_var)
                    .add_metadatum(context, span_md_idx);
                Ok(TerminatorValue::new(val, context))
            }
            Intrinsic::StateLoadQuad | Intrinsic::StateStoreQuad => {
                let key_exp = arguments[0].clone();
                let val_exp = arguments[1].clone();
                let number_of_slots_exp = arguments[2].clone();
                // Validate that the val_exp is of the right type. We couldn't do it
                // earlier during type checking as the type arguments may not have been resolved.
                let val_ty = engines.te().get_unaliased(val_exp.return_type);
                if !val_ty.eq(&TypeInfo::RawUntypedPtr, engines) {
                    return Err(CompileError::IntrinsicUnsupportedArgType {
                        name: kind.to_string(),
                        span,
                        hint: "This argument must be raw_ptr".to_string(),
                    });
                }
                let key_value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, &key_exp)?
                );
                let val_value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, &val_exp)?
                );
                let number_of_slots_value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, &number_of_slots_exp)?
                );
                let span_md_idx = md_mgr.span_to_md(context, &span);
                let key_var = store_key_in_local_mem(self, context, key_value, span_md_idx)?;
                let b256_ty = Type::get_b256(context);
                let b256_ptr_ty = Type::new_ptr(context, b256_ty);
                // For quad word, the IR instructions take in a pointer rather than a raw u64.
                let val_ptr = self
                    .current_block
                    .append(context)
                    .int_to_ptr(val_value, b256_ptr_ty)
                    .add_metadatum(context, span_md_idx);
                match kind {
                    Intrinsic::StateLoadQuad => {
                        let val = self
                            .current_block
                            .append(context)
                            .state_load_quad_word(val_ptr, key_var, number_of_slots_value)
                            .add_metadatum(context, span_md_idx);
                        Ok(TerminatorValue::new(val, context))
                    }
                    Intrinsic::StateStoreQuad => {
                        let val = self
                            .current_block
                            .append(context)
                            .state_store_quad_word(val_ptr, key_var, number_of_slots_value)
                            .add_metadatum(context, span_md_idx);
                        Ok(TerminatorValue::new(val, context))
                    }
                    _ => unreachable!(),
                }
            }
            Intrinsic::Log => {
                if context.program_kind == Kind::Predicate {
                    return Err(CompileError::DisallowedIntrinsicInPredicate {
                        intrinsic: kind.to_string(),
                        span: span.clone(),
                    });
                }

                // The log value and the log ID are just Value.
                let log_val = return_on_termination_or_extract!(self.compile_expression_to_value(
                    context,
                    md_mgr,
                    &arguments[0]
                )?);
                let logged_type = i
                    .get_logged_type(context.experimental.new_encoding)
                    .expect("Could not return logged type.");
                let log_id = match self.logged_types_map.get(&logged_type) {
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
                        let val = self
                            .current_block
                            .append(context)
                            .log(log_val, log_ty, log_id)
                            .add_metadatum(context, span_md_idx);
                        Ok(TerminatorValue::new(val, context))
                    }
                }
            }
            Intrinsic::Add
            | Intrinsic::Sub
            | Intrinsic::Mul
            | Intrinsic::Div
            | Intrinsic::And
            | Intrinsic::Or
            | Intrinsic::Xor
            | Intrinsic::Mod
            | Intrinsic::Rsh
            | Intrinsic::Lsh => {
                let op = match kind {
                    Intrinsic::Add => BinaryOpKind::Add,
                    Intrinsic::Sub => BinaryOpKind::Sub,
                    Intrinsic::Mul => BinaryOpKind::Mul,
                    Intrinsic::Div => BinaryOpKind::Div,
                    Intrinsic::And => BinaryOpKind::And,
                    Intrinsic::Or => BinaryOpKind::Or,
                    Intrinsic::Xor => BinaryOpKind::Xor,
                    Intrinsic::Mod => BinaryOpKind::Mod,
                    Intrinsic::Rsh => BinaryOpKind::Rsh,
                    Intrinsic::Lsh => BinaryOpKind::Lsh,
                    _ => unreachable!(),
                };
                let lhs = &arguments[0];
                let rhs = &arguments[1];
                let lhs_value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, lhs)?
                );
                let rhs_value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, rhs)?
                );
                let val = self
                    .current_block
                    .append(context)
                    .binary_op(op, lhs_value, rhs_value);
                Ok(TerminatorValue::new(val, context))
            }
            Intrinsic::Revert => {
                let revert_code_val = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, &arguments[0])?
                );

                // The `revert` instruction
                let span_md_idx = md_mgr.span_to_md(context, &span);
                let val = self
                    .current_block
                    .append(context)
                    .revert(revert_code_val)
                    .add_metadatum(context, span_md_idx);
                Ok(TerminatorValue::new(val, context))
            }
            Intrinsic::PtrAdd | Intrinsic::PtrSub => {
                let op = match kind {
                    Intrinsic::PtrAdd => BinaryOpKind::Add,
                    Intrinsic::PtrSub => BinaryOpKind::Sub,
                    _ => unreachable!(),
                };

                let len = type_arguments[0].clone();
                let ir_type = convert_resolved_typeid(
                    engines.te(),
                    engines.de(),
                    context,
                    &len.type_id,
                    &len.span,
                )?;
                let len_value = Constant::get_uint(context, 64, ir_type.size(context).in_bytes());

                let lhs = &arguments[0];
                let count = &arguments[1];
                let lhs_value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, lhs)?
                );
                let count_value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, count)?
                );
                let rhs_value = self.current_block.append(context).binary_op(
                    BinaryOpKind::Mul,
                    len_value,
                    count_value,
                );
                let val = self
                    .current_block
                    .append(context)
                    .binary_op(op, lhs_value, rhs_value);
                Ok(TerminatorValue::new(val, context))
            }
            Intrinsic::Smo => {
                let span_md_idx = md_mgr.span_to_md(context, &span);

                /* First operand: recipient */
                let recipient_value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, &arguments[0])?
                );
                let recipient_md_idx = md_mgr.span_to_md(context, &span);
                let recipient_var =
                    store_key_in_local_mem(self, context, recipient_value, recipient_md_idx)?;

                /* Second operand: message data */
                // Step 1: compile the user data and get its type
                let user_message = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, &arguments[1])?
                );

                let user_message_type = user_message.get_type(context).ok_or_else(|| {
                    CompileError::Internal(
                        "Unable to determine type for message data.",
                        span.clone(),
                    )
                })?;

                // Step 2: build a struct with two fields:
                // - The first field is a `u64` that contains the message ID
                // - The second field contains the actual user data
                let u64_ty = Type::get_uint64(context);
                let field_types = [u64_ty, user_message_type];
                let message_aggregate = Type::new_struct(context, field_types.to_vec());

                // Step 3: construct a local pointer for the message aggregate struct
                let message_aggregate_local_name = self.lexical_map.insert_anon();
                let message_ptr = self
                    .function
                    .new_local_var(
                        context,
                        message_aggregate_local_name,
                        message_aggregate,
                        None,
                        false,
                    )
                    .map_err(|ir_error| {
                        CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                    })?;

                // Step 4: Convert the local variable into a value via `get_local`.
                let message = self
                    .current_block
                    .append(context)
                    .get_local(message_ptr)
                    .add_metadatum(context, span_md_idx);

                // Step 5: Grab the message ID from `messages_types_map` and insert it as the
                // first field of the struct
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
                let gep_val = self
                    .current_block
                    .append(context)
                    .get_elem_ptr_with_idx(message, u64_ty, 0);
                self.current_block
                    .append(context)
                    .store(gep_val, message_id_val)
                    .add_metadatum(context, span_md_idx);

                // Step 6: Insert the user message data as the second field of the struct
                let gep_val = self.current_block.append(context).get_elem_ptr_with_idx(
                    message,
                    user_message_type,
                    1,
                );
                let user_message_size = 8 + user_message_type.size(context).in_bytes();
                self.current_block
                    .append(context)
                    .store(gep_val, user_message)
                    .add_metadatum(context, span_md_idx);

                /* Third operand: the size of the message data */
                let user_message_size_val = Constant::get_uint(context, 64, user_message_size);

                /* Fourth operand: the amount of coins to send */
                let coins = return_on_termination_or_extract!(self.compile_expression_to_value(
                    context,
                    md_mgr,
                    &arguments[2]
                )?);

                let val = self
                    .current_block
                    .append(context)
                    .smo(recipient_var, message, user_message_size_val, coins)
                    .add_metadatum(context, span_md_idx);
                Ok(TerminatorValue::new(val, context))
            }
            Intrinsic::Not => {
                assert!(arguments.len() == 1);

                let op = &arguments[0];
                let value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, op)?
                );

                let val = self
                    .current_block
                    .append(context)
                    .unary_op(UnaryOpKind::Not, value);
                Ok(TerminatorValue::new(val, context))
            }
        }
    }

    fn compile_return(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_expr: &ty::TyExpression,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
        let ret_value = return_on_termination_or_extract!(
            self.compile_expression_to_value(context, md_mgr, ast_expr)?
        );

        ret_value
            .get_type(context)
            .map(|ret_ty| {
                let val = self
                    .current_block
                    .append(context)
                    .ret(ret_value, ret_ty)
                    .add_metadatum(context, span_md_idx);
                TerminatorValue::new(val, context)
            })
            .ok_or_else(|| {
                CompileError::Internal(
                    "Unable to determine type for return expression.",
                    ast_expr.span.clone(),
                )
            })
    }

    fn compile_ref(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_expr: &ty::TyExpression,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
        let value = return_on_termination_or_extract!(
            self.compile_expression_to_ptr(context, md_mgr, ast_expr)?
        );

        // TODO-IG: Do we need to convert to `u64` here? Can we use `Ptr` directly? Investigate.
        let int_ty = Type::get_uint64(context);
        let val = self
            .current_block
            .append(context)
            .ptr_to_int(value, int_ty)
            .add_metadatum(context, span_md_idx);
        Ok(TerminatorValue::new(val, context))
    }

    fn compile_deref(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_expr: &ty::TyExpression,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
        let ref_value =
            return_on_termination_or_extract!(self.compile_expression(context, md_mgr, ast_expr)?);

        let ptr_as_int = if ref_value
            .get_type(context)
            .map_or(false, |ref_value_type| ref_value_type.is_ptr(context))
        {
            // We are dereferencing a reference variable and we got a pointer to it.
            // To get the address the reference is pointing to we need to load the value.
            self.current_block.append(context).load(ref_value)
        } else {
            // The value itself is the address.
            ref_value
        };

        let reference_type = self.engines.te().get_unaliased(ast_expr.return_type);

        let referenced_ast_type = match *reference_type {
            TypeInfo::Ref(ref referenced_type) => Ok(referenced_type.type_id),
            _ => Err(CompileError::Internal(
                "Cannot dereference a non-reference expression.",
                ast_expr.span.clone(),
            )),
        }?;

        let referenced_ir_type = convert_resolved_typeid(
            self.engines.te(),
            self.engines.de(),
            context,
            &referenced_ast_type,
            &ast_expr.span.clone(),
        )?;

        let ptr_type = Type::new_ptr(context, referenced_ir_type);
        let ptr = self
            .current_block
            .append(context)
            .int_to_ptr(ptr_as_int, ptr_type)
            .add_metadatum(context, span_md_idx);

        let referenced_type = self.engines.te().get_unaliased(referenced_ast_type);

        let result = if referenced_type.is_copy_type() || referenced_type.is_reference() {
            // For non aggregates, we need to return the value.
            // This means, loading the value the `ptr` is pointing to.
            self.current_block.append(context).load(ptr)
        } else {
            // For aggregates, we access them via pointer, so we just
            // need to return the `ptr`.
            ptr
        };

        Ok(TerminatorValue::new(result, context))
    }

    fn compile_lazy_op(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_op: &LazyOp,
        ast_lhs: &ty::TyExpression,
        ast_rhs: &ty::TyExpression,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
        let lhs_val = return_on_termination_or_extract!(
            self.compile_expression_to_value(context, md_mgr, ast_lhs)?
        );
        // Short-circuit: if LHS is true for AND we still must eval the RHS block; for OR we can
        // skip the RHS block, and vice-versa.
        let cond_block_end = self.current_block;
        let rhs_block = self.function.create_block(context, None);
        let final_block = self.function.create_block(context, None);

        let merge_val_arg_idx = final_block.new_arg(context, lhs_val.get_type(context).unwrap());

        let cond_builder = cond_block_end.append(context);
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

        self.current_block = rhs_block;
        let rhs_val = self.compile_expression_to_value(context, md_mgr, ast_rhs)?;

        if !rhs_val.is_terminator {
            self.current_block
                .append(context)
                .branch(final_block, vec![rhs_val.value])
                .add_metadatum(context, span_md_idx);
        }

        self.current_block = final_block;
        let val = final_block.get_arg(context, merge_val_arg_idx).unwrap();
        Ok(TerminatorValue::new(val, context))
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_contract_call(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        call_params: &ty::ContractCallParams,
        contract_call_parameters: &IndexMap<String, ty::TyExpression>,
        ast_name: &str,
        ast_args: &[(Ident, ty::TyExpression)],
        ast_return_type: TypeId,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
        // XXX This is very FuelVM specific and needs to be broken out of here and called
        // conditionally based on the target.

        // Compile each user argument
        let mut compiled_args = Vec::<Value>::new();
        for (_, arg) in ast_args.iter() {
            let val = return_on_termination_or_extract!(
                self.compile_expression_to_value(context, md_mgr, arg)?
            );
            compiled_args.push(val)
        }

        let u64_ty = Type::get_uint64(context);

        let user_args_val = match compiled_args.len() {
            0 => Constant::get_uint(context, 64, 0),
            1 => {
                // The single arg doesn't need to be put into a struct.
                let arg0 = compiled_args[0];
                let arg0_type = self.engines.te().get_unaliased(ast_args[0].1.return_type);

                match arg0_type {
                    _ if arg0_type.is_copy_type() => self
                        .current_block
                        .append(context)
                        .bitcast(arg0, u64_ty)
                        .add_metadatum(context, span_md_idx),
                    _ => {
                        // Use a temporary to pass a reference to the arg.
                        let arg0_type = arg0.get_type(context).unwrap();
                        let temp_arg_name = self
                            .lexical_map
                            .insert(format!("{}{}", "arg_for_", ast_name));
                        let temp_var = self
                            .function
                            .new_local_var(context, temp_arg_name, arg0_type, None, false)
                            .map_err(|ir_error| {
                                CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                            })?;

                        let temp_val = self.current_block.append(context).get_local(temp_var);
                        self.current_block.append(context).store(temp_val, arg0);

                        // NOTE: Here we're casting the temp pointer to an integer.
                        self.current_block
                            .append(context)
                            .ptr_to_int(temp_val, u64_ty)
                    }
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
                        false,
                    )
                    .map_err(|ir_error| {
                        CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                    })?;

                // Initialise each of the fields in the user args struct.
                let user_args_struct_val = self
                    .current_block
                    .append(context)
                    .get_local(user_args_struct_var)
                    .add_metadatum(context, span_md_idx);
                compiled_args
                    .into_iter()
                    .zip(field_types)
                    .enumerate()
                    .for_each(|(insert_idx, (field_val, field_type))| {
                        let gep_val = self
                            .current_block
                            .append(context)
                            .get_elem_ptr_with_idx(
                                user_args_struct_val,
                                field_type,
                                insert_idx as u64,
                            )
                            .add_metadatum(context, span_md_idx);

                        self.current_block
                            .append(context)
                            .store(gep_val, field_val)
                            .add_metadatum(context, span_md_idx);
                    });

                // NOTE: Here we're casting the args struct pointer to an integer.
                self.current_block
                    .append(context)
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
                false,
            )
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        let ra_struct_ptr_val = self
            .current_block
            .append(context)
            .get_local(ra_struct_var)
            .add_metadatum(context, span_md_idx);

        // Insert the contract address
        let addr = return_on_termination_or_extract!(self.compile_expression_to_value(
            context,
            md_mgr,
            &call_params.contract_address
        )?);
        let gep_val =
            self.current_block
                .append(context)
                .get_elem_ptr_with_idx(ra_struct_ptr_val, b256_ty, 0);
        self.current_block
            .append(context)
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
                .append(context)
                .get_elem_ptr_with_idx(ra_struct_ptr_val, u64_ty, 1);
        self.current_block
            .append(context)
            .store(gep_val, sel_val)
            .add_metadatum(context, span_md_idx);

        // Insert the user args value.
        let gep_val =
            self.current_block
                .append(context)
                .get_elem_ptr_with_idx(ra_struct_ptr_val, u64_ty, 2);
        self.current_block
            .append(context)
            .store(gep_val, user_args_val)
            .add_metadatum(context, span_md_idx);

        // Compile all other call parameters
        let coins = match contract_call_parameters
            .get(&constants::CONTRACT_CALL_COINS_PARAMETER_NAME.to_string())
        {
            Some(coins_expr) => {
                return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, coins_expr)?
                )
            }
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
                return_on_termination_or_extract!(self.compile_expression_to_ptr(
                    context,
                    md_mgr,
                    asset_id_expr
                )?)
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
                    .new_local_var(context, tmp_asset_id_name, b256_ty, None, false)
                    .map_err(|ir_error| {
                        CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                    })?;
                let tmp_val = self.current_block.append(context).get_local(tmp_var);
                self.current_block
                    .append(context)
                    .store(tmp_val, asset_id_val);
                tmp_val
            }
        };

        let gas = match contract_call_parameters
            .get(&constants::CONTRACT_CALL_GAS_PARAMETER_NAME.to_string())
        {
            Some(gas_expr) => {
                return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, gas_expr)?
                )
            }
            None => self
                .current_block
                .append(context)
                .read_register(sway_ir::Register::Cgas)
                .add_metadatum(context, span_md_idx),
        };

        // Convert the return type.  If it's a reference type then make it a pointer.
        let return_type = convert_resolved_typeid_no_span(
            self.engines.te(),
            self.engines.de(),
            context,
            &ast_return_type,
        )?;
        let ret_is_copy_type = self
            .engines
            .te()
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
            .append(context)
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
        let res = if ret_is_copy_type {
            call_val
        } else {
            self.current_block.append(context).load(call_val)
        };
        Ok(TerminatorValue::new(res, context))
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_fn_call(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_args: &[(Ident, ty::TyExpression)],
        callee: &ty::TyFunctionDecl,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
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
                    self.engines,
                    context,
                    md_mgr,
                    self.module,
                    &callee_fn_decl,
                    &self.logged_types_map,
                    &self.messages_types_map,
                    is_entry,
                    None,
                )
                .map_err(|mut x| x.pop().unwrap())?
                .unwrap();
                self.recreated_fns.insert(fn_key, new_func);
                new_func
            }
        };

        // Now actually call the new function.
        let mut args = Vec::with_capacity(ast_args.len());
        for ((_, expr), param) in ast_args.iter().zip(callee.parameters.iter()) {
            self.current_fn_param = Some(param.clone());
            let arg =
                return_on_termination_or_extract!(if param.is_reference && param.is_mutable {
                    self.compile_expression_to_ptr(context, md_mgr, expr)
                } else {
                    self.compile_expression_to_value(context, md_mgr, expr)
                }?);
            self.current_fn_param = None;
            args.push(arg);
        }

        let val = self
            .current_block
            .append(context)
            .call(new_callee, &args)
            .add_metadatum(context, span_md_idx);

        Ok(TerminatorValue::new(val, context))
    }

    fn compile_if(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_condition: &ty::TyExpression,
        ast_then: &ty::TyExpression,
        ast_else: Option<&ty::TyExpression>,
        return_type: TypeId,
    ) -> Result<TerminatorValue, CompileError> {
        // Compile the condition expression in the entry block.  Then save the current block so we
        // can jump to the true and false blocks after we've created them.
        let cond_span_md_idx = md_mgr.span_to_md(context, &ast_condition.span);
        let cond_value = return_on_termination_or_extract!(self.compile_expression_to_value(
            context,
            md_mgr,
            ast_condition
        )?);
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
        let true_value = self.compile_expression_to_value(context, md_mgr, ast_then)?;
        let true_block_end = self.current_block;

        let false_block_begin = self.function.create_block(context, None);
        self.current_block = false_block_begin;
        let false_value = match ast_else {
            None => TerminatorValue::new(Constant::get_unit(context), context),
            Some(expr) => self.compile_expression_to_value(context, md_mgr, expr)?,
        };
        let false_block_end = self.current_block;

        cond_block
            .append(context)
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
        // Rely on the type of the ast node when creating that argument.
        let val = if true_value.is_terminator && false_value.is_terminator {
            // Corner case: If both branches diverge, then the return type is 'Unknown', which we can't
            // compile. We also cannot add a block parameter of 'Unit' type or similar, since the
            // parameter may be used by dead code after the 'if' causing a potentially illegally typed
            // program. In this case we do not add a block parameter. Instead we add a diverging dummy
            // value to the merge branch to signal that the expression diverges.
            merge_block.append(context).branch(true_block_begin, vec![])
        } else {
            let return_type = convert_resolved_typeid_no_span(
                self.engines.te(),
                self.engines.de(),
                context,
                &return_type,
            )
            .unwrap_or_else(|_| Type::get_unit(context));
            let merge_val_arg_idx = merge_block.new_arg(context, return_type);
            if !true_value.is_terminator {
                true_block_end
                    .append(context)
                    .branch(merge_block, vec![true_value.value]);
            }
            if !false_value.is_terminator {
                false_block_end
                    .append(context)
                    .branch(merge_block, vec![false_value.value]);
            }
            self.current_block = merge_block;
            merge_block.get_arg(context, merge_val_arg_idx).unwrap()
        };
        Ok(TerminatorValue::new(val, context))
    }

    fn compile_unsafe_downcast(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        exp: &ty::TyExpression,
        variant: &ty::TyEnumVariant,
    ) -> Result<TerminatorValue, CompileError> {
        // Retrieve the type info for the enum.
        let enum_type = match convert_resolved_typeid(
            self.engines.te(),
            self.engines.de(),
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
        let compiled_value = return_on_termination_or_extract!(
            self.compile_expression_to_ptr(context, md_mgr, exp)?
        );

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
        let val = self.current_block.append(context).get_elem_ptr_with_idcs(
            compiled_value,
            variant_type,
            &[1, variant.tag as u64],
        );
        Ok(TerminatorValue::new(val, context))
    }

    fn compile_enum_tag(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        exp: Box<ty::TyExpression>,
    ) -> Result<TerminatorValue, CompileError> {
        let tag_span_md_idx = md_mgr.span_to_md(context, &exp.span);
        let struct_val = return_on_termination_or_extract!(
            self.compile_expression_to_ptr(context, md_mgr, &exp)?
        );

        let u64_ty = Type::get_uint64(context);
        let val = self
            .current_block
            .append(context)
            .get_elem_ptr_with_idx(struct_val, u64_ty, 0)
            .add_metadatum(context, tag_span_md_idx);
        Ok(TerminatorValue::new(val, context))
    }

    fn compile_while_loop(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        body: &ty::TyCodeBlock,
        condition: &ty::TyExpression,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
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
        self.current_block
            .append(context)
            .branch(cond_block, vec![]);

        // Compile the condition
        self.current_block = cond_block;
        let cond_value = return_on_termination_or_extract!(
            self.compile_expression_to_value(context, md_mgr, condition)?
        );

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
        let body_block_val = self
            .compile_code_block(context, md_mgr, body)
            .map_err(|mut x| x.pop().unwrap())?;
        if !body_block_val.is_terminator {
            self.current_block
                .append(context)
                .branch(cond_block, vec![]);
        }

        // Restore the blocks to jump to now that we're done with the current loop
        self.block_to_break_to = prev_block_to_break_to;
        self.block_to_continue_to = prev_block_to_continue_to;

        // Create the final block now we're finished with the body.
        let final_block = self
            .function
            .create_block(context, Some("end_while".into()));

        // Add an unconditional jump from the break block to the final block.
        break_block.append(context).branch(final_block, vec![]);

        // Add conditional jumps from the conditional block to the body block or the final block.
        cond_block.append(context).conditional_branch(
            cond_value,
            body_block,
            final_block,
            vec![],
            vec![],
        );

        self.current_block = final_block;
        let val = Constant::get_unit(context).add_metadatum(context, span_md_idx);
        Ok(TerminatorValue::new(val, context))
    }

    pub(crate) fn get_function_var(&self, context: &mut Context, name: &str) -> Option<LocalVar> {
        self.lexical_map
            .get(name)
            .and_then(|local_name| self.function.get_local_var(context, local_name))
    }

    pub(crate) fn get_function_arg(&self, context: &mut Context, name: &str) -> Option<Value> {
        self.function.get_arg(context, name)
    }

    fn compile_const_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        const_decl: &TyConstantDecl,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
        let result = self
            .compile_var_expr(
                context,
                &Some(const_decl.call_path.clone()),
                const_decl.name(),
                span_md_idx,
            )
            .or(self.compile_const_decl(context, md_mgr, const_decl, span_md_idx, true))?;

        // String slices are not allowed in constants
        if let Some(TypeContent::StringSlice) = result
            .value
            .get_type(context)
            .map(|t| t.get_content(context))
        {
            return Err(CompileError::TypeNotAllowed {
                reason: sway_error::error::TypeNotAllowedReason::StringSliceInConst,
                span: const_decl.span.clone(),
            });
        }

        Ok(result)
    }

    fn compile_var_expr(
        &mut self,
        context: &mut Context,
        call_path: &Option<CallPath>,
        name: &Ident,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
        let call_path = call_path
            .clone()
            .unwrap_or_else(|| CallPath::from(name.clone()));

        // We need to check the symbol map first, in case locals are shadowing the args, other
        // locals or even constants.
        if let Some(var) = self.get_function_var(context, name.as_str()) {
            let val = self
                .current_block
                .append(context)
                .get_local(var)
                .add_metadatum(context, span_md_idx);
            Ok(TerminatorValue::new(val, context))
        } else if let Some(val) = self.function.get_arg(context, name.as_str()) {
            Ok(TerminatorValue::new(val, context))
        } else if let Some(const_val) = self
            .module
            .get_global_constant(context, &call_path.as_vec_string())
        {
            Ok(TerminatorValue::new(const_val, context))
        } else if let Some(config_val) = self
            .module
            .get_global_configurable(context, &call_path.as_vec_string())
        {
            Ok(TerminatorValue::new(config_val, context))
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
    ) -> Result<Option<TerminatorValue>, CompileError> {
        let ty::TyVariableDecl {
            name,
            body,
            mutability,
            ..
        } = ast_var_decl;
        // Nothing to do for an abi cast declarations. The address specified in them is already
        // provided in each contract call node in the AST.
        if matches!(
            &&*self.engines.te().get_unaliased(body.return_type),
            TypeInfo::ContractCaller { .. }
        ) {
            return Ok(None);
        }

        // We must compile the RHS before checking for shadowing, as it will still be in the
        // previous scope.
        let init_val = self.compile_expression_to_value(context, md_mgr, body)?;
        if init_val.is_terminator {
            return Ok(Some(init_val));
        }

        let return_type = convert_resolved_typeid(
            self.engines.te(),
            self.engines.de(),
            context,
            &body.return_type,
            &body.span,
        )?;

        let mutable = matches!(mutability, ty::VariableMutability::Mutable);
        let local_name = self.lexical_map.insert(name.as_str().to_owned());
        let local_var = self
            .function
            .new_local_var(context, local_name.clone(), return_type, None, mutable)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        // We can have empty aggregates, especially arrays, which shouldn't be initialized, but
        // otherwise use a store.
        let var_ty = local_var.get_type(context);
        if var_ty.size(context).in_bytes() > 0 {
            let local_ptr = self
                .current_block
                .append(context)
                .get_local(local_var)
                .add_metadatum(context, span_md_idx);
            self.current_block
                .append(context)
                .store(local_ptr, init_val.value)
                .add_metadatum(context, span_md_idx);
        }
        Ok(None)
    }

    fn compile_const_decl(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_const_decl: &ty::TyConstantDecl,
        span_md_idx: Option<MetadataIndex>,
        is_const_expression: bool,
    ) -> Result<TerminatorValue, CompileError> {
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
                self.engines,
                context,
                md_mgr,
                self.module,
                None,
                Some(self),
                call_path,
                value,
                *is_configurable,
            )?;

            if is_const_expression {
                Ok(TerminatorValue::new(const_expr_val, context))
            } else {
                let local_name = self
                    .lexical_map
                    .insert(call_path.suffix.as_str().to_owned());

                let return_type = convert_resolved_typeid(
                    self.engines.te(),
                    self.engines.de(),
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
                    .new_local_var(context, local_name, return_type, None, false)
                    .map_err(|ir_error| {
                        CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                    })?;

                // We can have empty aggregates, especially arrays, which shouldn't be initialised, but
                // otherwise use a store.
                let var_ty = local_var.get_type(context);
                Ok(if var_ty.size(context).in_bytes() > 0 {
                    let local_val = self
                        .current_block
                        .append(context)
                        .get_local(local_var)
                        .add_metadatum(context, span_md_idx);
                    let val = self
                        .current_block
                        .append(context)
                        .store(local_val, const_expr_val)
                        .add_metadatum(context, span_md_idx);
                    TerminatorValue::new(val, context)
                } else {
                    TerminatorValue::new(const_expr_val, context)
                })
            }
        } else {
            unreachable!("cannot compile const declaration without an expression")
        }
    }

    fn compile_reassignment(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_reassignment: &ty::TyReassignment,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
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
                    .append(context)
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

        let reassign_val = return_on_termination_or_extract!(self.compile_expression_to_value(
            context,
            md_mgr,
            &ast_reassignment.rhs
        )?);

        let lhs_ptr = if ast_reassignment.lhs_indices.is_empty() {
            // A non-aggregate; use a direct `store`.
            lhs_val
        } else {
            // Create a GEP by following the chain of LHS indices.  We use a scan which is
            // essentially a map with context, which is the parent type id for the current field.
            let mut gep_indices = Vec::<Value>::new();
            let mut cur_type_id = ast_reassignment.lhs_type;
            for idx_kind in ast_reassignment.lhs_indices.iter() {
                let cur_type_info_arc = self.engines.te().get_unaliased(cur_type_id);
                let cur_type_info = &*cur_type_info_arc;
                match (idx_kind, cur_type_info) {
                    (
                        ProjectionKind::StructField { name: idx_name },
                        TypeInfo::Struct(decl_ref),
                    ) => {
                        let struct_decl = self.engines.de().get_struct(decl_ref);

                        match struct_decl.get_field_index_and_type(idx_name) {
                            None => {
                                return Err(CompileError::InternalOwned(
                                    format!(
                                        "Unknown field name '{idx_name}' for struct '{}' \
                                         in reassignment.",
                                        struct_decl.call_path.suffix.as_str(),
                                    ),
                                    ast_reassignment.lhs_base_name.span(),
                                ))
                            }
                            Some((field_idx, field_type_id)) => {
                                cur_type_id = field_type_id;
                                gep_indices.push(Constant::get_uint(context, 64, field_idx));
                            }
                        }
                    }
                    (ProjectionKind::TupleField { index, .. }, TypeInfo::Tuple(field_tys)) => {
                        cur_type_id = field_tys[*index].type_id;
                        gep_indices.push(Constant::get_uint(context, 64, *index as u64));
                    }
                    (ProjectionKind::ArrayIndex { index, .. }, TypeInfo::Array(elem_ty, _)) => {
                        cur_type_id = elem_ty.type_id;
                        let val = return_on_termination_or_extract!(
                            self.compile_expression_to_value(context, md_mgr, index)?
                        );
                        gep_indices.push(val);
                    }
                    _ => {
                        return Err(CompileError::Internal(
                            "Unknown field in reassignment.",
                            idx_kind.span(),
                        ))
                    }
                }
            }

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
                .append(context)
                .get_elem_ptr(lhs_val, field_type, gep_indices)
                .add_metadatum(context, span_md_idx)
        };

        self.current_block
            .append(context)
            .store(lhs_ptr, reassign_val)
            .add_metadatum(context, span_md_idx);

        let val = Constant::get_unit(context).add_metadatum(context, span_md_idx);
        Ok(TerminatorValue::new(val, context))
    }

    fn compile_array_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        elem_type: &TypeId,
        contents: &[ty::TyExpression],
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
        // If the first element diverges, then the element type has not been determined,
        // so we can't use the normal compilation scheme. Instead just generate code for
        // the first element and return.
        let first_elem_value = if !contents.is_empty() {
            let first_elem_value = return_on_termination_or_extract!(
                self.compile_expression_to_value(context, md_mgr, &contents[0])?
            );
            Some(first_elem_value)
        } else {
            None
        };

        let elem_type = convert_resolved_typeid_no_span(
            self.engines.te(),
            self.engines.de(),
            context,
            elem_type,
        )?;

        let array_type = Type::new_array(context, elem_type, contents.len() as u64);

        let temp_name = self.lexical_map.insert_anon();
        let array_var = self
            .function
            .new_local_var(context, temp_name, array_type, None, false)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;

        let array_value = self
            .current_block
            .append(context)
            .get_local(array_var)
            .add_metadatum(context, span_md_idx);

        // Nothing more to do if the array is empty
        if contents.is_empty() {
            return Ok(TerminatorValue::new(array_value, context));
        }

        // The array is not empty, so it's safe to unwrap the first element
        let first_elem_value = first_elem_value.unwrap();

        // If all elements are the same constant, then we can initialize the array
        // in a loop, reducing code size. But to check for that we've to compile
        // the expressions first, to compare. If it turns out that they're not all
        // constants, then we end up with all expressions compiled first, and then
        // all of them stored to the array. This could potentially be bad for register
        // pressure. So we do this in steps, at the cost of some compile time.
        let all_consts = contents
            .iter()
            .all(|elm| matches!(elm.expression, TyExpressionVariant::Literal(..)));

        // We only do the optimization for suffiently large arrays, so that
        // overhead due to the loop doesn't make it worse than the unrolled version.
        if all_consts && contents.len() > 5 {
            // We can compile all elements ahead of time without affecting register pressure.
            let compiled_elems = contents
                .iter()
                .enumerate()
                .map(|(idx, e)| {
                    if idx == 0 {
                        // The first element has already been compiled
                        Ok::<Value, CompileError>(first_elem_value)
                    } else {
                        let val = self.compile_expression_to_value(context, md_mgr, e)?;
                        Ok(val.value)
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut compiled_elems_iter = compiled_elems.iter();
            let first = compiled_elems_iter.next();
            let const_initialiser_opt = first.filter(|c| {
                compiled_elems_iter.all(|elem| {
                    elem.get_constant(context)
                        .expect("Constant expression must evaluate to a constant IR value")
                        .eq(
                            context,
                            c.get_constant(context)
                                .expect("Constant expression must evaluate to a constant IR value"),
                        )
                })
            });
            if let Some(const_initializer) = const_initialiser_opt {
                // Create a loop to insert const_initializer to all array elements.
                let loop_block = self
                    .function
                    .create_block(context, Some("array_init_loop".into()));
                // The loop begins with 0.
                let zero = Constant::new_uint(context, 64, 0);
                let zero = Value::new_constant(context, zero);
                // Branch to the loop block, passing the initial iteration value.
                self.current_block
                    .append(context)
                    .branch(loop_block, vec![zero]);
                // Add a block argument (for the IV) to the loop block.
                let index_var_index = loop_block.new_arg(context, Type::get_uint64(context));
                let index = loop_block.get_arg(context, index_var_index).unwrap();
                // Create an exit block.
                let exit_block = self
                    .function
                    .create_block(context, Some("array_init_exit".into()));
                // Start building the loop block.
                self.current_block = loop_block;
                let gep_val = self.current_block.append(context).get_elem_ptr(
                    array_value,
                    elem_type,
                    vec![index],
                );
                self.current_block
                    .append(context)
                    .store(gep_val, *const_initializer)
                    .add_metadatum(context, span_md_idx);
                // Increment index by one.
                let one = Constant::new_uint(context, 64, 1);
                let one = Value::new_constant(context, one);
                let index_inc =
                    self.current_block
                        .append(context)
                        .binary_op(BinaryOpKind::Add, index, one);
                // continue = index_inc < contents.len()
                let len = Constant::new_uint(context, 64, contents.len() as u64);
                let len = Value::new_constant(context, len);
                let r#continue =
                    self.current_block
                        .append(context)
                        .cmp(Predicate::LessThan, index_inc, len);
                // if continue then loop_block else exit_block.
                self.current_block.append(context).conditional_branch(
                    r#continue,
                    loop_block,
                    exit_block,
                    vec![index_inc],
                    vec![],
                );
                // Continue compilation in the exit block.
                self.current_block = exit_block;
            } else {
                // Insert each element separately.
                for (idx, elem_value) in compiled_elems.iter().enumerate() {
                    let gep_val = self.current_block.append(context).get_elem_ptr_with_idx(
                        array_value,
                        elem_type,
                        idx as u64,
                    );
                    self.current_block
                        .append(context)
                        .store(gep_val, *elem_value)
                        .add_metadatum(context, span_md_idx);
                }
            }
            return Ok(TerminatorValue::new(array_value, context));
        }

        // Compile each element and insert it immediately.
        for (idx, elem_expr) in contents.iter().enumerate() {
            let elem_value = if idx == 0 {
                first_elem_value
            } else {
                return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, elem_expr)?
                )
            };
            let gep_val = self.current_block.append(context).get_elem_ptr_with_idx(
                array_value,
                elem_type,
                idx as u64,
            );
            self.current_block
                .append(context)
                .store(gep_val, elem_value)
                .add_metadatum(context, span_md_idx);
        }
        Ok(TerminatorValue::new(array_value, context))
    }

    fn compile_array_index(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        array_expr: &ty::TyExpression,
        index_expr: &ty::TyExpression,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
        let array_val = return_on_termination_or_extract!(
            self.compile_expression_to_ptr(context, md_mgr, array_expr)?
        );

        // Get the array type and confirm it's an array.
        let array_type = array_val
            .get_type(context)
            .and_then(|ty| ty.get_pointee_type(context))
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
            self.engines,
            context,
            md_mgr,
            self.module,
            None,
            Some(self),
            index_expr,
            false,
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

        let index_val = return_on_termination_or_extract!(
            self.compile_expression_to_value(context, md_mgr, index_expr)?
        );

        let elem_type = array_type.get_array_elem_type(context).ok_or_else(|| {
            CompileError::Internal(
                "Array type has alread confirmed to be an array.  Getting elem type can't fail.",
                array_expr.span.clone(),
            )
        })?;

        let val = self
            .current_block
            .append(context)
            .get_elem_ptr(array_val, elem_type, vec![index_val])
            .add_metadatum(context, span_md_idx);
        Ok(TerminatorValue::new(val, context))
    }

    fn compile_struct_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        fields: &[ty::TyStructExpressionField],
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
        // NOTE: This is a struct instantiation with initialisers for each field of a named struct.
        // We don't know the actual type of the struct, but the AST guarantees that the fields are
        // in the declared order (regardless of how they are initialised in source) so we can
        // create a struct with the field types.

        // Compile each of the values for field initialisers, calculate their indices and also
        // gather their types with which to make an aggregate.

        let mut insert_values = Vec::with_capacity(fields.len());
        let mut field_types = Vec::with_capacity(fields.len());
        for struct_field in fields.iter() {
            let insert_val = return_on_termination_or_extract!(self.compile_expression_to_value(
                context,
                md_mgr,
                &struct_field.value
            )?);
            insert_values.push(insert_val);

            let field_type = convert_resolved_typeid_no_span(
                self.engines.te(),
                self.engines.de(),
                context,
                &struct_field.value.return_type,
            )?;
            field_types.push(field_type);
        }

        // Create the struct.
        let struct_type = Type::new_struct(context, field_types.clone());
        let temp_name = self.lexical_map.insert_anon();
        let struct_var = self
            .function
            .new_local_var(context, temp_name, struct_type, None, false)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;
        let struct_val = self
            .current_block
            .append(context)
            .get_local(struct_var)
            .add_metadatum(context, span_md_idx);

        // Fill it in.
        insert_values
            .into_iter()
            .zip(field_types)
            .enumerate()
            .for_each(|(insert_idx, (insert_val, field_type))| {
                let gep_val = self.current_block.append(context).get_elem_ptr_with_idx(
                    struct_val,
                    field_type,
                    insert_idx as u64,
                );

                self.current_block
                    .append(context)
                    .store(gep_val, insert_val)
                    .add_metadatum(context, span_md_idx);
            });

        // Return the pointer.
        Ok(TerminatorValue::new(struct_val, context))
    }

    fn compile_struct_field_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        ast_struct_expr: &ty::TyExpression,
        struct_type_id: TypeId,
        ast_field: &ty::TyStructField,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
        let struct_val = return_on_termination_or_extract!(self.compile_expression_to_ptr(
            context,
            md_mgr,
            ast_struct_expr
        )?);

        // Get the struct type info, with field names.
        let decl = self.engines.te().get_unaliased(struct_type_id);
        let TypeInfo::Struct(decl_ref) = &*decl else {
            return Err(CompileError::Internal(
                "Unknown struct in field expression.",
                ast_field.span.clone(),
            ));
        };

        let struct_decl = self.engines.de().get_struct(decl_ref);

        let (field_idx, field_type_id) = struct_decl
            .get_field_index_and_type(&ast_field.name)
            .ok_or_else(|| {
                CompileError::InternalOwned(
                    format!(
                        "Unknown field name '{}' for struct '{}' in field expression.",
                        struct_decl.call_path.suffix.as_str(),
                        ast_field.name
                    ),
                    ast_field.span.clone(),
                )
            })?;

        let field_type = convert_resolved_typeid(
            self.engines.te(),
            self.engines.de(),
            context,
            &field_type_id,
            &ast_field.span,
        )?;

        let val = self
            .current_block
            .append(context)
            .get_elem_ptr_with_idx(struct_val, field_type, field_idx)
            .add_metadatum(context, span_md_idx);
        Ok(TerminatorValue::new(val, context))
    }

    fn compile_enum_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        enum_decl: &ty::TyEnumDecl,
        tag: usize,
        contents: Option<&ty::TyExpression>,
    ) -> Result<TerminatorValue, CompileError> {
        // XXX The enum instantiation AST node includes the full declaration.  If the enum was
        // declared in a different module then it seems for now there's no easy way to pre-analyse
        // it and add its type/aggregate to the context.  We can re-use them here if we recognise
        // the name, and if not add a new aggregate... OTOH the naming seems a little fragile and
        // we could potentially use the wrong aggregate with the same name, different module...
        // dunno.
        let span_md_idx = md_mgr.span_to_md(context, &enum_decl.span);
        let enum_type = create_tagged_union_type(
            self.engines.te(),
            self.engines.de(),
            context,
            &enum_decl.variants,
        )?;
        let tag_value =
            Constant::get_uint(context, 64, tag as u64).add_metadatum(context, span_md_idx);

        // Start with a temporary local struct and insert the tag.
        let temp_name = self.lexical_map.insert_anon();
        let enum_var = self
            .function
            .new_local_var(context, temp_name, enum_type, None, false)
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;
        let enum_ptr = self
            .current_block
            .append(context)
            .get_local(enum_var)
            .add_metadatum(context, span_md_idx);
        let u64_ty = Type::get_uint64(context);
        let tag_gep_val = self
            .current_block
            .append(context)
            .get_elem_ptr_with_idx(enum_ptr, u64_ty, 0)
            .add_metadatum(context, span_md_idx);
        self.current_block
            .append(context)
            .store(tag_gep_val, tag_value)
            .add_metadatum(context, span_md_idx);

        // If the struct representing the enum has only one field, then that field is the tag and
        // all the variants must have unit types, hence the absence of the union. Therefore, there
        // is no need for another `store` instruction here.
        let field_tys = enum_type.get_field_types(context);
        if field_tys.len() != 1 && contents.is_some() {
            // Insert the value too.
            // Only store if the value does not diverge.
            let contents_value = return_on_termination_or_extract!(
                self.compile_expression_to_value(context, md_mgr, contents.unwrap())?
            );
            let contents_type = contents_value.get_type(context).ok_or_else(|| {
                CompileError::Internal(
                    "Unable to get type for enum contents.",
                    enum_decl.span.clone(),
                )
            })?;
            let gep_val = self
                .current_block
                .append(context)
                .get_elem_ptr_with_idcs(enum_ptr, contents_type, &[1, tag as u64])
                .add_metadatum(context, span_md_idx);
            self.current_block
                .append(context)
                .store(gep_val, contents_value)
                .add_metadatum(context, span_md_idx);
        }

        // Return the pointer.
        Ok(TerminatorValue::new(enum_ptr, context))
    }

    fn compile_tuple_expr(
        &mut self,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        fields: &[ty::TyExpression],
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
        if fields.is_empty() {
            // This is a Unit.  We're still debating whether Unit should just be an empty tuple in
            // the IR or not... it is a special case for now.
            let val = Constant::get_unit(context).add_metadatum(context, span_md_idx);
            Ok(TerminatorValue::new(val, context))
        } else {
            let mut init_values = Vec::with_capacity(fields.len());
            let mut init_types = Vec::with_capacity(fields.len());
            for field_expr in fields {
                let init_value = return_on_termination_or_extract!(
                    self.compile_expression_to_value(context, md_mgr, field_expr)?
                );
                let init_type = convert_resolved_typeid_no_span(
                    self.engines.te(),
                    self.engines.de(),
                    context,
                    &field_expr.return_type,
                )?;
                init_values.push(init_value);
                init_types.push(init_type);
            }

            let tuple_type = Type::new_struct(context, init_types.clone());
            let temp_name = self.lexical_map.insert_anon();
            let tuple_var = self
                .function
                .new_local_var(context, temp_name, tuple_type, None, false)
                .map_err(|ir_error| {
                    CompileError::InternalOwned(ir_error.to_string(), Span::dummy())
                })?;
            let tuple_val = self
                .current_block
                .append(context)
                .get_local(tuple_var)
                .add_metadatum(context, span_md_idx);

            init_values
                .into_iter()
                .zip(init_types)
                .enumerate()
                .for_each(|(insert_idx, (field_val, field_type))| {
                    let gep_val = self
                        .current_block
                        .append(context)
                        .get_elem_ptr_with_idx(tuple_val, field_type, insert_idx as u64)
                        .add_metadatum(context, span_md_idx);
                    self.current_block
                        .append(context)
                        .store(gep_val, field_val)
                        .add_metadatum(context, span_md_idx);
                });

            Ok(TerminatorValue::new(tuple_val, context))
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
    ) -> Result<TerminatorValue, CompileError> {
        let tuple_value = return_on_termination_or_extract!(
            self.compile_expression_to_ptr(context, md_mgr, tuple)?
        );
        let tuple_type = convert_resolved_typeid(
            self.engines.te(),
            self.engines.de(),
            context,
            &tuple_type,
            &span,
        )?;

        let val = tuple_type
            .get_field_type(context, idx as u64)
            .map(|field_type| {
                let span_md_idx = md_mgr.span_to_md(context, &span);
                self.current_block
                    .append(context)
                    .get_elem_ptr_with_idx(tuple_value, field_type, idx as u64)
                    .add_metadatum(context, span_md_idx)
            })
            .ok_or_else(|| {
                CompileError::Internal(
                    "Invalid (non-aggregate?) tuple type for TupleElemAccess.",
                    span,
                )
            })?;
        Ok(TerminatorValue::new(val, context))
    }

    fn compile_storage_access(
        &mut self,
        context: &mut Context,
        fields: &[ty::TyStorageAccessDescriptor],
        ix: &StateIndex,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
        // Get the list of indices used to access the storage field. This will be empty
        // if the storage field type is not a struct.
        // FIXME: shouldn't have to extract the first field like this.
        let base_type = fields[0].type_id;
        let field_idcs = get_indices_for_struct_access(
            self.engines.te(),
            self.engines.de(),
            base_type,
            &fields[1..],
        )?;

        // Get the IR type of the storage variable
        let base_type = convert_resolved_typeid_no_span(
            self.engines.te(),
            self.engines.de(),
            context,
            &base_type,
        )?;

        // Do the actual work. This is a recursive function because we want to drill down
        // to load each primitive type in the storage field in its own storage slot.
        self.compile_storage_read(context, ix, &field_idcs, &base_type, span_md_idx)
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
    ) -> Result<TerminatorValue, CompileError> {
        let mut compiled_registers = Vec::<AsmArg>::new();
        for reg in registers.iter() {
            let (init, name) = match reg {
                ty::TyAsmRegisterDeclaration {
                    initializer, name, ..
                } if initializer.is_none() => (None, name),
                ty::TyAsmRegisterDeclaration {
                    initializer, name, ..
                } => {
                    // Take the optional initialiser, map it to an Option<Result<TerminatorValue>>,
                    // transpose that to Result<Option<TerminatorValue>> and map that to an AsmArg.
                    //
                    // Here we need to compile based on the Sway 'copy-type' vs 'ref-type' since
                    // ASM args aren't explicitly typed, and if we send in a temporary it might
                    // be mutated and then discarded.  It *must* be a ptr to *the* ref-type value,
                    // *or* the value of the copy-type value.
                    let init_expr = initializer.as_ref().unwrap();
                    // I'm not sure if a register declaration can diverge, but check just to be safe
                    let initializer_val = return_on_termination_or_extract!(
                        self.compile_expression(context, md_mgr, init_expr)?
                    );
                    let init_type = self.engines.te().get_unaliased(init_expr.return_type);
                    if initializer_val
                        .get_type(context)
                        .map_or(false, |ty| ty.is_ptr(context))
                        && (init_type.is_copy_type() || init_type.is_reference())
                    {
                        // It's a pointer to a copy type, or a reference behind a pointer. We need to dereference it.
                        // We can get a reference behind a pointer if a reference variable is passed to the ASM block.
                        // By "reference" we mean th `u64` value that represents the memory address of the referenced
                        // value.
                        (
                            Some(self.current_block.append(context).load(initializer_val)),
                            name,
                        )
                    } else {
                        // If we have a direct value (not behind a pointer), we just passe it as the initial value.
                        // Note that if the `init_val` is a reference (`u64` representing the memory address) it
                        // behaves the same as any other value, we just passe it as the initial value to the register.
                        (Some(initializer_val), name)
                    }
                }
            };
            compiled_registers.push(AsmArg {
                name: name.clone(),
                initializer: init,
            });
        }

        let body = body
            .iter()
            .map(
                |AsmOp {
                     op_name,
                     op_args,
                     immediate,
                     span,
                 }| AsmInstruction {
                    op_name: op_name.clone(),
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
            self.engines.te(),
            self.engines.de(),
            context,
            &return_type,
        )?;
        let val = self
            .current_block
            .append(context)
            .asm_block(compiled_registers, body, return_type, returns)
            .add_metadatum(context, whole_block_span_md_idx);
        Ok(TerminatorValue::new(val, context))
    }

    fn compile_storage_read(
        &mut self,
        context: &mut Context,
        ix: &StateIndex,
        indices: &[u64],
        base_type: &Type,
        span_md_idx: Option<MetadataIndex>,
    ) -> Result<TerminatorValue, CompileError> {
        // Get the actual storage key as a `Bytes32` as well as the offset, in words,
        // within the slot. The offset depends on what field of the top level storage
        // variable is being accessed.
        let (storage_key, offset_within_slot) = {
            let offset_in_words = match base_type.get_indexed_offset(context, indices) {
                Some(offset_in_bytes) => {
                    // TODO-MEMLAY: Warning! Here we make an assumption about the memory layout of structs.
                    //       The memory layout of structs can be changed in the future.
                    //       We will not refactor the Storage API at the moment to remove this
                    //       assumption. It is a questionable effort because we anyhow
                    //       want to improve and refactor Storage API in the future.
                    assert!(
                        offset_in_bytes % 8 == 0,
                        "Expected struct fields to be aligned to word boundary. The field offset in bytes was {}.",
                        offset_in_bytes
                    );
                    offset_in_bytes / 8
                }
                None => {
                    return Err(CompileError::Internal(
                        "Cannot get the offset within the slot while compiling storage read.",
                        Span::dummy(),
                    ))
                }
            };
            let offset_in_slots = offset_in_words / 4;
            let offset_remaining = offset_in_words % 4;

            // The storage key we need is the storage key of the top level storage variable
            // plus the offset, in number of slots, computed above. The offset within this
            // particular slot is the remaining offset, in words.
            (
                add_to_b256(get_storage_key::<u64>(ix, &[]), offset_in_slots),
                offset_remaining,
            )
        };

        // Const value for the key from the hash
        let const_key = convert_literal_to_value(context, &Literal::B256(storage_key.into()))
            .add_metadatum(context, span_md_idx);

        // The type of a storage access is `StorageKey` which is a struct containing
        // a `b256`, `u64` and `b256`.
        let b256_ty = Type::get_b256(context);
        let uint64_ty = Type::get_uint64(context);
        let storage_key_aggregate = Type::new_struct(context, vec![b256_ty, uint64_ty, b256_ty]);

        // Local variable holding the `StorageKey` struct
        let storage_key_local_name = self.lexical_map.insert_anon();
        let storage_key_ptr = self
            .function
            .new_local_var(
                context,
                storage_key_local_name,
                storage_key_aggregate,
                None,
                false,
            )
            .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))?;
        let storage_key = self
            .current_block
            .append(context)
            .get_local(storage_key_ptr)
            .add_metadatum(context, span_md_idx);

        // Store the key as the first field in the `StorageKey` struct
        let gep_0_val =
            self.current_block
                .append(context)
                .get_elem_ptr_with_idx(storage_key, b256_ty, 0);
        self.current_block
            .append(context)
            .store(gep_0_val, const_key)
            .add_metadatum(context, span_md_idx);

        // Store the offset as the second field in the `StorageKey` struct
        let offset_within_slot_val = Constant::get_uint(context, 64, offset_within_slot);
        let gep_1_val =
            self.current_block
                .append(context)
                .get_elem_ptr_with_idx(storage_key, uint64_ty, 1);
        self.current_block
            .append(context)
            .store(gep_1_val, offset_within_slot_val)
            .add_metadatum(context, span_md_idx);

        // Store the field identifier as the third field in the `StorageKey` struct
        let unique_field_id = get_storage_key(ix, indices); // use the indices to get a field id that is unique even for zero-sized values that live in the same slot
        let field_id = convert_literal_to_value(context, &Literal::B256(unique_field_id.into()))
            .add_metadatum(context, span_md_idx);
        let gep_2_val =
            self.current_block
                .append(context)
                .get_elem_ptr_with_idx(storage_key, b256_ty, 2);
        self.current_block
            .append(context)
            .store(gep_2_val, field_id)
            .add_metadatum(context, span_md_idx);

        Ok(TerminatorValue::new(storage_key, context))
    }
}
