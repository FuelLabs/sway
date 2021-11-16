use std::collections::HashMap;
use std::iter::FromIterator;

use crate::parse_tree::{AsmOp, AsmRegister};
use crate::{
    parse_tree::{LazyOp, Literal, Visibility},
    semantic_analysis::{ast_node::TypedCodeBlock, ast_node::*, *},
    type_engine::*,
    Ident,
};

mod asm;
mod block;
mod constant;
mod context;
mod function;
mod instruction;
mod irtype;
mod module;
mod optimise;
mod parser;
mod pointer;
pub mod printer;
mod value;
mod verify;

#[cfg(test)]
mod tests;

pub(crate) use asm::*;
pub(crate) use block::*;
pub(crate) use constant::*;
pub(crate) use context::*;
pub(crate) use function::*;
pub(crate) use instruction::*;
pub(crate) use irtype::*;
pub(crate) use module::*;
pub(crate) use pointer::*;
pub(crate) use value::*;

// Exported to asm_generation but not actually used here.
#[allow(unused_imports)]
pub(crate) use parser::*;

// -------------------------------------------------------------------------------------------------

pub(crate) fn compile_ast<'sc>(ast: TypedParseTree<'sc>) -> Result<Context, String> {
    let mut ctx = Context::new();
    match ast {
        TypedParseTree::Script {
            namespace,
            main_function,
            declarations,
            ..
        } => compile_script(&mut ctx, namespace, main_function, declarations),
        TypedParseTree::Predicate {
            //namespace,
            //declarations,
            ..
        } => unimplemented!("compile predicate to ir"),
        TypedParseTree::Contract {
            //abi_entries,
            //namespace,
            //declarations,
            ..
        } => unimplemented!("compile contract to ir"),
        TypedParseTree::Library {
            //namespace,
            //all_nodes,
            ..
        } => unimplemented!("compile library to ir"),
    }?;
    ctx.verify()?;
    Ok(ctx)
}

// -------------------------------------------------------------------------------------------------

fn compile_script<'sc>(
    context: &mut Context,
    _namespace: Namespace<'sc>,
    main_function: TypedFunctionDeclaration<'sc>,
    declarations: Vec<TypedDeclaration<'sc>>,
) -> Result<Module, String> {
    let module = Module::new(context, Kind::Script, "script");

    for declaration in declarations {
        match declaration {
            TypedDeclaration::VariableDeclaration(_) => {
                unimplemented!("compile variable declaration to ir")
            }
            TypedDeclaration::ConstantDeclaration(_) => {
                unimplemented!("compile constant declaration to ir")
            }
            TypedDeclaration::FunctionDeclaration(decl) => compile_function(context, module, decl),
            TypedDeclaration::TraitDeclaration(_) => {
                unimplemented!("compile trait declaration to ir")
            }
            TypedDeclaration::StructDeclaration(decl) => compile_struct_decl(context, decl),
            TypedDeclaration::EnumDeclaration(decl) => compile_enum_decl(context, decl),
            TypedDeclaration::Reassignment(_) => {
                unimplemented!("compile reassignment declaration to ir")
            }
            TypedDeclaration::ImplTrait { .. } => {
                unimplemented!("compile impltrait declaration to ir")
            }
            TypedDeclaration::AbiDeclaration(_) => {
                unimplemented!("compile abideclaration trait declaration to ir")
            }
            TypedDeclaration::ErrorRecovery => unimplemented!("compile error recovery to ir"),
        }?
    }
    compile_function(context, module, main_function)?;

    Ok(module)
}

// -------------------------------------------------------------------------------------------------

fn compile_struct_decl<'sc>(
    context: &mut Context,
    struct_decl: TypedStructDeclaration<'sc>,
) -> Result<(), String> {
    let (field_types, syms): (Vec<_>, Vec<_>) = struct_decl
        .fields
        .into_iter()
        .map(|tsf| {
            (
                convert_resolved_typeid(context, &tsf.r#type, &tsf.span),
                tsf.name.primary_name.to_owned(),
            )
        })
        .unzip();

    let field_types = field_types
        .into_iter()
        .collect::<Result<Vec<_>, String>>()?;

    let aggregate = Aggregate::new(
        context,
        Some(struct_decl.name.primary_name.to_owned()),
        field_types,
    );
    context.add_aggregate_symbols(
        aggregate,
        HashMap::from_iter(syms.into_iter().enumerate().map(|(n, sym)| (sym, n as u64))),
    )
}

// -------------------------------------------------------------------------------------------------

fn compile_enum_decl<'sc>(
    context: &mut Context,
    enum_decl: TypedEnumDeclaration<'sc>,
) -> Result<(), String> {
    let TypedEnumDeclaration {
        name,
        type_parameters,
        variants,
        .. //span,
    } = enum_decl;
    if !type_parameters.is_empty() {
        return Err("Unable to compile generic enums.".into());
    }

    let (field_types, syms): (Vec<_>, Vec<_>) = variants
        .into_iter()
        .map(|tev| {
            (
                convert_resolved_typeid(context, &tev.r#type, &tev.span),
                tev.name.primary_name.to_owned(),
            )
        })
        .unzip();

    let field_types = field_types
        .into_iter()
        .collect::<Result<Vec<_>, String>>()?;

    let aggregate = Aggregate::new(context, Some(name.primary_name.to_owned()), field_types);
    context.add_aggregate_symbols(
        aggregate,
        HashMap::from_iter(syms.into_iter().enumerate().map(|(n, sym)| (sym, n as u64))),
    )
}

// -------------------------------------------------------------------------------------------------

fn compile_function<'sc>(
    context: &mut Context,
    module: Module,
    ast_fn_decl: TypedFunctionDeclaration<'sc>,
) -> Result<(), String> {
    let TypedFunctionDeclaration {
        name,
        body,
        parameters,
        return_type,
        return_type_span,
        visibility,
        is_contract_call,
        ..
    } = ast_fn_decl;

    let args: Vec<_> = parameters
        .iter()
        .map(|param| {
            convert_resolved_typeid(context, &param.r#type, &param.type_span)
                .map(|ty| (param.name.primary_name.into(), ty))
        })
        .collect::<Result<Vec<(String, Type)>, String>>()?;

    let ret_type = convert_resolved_typeid(context, &return_type, &return_type_span)?;
    let func = Function::new(
        context,
        module,
        name.primary_name.to_owned(),
        args,
        ret_type,
        is_contract_call,
        visibility == Visibility::Public,
    );

    let mut compiler = FnCompiler::new(context, module, func);

    let ret_val = compiler.compile_code_block(context, &body)?;
    compiler.current_block.ins(context).ret(ret_val, ret_type);
    Ok(())
}

// -------------------------------------------------------------------------------------------------

struct FnCompiler {
    module: Module,
    function: Function,
    current_block: Block,
}

impl<'sc> FnCompiler {
    fn new(context: &mut Context, module: Module, function: Function) -> Self {
        FnCompiler {
            module,
            function,
            current_block: function.get_entry_block(context),
        }
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_code_block(
        &mut self,
        context: &mut Context,
        ast_block: &TypedCodeBlock<'sc>,
    ) -> Result<Value, String> {
        ast_block
            .contents
            .iter()
            .map(|ast_node| {
                match &ast_node.content {
                    TypedAstNodeContent::ReturnStatement(trs) => {
                        self.compile_return_statement(context, &trs.expr)
                    }
                    TypedAstNodeContent::Declaration(td) => match td {
                        TypedDeclaration::VariableDeclaration(tvd) => {
                            self.compile_var_decl(context, tvd)
                        }
                        TypedDeclaration::ConstantDeclaration(_) => Err("const decl".into()),
                        TypedDeclaration::FunctionDeclaration(_) => Err("func decl".into()),
                        TypedDeclaration::TraitDeclaration(_) => Err("trait decl".into()),
                        TypedDeclaration::StructDeclaration(_) => Err("struct decl".into()),
                        TypedDeclaration::EnumDeclaration(_) => Err("enum decl".into()),
                        TypedDeclaration::Reassignment(tr) => {
                            self.compile_reassignment(context, tr)
                        }
                        TypedDeclaration::ImplTrait { .. } => Err("impl trait decl".into()),
                        TypedDeclaration::AbiDeclaration(_) => Err("abi decl".into()),
                        TypedDeclaration::ErrorRecovery { .. } => Err("error recovery".into()),
                    },
                    TypedAstNodeContent::Expression(te) => {
                        // An expression with an ignored return value... I assume.
                        self.compile_expression(context, te)
                    }
                    TypedAstNodeContent::ImplicitReturnExpression(te) => {
                        self.compile_expression(context, te)
                    }
                    TypedAstNodeContent::WhileLoop(twl) => self.compile_while_loop(context, twl),
                    TypedAstNodeContent::SideEffect => Err("code block side effect".into()),
                }
            })
            .collect::<Result<Vec<_>, String>>()
            .map(|vals| vals.last().cloned())
            .transpose()
            .unwrap_or_else(|| Err("empty code block has no value".into()))
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_expression(
        &mut self,
        context: &mut Context,
        ast_expr: &TypedExpression<'sc>,
    ) -> Result<Value, String> {
        match &ast_expr.expression {
            TypedExpressionVariant::Literal(l) => self.compile_literal(context, l),
            TypedExpressionVariant::FunctionApplication {
                name,
                arguments,
                function_body,
                ..
            } => self.compile_fn_call(
                context,
                name.suffix.primary_name,
                arguments,
                Some(function_body),
            ),
            TypedExpressionVariant::LazyOperator { op, lhs, rhs, .. } => {
                self.compile_lazy_op(context, op, lhs, rhs)
            }
            TypedExpressionVariant::VariableExpression { name } => {
                self.compile_var_expr(context, name.primary_name)
            }
            TypedExpressionVariant::Unit => Ok(Constant::get_unit(context)),
            TypedExpressionVariant::Array { .. } => Err("expr array".into()),
            TypedExpressionVariant::MatchExpression { .. } => Err("expr match".into()),
            TypedExpressionVariant::StructExpression {
                struct_name,
                fields,
            } => self.compile_struct_expr(context, struct_name.primary_name, fields),
            TypedExpressionVariant::CodeBlock(cb) => self.compile_code_block(context, cb),
            TypedExpressionVariant::FunctionParameter => Err("expr func param".into()),
            TypedExpressionVariant::IfExp {
                condition,
                then,
                r#else,
            } => self.compile_if(context, condition, then, r#else),
            TypedExpressionVariant::AsmExpression {
                registers,
                body,
                returns,
                ..
            } => self.compile_asm_expr(context, registers, body, returns),
            TypedExpressionVariant::StructFieldAccess {
                prefix,
                field_to_access,
                resolved_type_of_parent,
                ..
            } => self.compile_struct_field_expr(
                context,
                prefix,
                field_to_access,
                resolved_type_of_parent,
            ),
            TypedExpressionVariant::EnumInstantiation { tag, contents, .. } => {
                self.compile_enum_expr(context, tag, contents)
            }
            TypedExpressionVariant::AbiCast { .. } => Err("expr abi".into()),
        }
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_return_statement(
        &mut self,
        context: &mut Context,
        ast_expr: &TypedExpression<'sc>,
    ) -> Result<Value, String> {
        let ret_value = self.compile_expression(context, ast_expr)?;
        match ret_value.get_type(context) {
            None => Err("Unable to determine type for return statement expression.".into()),
            Some(ret_ty) => {
                self.current_block.ins(context).ret(ret_value, ret_ty);
                Ok(Constant::get_unit(context))
            }
        }
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_literal(
        &mut self,
        context: &mut Context,
        ast_literal: &Literal<'sc>,
    ) -> Result<Value, String> {
        match ast_literal {
            Literal::U8(n) | Literal::Byte(n) => Ok(Constant::get_uint(context, 8, *n as u64)),
            Literal::U16(n) => Ok(Constant::get_uint(context, 16, *n as u64)),
            Literal::U32(n) => Ok(Constant::get_uint(context, 32, *n as u64)),
            Literal::U64(n) => Ok(Constant::get_uint(context, 64, *n)),
            Literal::String(s) => Ok(Constant::get_string(context, (*s).to_owned())),
            Literal::Boolean(b) => Ok(Constant::get_bool(context, *b)),
            Literal::B256(bs) => Ok(Constant::get_b256(context, bs.clone())),
        }
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_lazy_op(
        &mut self,
        context: &mut Context,
        ast_op: &LazyOp,
        ast_lhs: &TypedExpression<'sc>,
        ast_rhs: &TypedExpression<'sc>,
    ) -> Result<Value, String> {
        let rhs_block = self.function.create_block(context, None);
        let final_block = self.function.create_block(context, None);

        // Short-circuit: if LHS is true for AND we still must eval the RHS block; for OR we can
        // skip the RHS block, and vice-versa.
        let lhs_val = self.compile_expression(context, ast_lhs)?;
        let cond_builder = self.current_block.ins(context);
        match ast_op {
            LazyOp::And => {
                cond_builder.conditional_branch(lhs_val, rhs_block, final_block, Some(lhs_val))
            }
            LazyOp::Or => {
                cond_builder.conditional_branch(lhs_val, final_block, rhs_block, Some(lhs_val))
            }
        };

        self.current_block = rhs_block;
        let rhs_val = self.compile_expression(context, ast_rhs)?;
        self.current_block
            .ins(context)
            .branch(final_block, Some(rhs_val));

        self.current_block = final_block;
        Ok(final_block.get_phi(context))
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_fn_call(
        &mut self,
        context: &mut Context,
        ast_name: &str,
        ast_args: &[(Ident<'sc>, TypedExpression<'sc>)],
        callee_body: Option<&TypedCodeBlock<'sc>>,
    ) -> Result<Value, String> {
        // XXX To do: Calling into other modules, managing namespaces.
        //
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

        match context
            .module_iter()
            .flat_map(|module| module.function_iter(context))
            .find(|function| function.get_name(context) == ast_name)
        {
            Some(callee) => {
                let args = ast_args
                    .iter()
                    .map(|(_, expr)| self.compile_expression(context, expr))
                    .collect::<Result<Vec<Value>, String>>()?;
                Ok(self.current_block.ins(context).call(callee, &args))
            }

            None if callee_body.is_none() => Err(format!("function not found: {}", ast_name)),

            None => {
                // Firstly create the single-use callee by fudging an AST declaration.
                let callee_name = context.get_unique_name();
                let callee_ident = Ident {
                    primary_name: &callee_name,
                    span: crate::span::Span {
                        span: pest::Span::new(" ", 0, 0).unwrap(),
                        path: None,
                    },
                };

                let parameters = ast_args
                    .iter()
                    .map(|(name, expr)| TypedFunctionParameter {
                        name: name.clone(),
                        r#type: expr.return_type,
                        type_span: crate::span::Span {
                            span: pest::Span::new(" ", 0, 0).unwrap(),
                            path: None,
                        },
                    })
                    .collect();

                let callee_body = callee_body.unwrap();

                // We're going to have to reverse engineer the return type.
                //    pub(crate) return_type: TypeId,
                let return_type = Self::get_codeblock_return_type(&callee_body).ok_or(
                    "Unable to determine return type of code block when reconstructing \
                    library function."
                        .to_owned(),
                )?;

                let callee_fn_decl = TypedFunctionDeclaration {
                    name: callee_ident,
                    body: callee_body.clone(),
                    parameters,
                    span: crate::span::Span {
                        span: pest::Span::new(" ", 0, 0).unwrap(),
                        path: None,
                    },
                    return_type,
                    type_parameters: Vec::new(),
                    return_type_span: crate::span::Span {
                        span: pest::Span::new(" ", 0, 0).unwrap(),
                        path: None,
                    },
                    visibility: Visibility::Private,
                    is_contract_call: false,
                };

                compile_function(context, self.module, callee_fn_decl)?;

                // Then recursively create a call to it.
                self.compile_fn_call(context, &callee_name, ast_args, None)
            }
        }
    }

    fn get_codeblock_return_type(codeblock: &TypedCodeBlock<'sc>) -> Option<TypeId> {
        codeblock
            .contents
            .iter()
            .find_map(|node| match &node.content {
                TypedAstNodeContent::ReturnStatement(trs) => Some(trs.expr.return_type),
                TypedAstNodeContent::ImplicitReturnExpression(te) => Some(te.return_type),
                _otherwise => None,
            })
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_if(
        &mut self,
        context: &mut Context,
        ast_condition: &TypedExpression<'sc>,
        ast_then: &TypedExpression<'sc>,
        ast_else: &Option<Box<TypedExpression<'sc>>>,
    ) -> Result<Value, String> {
        let true_block = self.function.create_block(context, None);
        let false_block = self.function.create_block(context, None);
        let merge_block = self.function.create_block(context, None);

        let cond_value = self.compile_expression(context, ast_condition)?;
        self.current_block.ins(context).conditional_branch(
            cond_value,
            true_block,
            false_block,
            None,
        );

        self.current_block = true_block;
        let true_value = self.compile_expression(context, ast_then)?;
        true_block
            .ins(context)
            .branch(merge_block, Some(true_value));

        self.current_block = false_block;
        let false_value = match ast_else {
            None => Constant::get_unit(context),
            Some(expr) => self.compile_expression(context, expr)?,
        };
        false_block
            .ins(context)
            .branch(merge_block, Some(false_value));

        self.current_block = merge_block;
        Ok(merge_block.get_phi(context))
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_while_loop(
        &mut self,
        context: &mut Context,
        ast_while_loop: &TypedWhileLoop<'sc>,
    ) -> Result<Value, String> {
        // We're dancing around a bit here to make the blocks sit in the right order.  Ideally we
        // have the cond block, followed by the body block which may contain other blocks, and the
        // final block comes after any body block(s).

        // Jump to the while cond block.
        let cond_block = self.function.create_block(context, Some("while".into()));
        self.current_block.ins(context).branch(cond_block, None);

        // Fill in the body block now, jump unconditionally to the cond block at its end.
        let body_block = self
            .function
            .create_block(context, Some("while_body".into()));
        self.current_block = body_block;
        self.compile_code_block(context, &ast_while_loop.body)?;
        self.current_block.ins(context).branch(cond_block, None);

        // Create the final block after we're finished with the body.
        let final_block = self
            .function
            .create_block(context, Some("end_while".into()));

        // Add the conditional which jumps into the body or out to the final block.
        self.current_block = cond_block;
        let cond_value = self.compile_expression(context, &ast_while_loop.condition)?;
        self.current_block.ins(context).conditional_branch(
            cond_value,
            body_block,
            final_block,
            None,
        );

        self.current_block = final_block;
        Ok(Constant::get_unit(context))
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_var_expr(&mut self, context: &mut Context, name: &str) -> Result<Value, String> {
        match self.function.get_arg(context, name) {
            Some(val) => Ok(val),
            None => {
                let ptr = self
                    .function
                    .get_local_ptr(context, name)
                    .ok_or(format!("variable not found: {}", name))?;
                Ok(if ptr.is_struct_ptr(context) {
                    self.current_block.ins(context).get_ptr(ptr)
                } else {
                    self.current_block.ins(context).load(ptr)
                })
            }
        }
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_var_decl(
        &mut self,
        context: &mut Context,
        ast_var_decl: &TypedVariableDeclaration<'sc>,
    ) -> Result<Value, String> {
        let TypedVariableDeclaration {
            name,
            body,
            is_mutable,
        } = ast_var_decl;
        let return_type = convert_resolved_typeid(context, &body.return_type, &body.span)?;
        let ptr = self.function.new_local_ptr(
            context,
            name.primary_name.into(),
            return_type,
            *is_mutable,
            None,
        )?;
        let init_val = self.compile_expression(context, body)?;
        self.current_block.ins(context).store(ptr, init_val);
        Ok(init_val)
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_reassignment(
        &mut self,
        context: &mut Context,
        ast_reassignment: &TypedReassignment<'sc>,
    ) -> Result<Value, String> {
        let name = ast_reassignment.lhs[0].name.primary_name;
        let ptr_val = self
            .function
            .get_local_ptr(context, name)
            .ok_or(format!("variable not found: {}", name))?;

        let reassign_val = self.compile_expression(context, &ast_reassignment.rhs)?;

        if ast_reassignment.lhs.len() == 1 {
            // A non-aggregate; use a `store`.
            self.current_block.ins(context).store(ptr_val, reassign_val);
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
                                    .get_aggregate_index(&aggregate, field_name.name.primary_name)
                                {
                                    None => Err(format!(
                                        "Unknown field name {} for struct ???",
                                        field_name.name.primary_name
                                    )),
                                    Some(field_idx) => {
                                        let field_type = context.aggregates[aggregate.0]
                                            .field_types
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

            let get_ptr_val = self.current_block.ins(context).get_ptr(ptr_val);
            self.current_block
                .ins(context)
                .insert_value(get_ptr_val, ty, reassign_val, field_idcs);
        }

        // This shouldn't really return a value, it doesn't make sense to return the `store` or
        // `insert_value` instruction, but we need to return something at this stage.
        Ok(reassign_val)
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_struct_expr(
        &mut self,
        context: &mut Context,
        struct_name: &str,
        fields: &[TypedStructExpressionField<'sc>],
    ) -> Result<Value, String> {
        let aggregate = context
            .get_aggregate_by_name(struct_name)
            .ok_or_else(|| format!("Unknown aggregate {}", struct_name))?;

        // Compile each of the values for field initialisers.
        let inserted_values = fields
            .iter()
            .map(|field_value| self.compile_expression(context, &field_value.value))
            .collect::<Result<Vec<_>, String>>()?;

        // Collect the corresponding field indices.
        let inserted_idcs = fields
            .iter()
            .map(|field_value| {
                context
                    .get_aggregate_index(&aggregate, &field_value.name.primary_name)
                    .ok_or_else(|| {
                        format!(
                            "Unknown field name {} for aggregate {}",
                            field_value.name.primary_name, struct_name
                        )
                    })
            })
            .collect::<Result<Vec<_>, String>>()?;

        // Start with a constant empty struct and then fill in the values.
        let agg_value = Constant::get_undef(context, Type::Struct(aggregate));
        Ok(inserted_values
            .into_iter()
            .zip(inserted_idcs.into_iter())
            .fold(agg_value, |agg_value, (insert_val, insert_idx)| {
                self.current_block.ins(context).insert_value(
                    agg_value,
                    aggregate,
                    insert_val,
                    vec![insert_idx],
                )
            }))
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_struct_field_expr(
        &mut self,
        context: &mut Context,
        ast_struct_expr: &TypedExpression<'sc>,
        ast_field: &OwnedTypedStructField,
        _ast_parent_type: &TypeId,
    ) -> Result<Value, String> {
        // ast_struct_expr must be either a variable expression, or a..? So struct_val may be a
        // get_ptr instruction, or..?
        let struct_val = self.compile_expression(context, ast_struct_expr)?;
        if let ValueContent::Instruction(instruction) = &context.values[struct_val.0] {
            match instruction {
                Instruction::GetPointer(ptr) => {
                    let aggregate = match ptr.get_type(context) {
                        Type::Struct(agg) => *agg,
                        _otherwise => return Err("Bug! get_ptr() is not to a struct.".to_owned()),
                    };
                    let field_idx = context
                        .get_aggregate_index(&aggregate, &ast_field.name)
                        .ok_or_else(|| {
                            format!("Unknown field name {} in struct ???", ast_field.name)
                        })?;

                    Ok(self.current_block.ins(context).extract_value(
                        struct_val,
                        aggregate,
                        vec![field_idx],
                    ))
                }
                _otherwise => {
                    Err("Bug! Unhandled instruction parent for struct field access.".to_owned())
                }
            }
        } else {
            Err("Unsupported struct value in field expression".to_owned())
        }
    }

    // ---------------------------------------------------------------------------------------------
    // This might not work in the end, but I'm currently thinking an enum instantiation can
    // literally be a tagged value, which is a tuple (tag, value), which is an aggregate.
    //
    // It might fall over when we try to compare them or something -- we need to compare the tags
    // before we decide they're different, even before looking at the type of the value since it
    // will almost definitely be different if the tag is.  OTOH, for them to be comparable they'd
    // at least have to have the same type AND the same tag.  The tag is only there if there's no
    // content or same-typed-but-different variants.

    fn compile_enum_expr(
        &mut self,
        context: &mut Context,
        tag: &usize,
        contents: &Option<Box<TypedExpression<'sc>>>,
    ) -> Result<Value, String> {
        let tag_value = Constant::get_uint(context, 64, *tag as u64);
        Ok(match contents {
            Some(te) => {
                let contents_value = self.compile_expression(context, &te)?;
                let contents_type = contents_value
                    .get_type(context)
                    .ok_or("Unable to determine type of enum variant.".to_owned())?;
                let aggregate = Aggregate::new(context, None, vec![Type::Uint(64), contents_type]);
                let agg_value = Constant::get_undef(context, Type::Struct(aggregate));
                let agg_value = self.current_block.ins(context).insert_value(
                    agg_value,
                    aggregate,
                    tag_value,
                    vec![0],
                );
                self.current_block.ins(context).insert_value(
                    agg_value,
                    aggregate,
                    contents_value,
                    vec![1],
                )
            }
            None => {
                // Create a tuple with just the tag.
                let aggregate = Aggregate::new(context, None, vec![Type::Uint(64)]);
                let agg_value = Constant::get_undef(context, Type::Struct(aggregate));
                self.current_block.ins(context).insert_value(
                    agg_value,
                    aggregate,
                    tag_value,
                    vec![0],
                )
            }
        })
    }

    // ---------------------------------------------------------------------------------------------

    fn compile_asm_expr(
        &mut self,
        context: &mut Context,
        registers: &Vec<TypedAsmRegisterDeclaration<'sc>>,
        body: &Vec<AsmOp<'sc>>,
        returns: &Option<(AsmRegister, crate::Span<'sc>)>,
    ) -> Result<Value, String> {
        let registers = registers
            .iter()
            .map(
                |TypedAsmRegisterDeclaration {
                     initializer, name, ..
                 }| {
                    // Take the optional initialiser, map it to an Option<Result<Value>>,
                    // transpose that to Result<Option<Value>> and map that to an AsmArg.
                    initializer
                        .as_ref()
                        .map(|init_expr| self.compile_expression(context, init_expr))
                        .transpose()
                        .map(|init| AsmArg {
                            name: (*name).into(),
                            initializer: init,
                        })
                },
            )
            .collect::<Result<Vec<AsmArg>, String>>()?;
        let body = body
            .iter()
            .map(
                |AsmOp {
                     op_name,
                     op_args,
                     immediate,
                     ..
                 }| AsmInstruction {
                    name: op_name.primary_name.to_owned(),
                    args: op_args
                        .iter()
                        .map(|id| id.primary_name.to_owned())
                        .collect(),
                    immediate: immediate.as_ref().map(|id| id.primary_name.to_owned()),
                },
            )
            .collect();
        let returns = returns.as_ref().map(|(asm_reg, _)| asm_reg.name.clone());
        Ok(self
            .current_block
            .ins(context)
            .asm_block(registers, body, returns))
    }
}

// -------------------------------------------------------------------------------------------------

fn convert_resolved_typeid<'sc>(
    context: &mut Context,
    ast_type: &TypeId,
    span: &crate::Span<'sc>,
) -> Result<Type, String> {
    // There's probably a better way to convert TypeError to String, but... we'll use something
    // other than String eventually?  IrError?
    convert_resolved_type(
        context,
        &resolve_type(*ast_type, span).map_err(|ty_err| format!("{:?}", ty_err))?,
    )
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
        TypeInfo::Boolean => Type::Bool,
        TypeInfo::Unit => Type::Unit,
        TypeInfo::Byte => Type::Uint(8), // XXX?
        TypeInfo::B256 => Type::B256,
        TypeInfo::Str(n) => Type::String(*n),
        TypeInfo::Struct { name, .. } => match context.get_aggregate_by_name(name) {
            Some(existing_aggregate) => Type::Struct(existing_aggregate),
            None => {
                return Err(format!(
                    "Unknown struct: {} XXX bug, need to find struct decls and add them first",
                    name
                ))
            }
        },
        TypeInfo::Enum { name, .. } => match context.get_aggregate_by_name(name) {
            Some(existing_aggregate) => Type::Enum(existing_aggregate),
            None => {
                return Err(format!(
                    "Unknown enum: {} XXX bug, need to find enum decls and add them first",
                    name
                ))
            }
        },
        TypeInfo::Custom { .. } => return Err("can't do custom types yet".into()),
        TypeInfo::SelfType { .. } => return Err("can't do self types yet".into()),
        TypeInfo::Contract => return Err("contract types not supported yet!".into()),
        TypeInfo::ContractCaller { .. } => {
            return Err("contract caller types not supported yet!".into())
        }
        TypeInfo::Unknown => return Err("unknown type found in AST..?".into()),
        TypeInfo::Numeric => return Err("'numeric' type found in AST..?".into()),
        TypeInfo::Ref(_) => return Err("ref type found in AST..?".into()),
        TypeInfo::ErrorRecovery => return Err("error recovery type found in AST..?".into()),
    })
}

//fn convert_resolved_type_to_aggregate(
//    context: &mut Context,
//    ast_type: &ResolvedType,
//) -> Result<Aggregate, String> {
//    match convert_resolved_type(context, ast_type)? {
//        Type::Struct(agg) => Ok(agg),
//        ty => Err(format!(
//            "Expecting struct but found {}",
//            ty.as_string(context)
//        )),
//    }
//}

// -------------------------------------------------------------------------------------------------
