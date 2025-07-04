//! This module contains common API for visiting elements in lexed and typed trees.

#![allow(dead_code)]

use anyhow::{bail, Ok, Result};
use std::sync::Arc;

use duplicate::duplicate_item;
use sway_ast::{
    expr::{LoopControlFlow, ReassignmentOp, ReassignmentOpVariant},
    keywords::*,
    Assignable, CodeBlockContents, Expr, IfCondition, IfExpr, ItemFn, ItemImpl, ItemImplItem,
    ItemKind, ItemStruct, ItemUse, PathExprSegment, Statement, StatementLet,
};
use sway_core::{
    decl_engine::DeclEngine,
    language::{
        lexed::LexedModule,
        ty::{
            self, TyAstNodeContent, TyCodeBlock, TyDecl, TyExpression, TyExpressionVariant,
            TyFunctionDecl, TyImplSelfOrTrait, TyIntrinsicFunctionKind, TyModule,
            TyReassignmentTarget, TySideEffect, TySideEffectVariant, TyStructDecl, TyTraitItem,
            TyUseStatement, TyVariableDecl,
        },
        CallPath,
    },
    Engines, TypeId,
};
use sway_types::{Ident, Spanned};

use crate::{
    internal_error,
    migrations::{DryRun, MutProgramInfo, ProgramInfo},
};

pub(crate) struct VisitingContext<'a> {
    /// The name of the current package being migrated.
    pub pkg_name: &'a str,
    pub engines: &'a Engines,
    pub dry_run: DryRun,
}

/// If a [TreesVisitorMut] modifies the lexed element in a way
/// that its corresponding typed element becomes obsolete,
/// it must communicate that by returning [InvalidateTypedElement::Yes].
pub(crate) enum InvalidateTypedElement {
    Yes,
    No,
}

// TODO: This is a very first, pragmatic version of the more detailed visitor pattern,
//       to support migrations localized in expressions, that do not need access to
//       a larger context. If needed, we can later provide specific `VisitingContext`
//       for each `visiting_...` method, that will provide additional contextual
//       information about the parent. Such and similar extensions will be driven
//       by the concrete need of migrations we will encounter in the future.
#[duplicate_item(
    __TreesVisitor      __ref_type(type);
    [TreesVisitor]      [&type];
    [TreesVisitorMut]   [&mut type];
)]
#[allow(unused_variables)]
/// Represents a visitor that simultaneously traverses the elements in the lexed tree,
/// mutable or immutable, and their corresponding typed elements.
///
/// Due to conditional compilation, the corresponding typed elements do not necessarily
/// exist. That's why they are always passed as `Option`al.
///
/// A [TreesVisitorMut] can mutate lexed elements it visits. While this is far from ideal,
/// it is a pragmatic design choice that still allows writing a large category of
/// migrations, without developing a full-blown framework for matching, transforming, and
/// rendering trees, as proposed in
/// [Provide common infrastructure for writing Sway code analyzers and generators](https://github.com/FuelLabs/sway/issues/6836).
/// Even just separating the traversal, marking lexed elements for change, and then changing
/// them in a separate pass, would be an investment that hardly pays off only for migrations.
///
/// The consequence of the fact, that the visitor can mutate the tree it traverses, requires
/// invalidation of the corresponding typed element, which is handled via [InvalidateTypedElement].
///
/// Visitors can have their own state, but most of them will only want to collect [Span]s
/// of occurrences to migrate. To avoid boilerplate code in visitors and support that
/// most common case, all the `visit_...` methods provide a convenient mutable `output`
/// argument, that can be used to collect the output of a migration step, most commonly
/// the [Span]s of occurrences.
pub(crate) trait __TreesVisitor<O> {
    fn visit_module(
        &mut self,
        ctx: &VisitingContext,
        lexed_module: __ref_type([LexedModule]),
        ty_module: Option<&TyModule>,
        output: &mut Vec<O>,
    ) -> Result<InvalidateTypedElement> {
        Ok(InvalidateTypedElement::No)
    }
    fn visit_use(
        &mut self,
        ctx: &VisitingContext,
        lexed_use: __ref_type([ItemUse]),
        ty_use: Option<&TyUseStatement>,
        output: &mut Vec<O>,
    ) -> Result<InvalidateTypedElement> {
        Ok(InvalidateTypedElement::No)
    }
    fn visit_struct_decl(
        &mut self,
        ctx: &VisitingContext,
        lexed_struct: __ref_type([ItemStruct]),
        ty_struct: Option<Arc<TyStructDecl>>,
        output: &mut Vec<O>,
    ) -> Result<InvalidateTypedElement> {
        Ok(InvalidateTypedElement::No)
    }
    fn visit_fn_decl(
        &mut self,
        ctx: &VisitingContext,
        lexed_fn: __ref_type([ItemFn]),
        ty_fn: Option<Arc<TyFunctionDecl>>,
        output: &mut Vec<O>,
    ) -> Result<InvalidateTypedElement> {
        Ok(InvalidateTypedElement::No)
    }
    fn visit_impl(
        &mut self,
        ctx: &VisitingContext,
        lexed_impl: __ref_type([ItemImpl]),
        ty_impl: Option<Arc<TyImplSelfOrTrait>>,
        output: &mut Vec<O>,
    ) -> Result<InvalidateTypedElement> {
        Ok(InvalidateTypedElement::No)
    }
    fn visit_block(
        &mut self,
        ctx: &VisitingContext,
        lexed_block: __ref_type([CodeBlockContents]),
        ty_block: Option<&TyCodeBlock>,
        output: &mut Vec<O>,
    ) -> Result<InvalidateTypedElement> {
        Ok(InvalidateTypedElement::No)
    }
    fn visit_statement_let(
        &mut self,
        ctx: &VisitingContext,
        lexed_let: __ref_type([StatementLet]),
        ty_var_decl: Option<&TyVariableDecl>,
        output: &mut Vec<O>,
    ) -> Result<InvalidateTypedElement> {
        Ok(InvalidateTypedElement::No)
    }
    fn visit_expr(
        &mut self,
        ctx: &VisitingContext,
        lexed_expr: __ref_type([Expr]),
        ty_expr: Option<&TyExpression>,
        output: &mut Vec<O>,
    ) -> Result<InvalidateTypedElement> {
        Ok(InvalidateTypedElement::No)
    }
    fn visit_if(
        &mut self,
        ctx: &VisitingContext,
        lexed_if: __ref_type([IfExpr]),
        ty_if: Option<&TyExpression>,
        output: &mut Vec<O>,
    ) -> Result<InvalidateTypedElement> {
        Ok(InvalidateTypedElement::No)
    }
    fn visit_fn_call(
        &mut self,
        ctx: &VisitingContext,
        lexed_fn_call: __ref_type([Expr]),
        ty_fn_call: Option<&TyExpression>,
        output: &mut Vec<O>,
    ) -> Result<InvalidateTypedElement> {
        Ok(InvalidateTypedElement::No)
    }
    fn visit_method_call(
        &mut self,
        ctx: &VisitingContext,
        lexed_method_call: __ref_type([Expr]),
        ty_method_call: Option<&TyExpression>,
        output: &mut Vec<O>,
    ) -> Result<InvalidateTypedElement> {
        Ok(InvalidateTypedElement::No)
    }
    #[allow(clippy::too_many_arguments)]
    fn visit_reassignment(
        &mut self,
        ctx: &VisitingContext,
        lexed_op: __ref_type([ReassignmentOp]),
        lexed_lhs: __ref_type([Assignable]),
        ty_lhs: Option<&TyReassignmentTarget>,
        lexed_rhs: __ref_type([Expr]),
        ty_rhs: Option<&TyExpression>,
        output: &mut Vec<O>,
    ) -> Result<InvalidateTypedElement> {
        Ok(InvalidateTypedElement::No)
    }
    #[allow(clippy::too_many_arguments)]
    fn visit_binary_op(
        &mut self,
        ctx: &VisitingContext,
        op: &'static str,
        lexed_lhs: __ref_type([Expr]),
        ty_lhs: Option<&TyExpression>,
        lexed_rhs: __ref_type([Expr]),
        ty_rhs: Option<&TyExpression>,
        output: &mut Vec<O>,
    ) -> Result<InvalidateTypedElement> {
        Ok(InvalidateTypedElement::No)
    }
}

#[allow(dead_code)]
pub(crate) struct ProgramVisitor;
pub(crate) struct ProgramVisitorMut;

#[duplicate_item(
    __ProgramVisitor      __ProgramInfo      __TreesVisitor     __ref_type(type)  __ref(value)  __iter       __as_ref;
    [ProgramVisitor]      [ProgramInfo]      [TreesVisitor]     [&type]           [&value]      [iter]       [as_ref];
    [ProgramVisitorMut]   [MutProgramInfo]   [TreesVisitorMut]  [&mut type]       [&mut value]  [iter_mut]   [as_mut];
)]
impl __ProgramVisitor {
    pub(crate) fn visit_program<V, O>(
        program_info: __ref_type([__ProgramInfo]),
        dry_run: DryRun,
        visitor: &mut V,
    ) -> Result<Vec<O>>
    where
        V: __TreesVisitor<O>,
    {
        let ctx = VisitingContext {
            #[allow(clippy::needless_borrow)] // Clippy lint false positive. Actually, a Clippy bug.
            pkg_name: &program_info.pkg_name,
            engines: program_info.engines,
            dry_run,
        };

        let mut output = vec![];

        Self::visit_module(
            &ctx,
            __ref([program_info.lexed_program.root]),
            Some(&program_info.ty_program.root_module),
            visitor,
            &mut output,
        )?;

        Ok(output)
    }

    fn visit_module<V, O>(
        ctx: &VisitingContext,
        lexed_module: __ref_type([LexedModule]),
        ty_module: Option<&TyModule>,
        visitor: &mut V,
        output: &mut Vec<O>,
    ) -> Result<()>
    where
        V: __TreesVisitor<O>,
    {
        let ty_module = match visitor.visit_module(ctx, lexed_module, ty_module, output)? {
            InvalidateTypedElement::Yes => None,
            InvalidateTypedElement::No => ty_module,
        };

        // We need to visit submodules separately of other items, because they
        // are actually stored in `lexed_modules.submodules`.
        for submodule in lexed_module.submodules.__iter() {
            let ty_submodule = ty_module.and_then(|ty_module| {
                ty_module
                    .submodules
                    .iter()
                    .find(|ty_submodule| ty_submodule.0 == submodule.0)
                    .map(|ty_submodule| &*ty_submodule.1.module)
            });
            Self::visit_module(
                ctx,
                __ref([submodule.1.module]),
                ty_submodule,
                visitor,
                output,
            )?;
        }

        for annotated_item in lexed_module.tree.value.items.__iter() {
            match __ref([annotated_item.value]) {
                ItemKind::Submodule(_submodule) => {
                    // TODO: Implement visiting `mod`.
                    // Modules are already visited above, but we also want to
                    // visit `mod` items, in case migrations need to inspect
                    // or modify them.
                }
                ItemKind::Use(item_use) => {
                    let ty_use = ty_module.and_then(|ty_module| {
                        ty_module
                            .all_nodes
                            .iter()
                            .find_map(|node| match &node.content {
                                TyAstNodeContent::SideEffect(TySideEffect {
                                    side_effect: TySideEffectVariant::UseStatement(ty_use),
                                }) if ty_use.span == item_use.span() => Some(ty_use),
                                _ => None,
                            })
                    });

                    visitor.visit_use(ctx, item_use, ty_use, output)?;
                }
                ItemKind::Struct(item_struct) => {
                    let ty_struct_decl = ty_module.and_then(|ty_module| {
                        ty_module
                            .all_nodes
                            .iter()
                            .find_map(|node| match &node.content {
                                TyAstNodeContent::Declaration(TyDecl::StructDecl(
                                    ty_struct_decl,
                                )) => {
                                    let ty_struct_decl =
                                        ctx.engines.de().get_struct(&ty_struct_decl.decl_id);
                                    if ty_struct_decl.span == item_struct.span() {
                                        Some(ty_struct_decl)
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            })
                    });

                    visitor.visit_struct_decl(ctx, item_struct, ty_struct_decl, output)?;
                }
                ItemKind::Enum(_item_enum) => {
                    // TODO: Implement visiting `enum`.
                }
                ItemKind::Fn(item_fn) => {
                    let ty_fn = ty_module.and_then(|ty_module| {
                        ty_module
                            .all_nodes
                            .iter()
                            .find_map(|node| match &node.content {
                                TyAstNodeContent::Declaration(TyDecl::FunctionDecl(
                                    function_decl,
                                )) => {
                                    let function_decl =
                                        ctx.engines.de().get_function(&function_decl.decl_id);
                                    (function_decl.name == item_fn.fn_signature.name)
                                        .then_some(function_decl)
                                }
                                _ => None,
                            })
                    });

                    Self::visit_fn_decl(ctx, item_fn, ty_fn, visitor, output)?;
                }
                ItemKind::Trait(_item_trait) => {
                    // TODO: Implement visiting `trait`.
                }
                ItemKind::Impl(item_impl) => {
                    let ty_impl = ty_module.and_then(|ty_module| {
                        ty_module
                            .all_nodes
                            .iter()
                            .find_map(|node| match &node.content {
                                TyAstNodeContent::Declaration(TyDecl::ImplSelfOrTrait(
                                    impl_decl,
                                )) => {
                                    let impl_decl =
                                        ctx.engines.de().get_impl_self_or_trait(&impl_decl.decl_id);
                                    (impl_decl.span == item_impl.span()).then_some(impl_decl)
                                }
                                _ => None,
                            })
                    });
                    Self::visit_impl(ctx, item_impl, ty_impl, visitor, output)?;
                }
                ItemKind::Abi(_item_abi) => {
                    // TODO: Implement visiting `abi`.
                }
                ItemKind::Const(_item_const) => {
                    // TODO: Implement visiting `const`.
                }
                ItemKind::Storage(_item_storage) => {
                    // TODO: Implement visiting `storage`.
                }
                ItemKind::Configurable(_item_configurable) => {
                    // TODO: Implement visiting `configurable`.
                }
                ItemKind::TypeAlias(_item_type_alias) => {
                    // TODO: Implement visiting `type`.
                }
                ItemKind::Error(_spans, _error_emitted) => {
                    bail!(internal_error("`ItemKind::Error` cannot happen, because `forc migrate` analyzes only successfully compiled programs."));
                }
            }
        }

        Ok(())
    }

    fn visit_fn_decl<V, O>(
        ctx: &VisitingContext,
        lexed_fn: __ref_type([ItemFn]),
        ty_fn: Option<Arc<TyFunctionDecl>>,
        visitor: &mut V,
        output: &mut Vec<O>,
    ) -> Result<()>
    where
        V: __TreesVisitor<O>,
    {
        let ty_fn = match visitor.visit_fn_decl(ctx, lexed_fn, ty_fn.clone(), output)? {
            InvalidateTypedElement::Yes => None,
            InvalidateTypedElement::No => ty_fn,
        };

        Self::visit_block(
            ctx,
            __ref([lexed_fn.body.inner]),
            ty_fn.as_ref().map(|ty| &ty.body),
            visitor,
            output,
        )?;

        Ok(())
    }

    fn visit_impl<V, O>(
        ctx: &VisitingContext,
        lexed_impl: __ref_type([ItemImpl]),
        ty_impl: Option<Arc<TyImplSelfOrTrait>>,
        visitor: &mut V,
        output: &mut Vec<O>,
    ) -> Result<()>
    where
        V: __TreesVisitor<O>,
    {
        let ty_impl = match visitor.visit_impl(ctx, lexed_impl, ty_impl.clone(), output)? {
            InvalidateTypedElement::Yes => None,
            InvalidateTypedElement::No => ty_impl,
        };

        for annotated_lexed_impl_item in lexed_impl.contents.inner.__iter() {
            // TODO: Implement visiting `item's annotations`.
            let lexed_impl_item = __ref([annotated_lexed_impl_item.value]);
            match lexed_impl_item {
                ItemImplItem::Fn(item_fn) => {
                    let ty_item_fn = ty_impl.as_ref().and_then(|ty_impl| {
                        ty_impl.items.iter().find_map(|item| match item {
                            TyTraitItem::Fn(function_decl) => {
                                let function_decl =
                                    ctx.engines.de().get_function(function_decl.id());
                                (function_decl.name == item_fn.fn_signature.name)
                                    .then_some(function_decl)
                            }
                            _ => None,
                        })
                    });

                    Self::visit_fn_decl(ctx, item_fn, ty_item_fn, visitor, output)?;
                }
                ItemImplItem::Const(_item_const) => {
                    // TODO: Implement visiting `associated consts`.
                }
                ItemImplItem::Type(_trait_type) => {
                    // TODO: Implement visiting `associated types`.
                }
            }
        }

        Ok(())
    }

    fn visit_block<V, O>(
        ctx: &VisitingContext,
        lexed_block: __ref_type([CodeBlockContents]),
        ty_block: Option<&TyCodeBlock>,
        visitor: &mut V,
        output: &mut Vec<O>,
    ) -> Result<()>
    where
        V: __TreesVisitor<O>,
    {
        let ty_block = match visitor.visit_block(ctx, lexed_block, ty_block, output)? {
            InvalidateTypedElement::Yes => None,
            InvalidateTypedElement::No => ty_block,
        };

        for statement in lexed_block.statements.__iter() {
            let ty_node = ty_block.and_then(|ty_block| {
                ty_block
                    .contents
                    .iter()
                    .find(|ty_node| statement.span().contains(&ty_node.span))
            });
            match statement {
                Statement::Let(statement_let) => {
                    let ty_var_decl = ty_node.map(|ty_node|
                        match &ty_node.content {
                            TyAstNodeContent::Declaration(ty_decl) => match ty_decl {
                                TyDecl::VariableDecl(ty_variable_decl) => Ok(ty_variable_decl.as_ref()),
                                _ => bail!(internal_error("`Statement::Let` must correspond to a `TyDecl::VariableDecl`.")),
                            },
                            _ => bail!(internal_error("`Statement::Let` must correspond to a `TyAstNodeContent::Declaration`.")),
                        }
                    ).transpose()?;
                    Self::visit_statement_let(ctx, statement_let, ty_var_decl, visitor, output)?;
                }
                Statement::Item(annotated) => {
                    // TODO: Implement visiting `annotations`.
                    match __ref([annotated.value]) {
                        ItemKind::Use(item_use) => {
                            let ty_use = ty_node.map(|ty_node|
                                match &ty_node.content {
                                    TyAstNodeContent::SideEffect(ty_side_effect) => match &ty_side_effect.side_effect {
                                        TySideEffectVariant::UseStatement(ty_use) => Ok(ty_use),
                                        _ => bail!(internal_error("`ItemKind::Use` must correspond to a `TySideEffectVariant::UseStatement`.")),
                                    },
                                    _ => bail!(internal_error("`ItemKind::Use` must correspond to a `TyAstNodeContent::SideEffect`.")),
                                }
                            ).transpose()?;

                            visitor.visit_use(ctx, item_use, ty_use, output)?;
                        }
                        _ => {
                            // TODO: Implement visiting `nested items`.
                        }
                    }
                }
                Statement::Expr { expr, .. } => {
                    let ty_expr = ty_node.map(|ty_node|
                        match &ty_node.content {
                            TyAstNodeContent::Expression(ty_expr) => Ok(ty_expr),
                            _ => bail!(internal_error("`Statement::Expr` must correspond to a `TyAstNodeContent::Expression`.")),
                        }
                    ).transpose()?;

                    Self::visit_expr(ctx, expr, ty_expr, visitor, output)?;
                }
                Statement::Error(..) => {
                    bail!(internal_error("`Statement::Error` cannot happen, because `forc migrate` analyzes only successfully compiled programs."));
                }
            }
        }

        if let Some(final_expr) = __ref([lexed_block.final_expr_opt]) {
            let ty_final_expr = ty_block.map(|ty_block|
                match &ty_block.contents.last() {
                    Some(ty_node) => match &ty_node.content {
                        TyAstNodeContent::Expression(ty_expression) => Ok(ty_expression),
                        _ => bail!(internal_error("Last node in the typed block must be an expression, because the lexed block ends in implicit return.")),
                    },
                    None => bail!(internal_error("Typed block must have content, because the lexed block ends in implicit return.")),
                }
            ).transpose()?;

            Self::visit_expr(ctx, final_expr.__as_ref(), ty_final_expr, visitor, output)?;
        }

        Ok(())
    }

    fn visit_statement_let<V, O>(
        ctx: &VisitingContext,
        lexed_let: __ref_type([StatementLet]),
        ty_var_decl: Option<&TyVariableDecl>,
        visitor: &mut V,
        output: &mut Vec<O>,
    ) -> Result<()>
    where
        V: __TreesVisitor<O>,
    {
        let ty_var_decl = match visitor.visit_statement_let(ctx, lexed_let, ty_var_decl, output)? {
            InvalidateTypedElement::Yes => None,
            InvalidateTypedElement::No => ty_var_decl,
        };

        let ty_expr = ty_var_decl.map(|ty_var_decl| &ty_var_decl.body);
        Self::visit_expr(ctx, __ref([lexed_let.expr]), ty_expr, visitor, output)?;

        Ok(())
    }

    fn visit_binary_op<V, O>(
        ctx: &VisitingContext,
        op: &'static str,
        lexed_lhs: __ref_type([Expr]),
        lexed_rhs: __ref_type([Expr]),
        visitor: &mut V,
        output: &mut Vec<O>,
    ) -> Result<()>
    where
        V: __TreesVisitor<O>,
    {
        // TODO: Implement getting typed LHS and RHS when visiting operands' expressions.
        //       We need to properly handle the desugaring.
        //       E.g., `x + func(1, 2);`
        //       will be desugared into `add(x, func(1, 2));`
        //       When visiting the operands in the lexed tree, in the typed tree
        //       we need to skip the operator method call, like `add` in the above example,
        //       and provide the typed arguments instead.
        let ty_lhs = None;
        let ty_rhs = None;

        match visitor.visit_binary_op(ctx, op, lexed_lhs, ty_lhs, lexed_rhs, ty_rhs, output)? {
            InvalidateTypedElement::No => (ty_lhs, ty_rhs),
            InvalidateTypedElement::Yes => (None, None),
        };

        Self::visit_expr(ctx, lexed_lhs, ty_lhs, visitor, output)?;
        Self::visit_expr(ctx, lexed_rhs, ty_rhs, visitor, output)?;

        Ok(())
    }

    fn visit_expr<V, O>(
        ctx: &VisitingContext,
        lexed_expr: __ref_type([Expr]),
        ty_expr: Option<&TyExpression>,
        visitor: &mut V,
        output: &mut Vec<O>,
    ) -> Result<()>
    where
        V: __TreesVisitor<O>,
    {
        // We visit the whole expression first.
        // If `ty_expr` is an `ImplicitReturn`, we visit is as such.
        let ty_expr = match visitor.visit_expr(ctx, lexed_expr, ty_expr, output)? {
            InvalidateTypedElement::Yes => None,
            InvalidateTypedElement::No => ty_expr,
        };

        // Afterwards, since `ImplicitReturn` as a wrapper does not exist
        // in the lexed tree, when recursing into the expression, we skip
        // the `ImplicitReturn` wrapper and visit the wrapped typed expression.
        let ty_expr = if let Some(ty_expr) = ty_expr {
            match &ty_expr.expression {
                ty::TyExpressionVariant::ImplicitReturn(exp) => Some(exp.as_ref()),
                _ => Some(ty_expr),
            }
        } else {
            None
        };

        match lexed_expr {
            Expr::Error(..) => {
                bail!(internal_error("`Expr::Error` cannot happen, because `forc migrate` analyzes only successfully compiled programs."));
            }
            Expr::Path(_path_expr) => {}
            Expr::Literal(_literal) => {}
            Expr::AbiCast { args, .. } => {
                let ty_abi_cast_expr = ty_expr
                    .map(|ty_expr| match &ty_expr.expression {
                        ty::TyExpressionVariant::AbiCast { address, .. } => Ok(address.as_ref()),
                        _ => bail!(invalid_ty_expression_variant("AbiCast", "AbiCast")),
                    })
                    .transpose()?;

                Self::visit_expr(
                    ctx,
                    __ref([args.inner.address]),
                    ty_abi_cast_expr,
                    visitor,
                    output,
                )?;
            }
            Expr::Struct { path: _, fields } => {
                for (_colon_token, field_init_expr) in fields
                    .inner
                    .__iter()
                    .filter_map(|field| field.expr_opt.__as_ref())
                {
                    let ty_field_init_expr = ty_expr.map(|ty_expr|
                        match &ty_expr.expression {
                            ty::TyExpressionVariant::StructExpression { fields, .. } => {
                                fields.iter()
                                    .find(|field| field.value.span == field_init_expr.span())
                                    .ok_or_else(|| anyhow::anyhow!(internal_error("Typed field initialization must exist, because the lexed initialization exists.")))
                            },
                            _ => bail!(invalid_ty_expression_variant("StructExpression", "Struct")),
                        }
                    )
                    .transpose()?
                    .map(|field| &field.value);

                    Self::visit_expr(
                        ctx,
                        field_init_expr.__as_ref(),
                        ty_field_init_expr,
                        visitor,
                        output,
                    )?;
                }
            }
            Expr::Tuple(_parens) => {
                // TODO: Implement visiting `tuple`.
            }
            Expr::Parens(parens) => {
                Self::visit_expr(ctx, parens.inner.__as_ref(), ty_expr, visitor, output)?;
            }
            Expr::Block(braces) => {
                let ty_block = ty_expr
                    .map(|ty_expr| match &ty_expr.expression {
                        ty::TyExpressionVariant::CodeBlock(ty_block) => Ok(ty_block),
                        _ => bail!(invalid_ty_expression_variant("CodeBlock", "Block")),
                    })
                    .transpose()?;

                Self::visit_block(ctx, __ref([braces.inner]), ty_block, visitor, output)?;
            }
            Expr::Array(_square_brackets) => {
                // TODO: Implement visiting `array`.
            }
            Expr::Asm(_asm_block) => {
                // TODO: Implement visiting `asm_block`.
            }
            Expr::Return { expr_opt, .. } => {
                if let Some(lexed_returned) = expr_opt {
                    let ty_returned = ty_expr
                        .map(|ty_expr| match &ty_expr.expression {
                            ty::TyExpressionVariant::Return(ty_returned) => {
                                Ok(ty_returned.as_ref())
                            }
                            _ => bail!(invalid_ty_expression_variant("Return", "Return")),
                        })
                        .transpose()?;

                    Self::visit_expr(ctx, lexed_returned.__as_ref(), ty_returned, visitor, output)?;
                }
            }
            Expr::Panic { expr_opt: _, .. } => {
                // TODO: Implement visiting `panic`.
            }
            Expr::If(if_expr) => {
                Self::visit_if(ctx, if_expr, ty_expr, visitor, output)?;
            }
            Expr::Match {
                match_token: _,
                value: _,
                branches: _,
            } => {
                // TODO: Implement visiting `match`.
            }
            Expr::While {
                while_token: _,
                condition,
                block,
            } => {
                let ty_while = ty_expr
                    .map(|ty_expr| match &ty_expr.expression {
                        ty::TyExpressionVariant::WhileLoop { condition, body } => {
                            Ok((condition.as_ref(), body))
                        }
                        _ => bail!(invalid_ty_expression_variant("WhileLoop", "While")),
                    })
                    .transpose()?;

                let ty_while_condition = ty_while.map(|ty_while| ty_while.0);
                let ty_while_block = ty_while.map(|ty_while| ty_while.1);

                Self::visit_expr(
                    ctx,
                    condition.__as_ref(),
                    ty_while_condition,
                    visitor,
                    output,
                )?;
                Self::visit_block(ctx, __ref([block.inner]), ty_while_block, visitor, output)?;
            }
            Expr::For {
                for_token: _,
                in_token: _,
                value_pattern: _,
                iterator: _,
                block: _,
            } => {
                // TODO: Implement visiting `for`.
            }
            Expr::FuncApp { func: _, args: _ } => {
                let ty_expr = match visitor.visit_fn_call(ctx, lexed_expr, ty_expr, output)? {
                    InvalidateTypedElement::Yes => None,
                    InvalidateTypedElement::No => ty_expr,
                };

                Self::visit_args(ctx, lexed_expr, ty_expr, false, visitor, output)?;
            }
            Expr::Index { target, arg } => {
                // TODO: Implement extracting typed elements for `array[index]`.
                let ty_target = None;
                let ty_arg = None;

                Self::visit_expr(ctx, target.__as_ref(), ty_target, visitor, output)?;
                Self::visit_expr(ctx, arg.inner.__as_ref(), ty_arg, visitor, output)?;
            }
            Expr::MethodCall {
                target: _,
                dot_token: _,
                path_seg: _,
                contract_args_opt: _,
                args: _,
            } => {
                // TODO: Implement visiting `method call target`.
                //       In the `ty_expr` this is the first argument in `arguments`.

                let ty_expr = match visitor.visit_method_call(ctx, lexed_expr, ty_expr, output)? {
                    InvalidateTypedElement::Yes => None,
                    InvalidateTypedElement::No => ty_expr,
                };

                Self::visit_args(ctx, lexed_expr, ty_expr, true, visitor, output)?;
            }
            Expr::FieldProjection {
                target,
                dot_token: _,
                name: _,
            } => {
                // TODO: Implement extracting typed target for `struct.field`.
                let ty_target = None;

                Self::visit_expr(ctx, target.__as_ref(), ty_target, visitor, output)?;
            }
            Expr::TupleFieldProjection {
                target,
                dot_token: _,
                field: _,
                field_span: _,
            } => {
                // TODO: Implement extracting typed target for `tuple.N`.
                let ty_target = None;

                Self::visit_expr(ctx, target.__as_ref(), ty_target, visitor, output)?;
            }
            Expr::Ref {
                ampersand_token: _,
                mut_token: _,
                expr: _,
            } => {
                // TODO: Implement visiting `ref`.
            }
            Expr::Deref {
                star_token: _,
                expr: _,
            } => {
                // TODO: Implement visiting `deref`.
            }
            Expr::Not {
                bang_token: _,
                expr,
            } => {
                // TODO: Implement extracting typed expressions when visiting `not`.
                let ty_expr = None;
                Self::visit_expr(ctx, expr.__as_ref(), ty_expr, visitor, output)?;
            }
            Expr::Mul {
                lhs,
                star_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <StarToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::Div {
                lhs,
                forward_slash_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <ForwardSlashToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::Pow {
                lhs,
                double_star_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <DoubleStarToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::Modulo {
                lhs,
                percent_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <PercentToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::Add {
                lhs,
                add_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <AddToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::Sub {
                lhs,
                sub_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <SubToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::Shl {
                lhs,
                shl_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <ShlToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::Shr {
                lhs,
                shr_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <ShrToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::BitAnd {
                lhs,
                ampersand_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <AmpersandToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::BitXor {
                lhs,
                caret_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <CaretToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::BitOr {
                lhs,
                pipe_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <PipeToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::Equal {
                lhs,
                double_eq_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <DoubleEqToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::NotEqual {
                lhs,
                bang_eq_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <BangEqToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::LessThan {
                lhs,
                less_than_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <LessThanToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::GreaterThan {
                lhs,
                greater_than_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <GreaterThanToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::LessThanEq {
                lhs,
                less_than_eq_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <LessThanEqToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::GreaterThanEq {
                lhs,
                greater_than_eq_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <GreaterThanEqToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::LogicalAnd {
                lhs,
                double_ampersand_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <DoubleAmpersandToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::LogicalOr {
                lhs,
                double_pipe_token: _,
                rhs,
            } => {
                Self::visit_binary_op(
                    ctx,
                    <DoublePipeToken as Token>::AS_STR,
                    lhs.__as_ref(),
                    rhs.__as_ref(),
                    visitor,
                    output,
                )?;
            }
            Expr::Reassignment {
                assignable,
                reassignment_op,
                expr,
            } => {
                let ty_reassignment = ty_expr
                    .map(|ty_expr| match &ty_expr.expression {
                        ty::TyExpressionVariant::Reassignment(ty_reassignment) => {
                            Ok(ty_reassignment.as_ref())
                        }
                        _ => bail!(invalid_ty_expression_variant(
                            "Reassignment",
                            "Reassignment"
                        )),
                    })
                    .transpose()?;
                let ty_lhs = ty_reassignment.map(|ty_reassignment| &ty_reassignment.lhs);
                let ty_rhs = ty_reassignment.map(|ty_reassignment| &ty_reassignment.rhs);

                let (_ty_lhs, ty_rhs) = match visitor.visit_reassignment(
                    ctx,
                    reassignment_op,
                    assignable,
                    ty_lhs,
                    expr.__as_ref(),
                    ty_rhs,
                    output,
                )? {
                    InvalidateTypedElement::Yes => (None, None),
                    InvalidateTypedElement::No => (ty_lhs, ty_rhs),
                };

                match reassignment_op.variant {
                    ReassignmentOpVariant::Equals => {
                        // TODO: Implement visiting expressions in the reassignment LHS.
                        Self::visit_expr(ctx, expr, ty_rhs, visitor, output)?;
                    }
                    _ => {
                        // TODO: Implement getting `ty_expr` when visiting `compound reassignments`.
                        //       We need to properly handle the desugaring.
                        //       E.g., `x += func(1, 2);`
                        //       will be desugared into `x = add(x, func(1, 2));`
                        //       When visiting the RHS in the lexed tree, we need to skip the
                        //       operator method call in the typed tree, and provide the
                        //       typed arguments instead.
                        //       To provide visiting without losing the information about compound
                        //       reassignment, we will need to have a dedicated `visit_reassignment`
                        //       method.
                        // TODO: Implement visiting expressions in the reassignment LHS.
                        Self::visit_expr(ctx, expr, None, visitor, output)?;
                    }
                }
            }
            Expr::Break { .. } => {}
            Expr::Continue { .. } => {}
        }

        Ok(())
    }

    fn visit_if<V, O>(
        ctx: &VisitingContext,
        lexed_if: __ref_type([IfExpr]),
        ty_if_expr: Option<&TyExpression>,
        visitor: &mut V,
        output: &mut Vec<O>,
    ) -> Result<()>
    where
        V: __TreesVisitor<O>,
    {
        match __ref([lexed_if.condition]) {
            IfCondition::Expr(lexed_condition) => {
                let ty_if = ty_if_expr
                    .map(|ty_expr| match &ty_expr.expression {
                        ty::TyExpressionVariant::IfExp {
                            condition,
                            then,
                            r#else,
                        } => Ok((
                            condition.as_ref(),
                            then.as_ref(),
                            r#else.as_ref().map(|r#else| r#else.as_ref()),
                        )),
                        _ => bail!(invalid_ty_expression_variant("IfExpr", "If")),
                    })
                    .transpose()?;
                let ty_if_condition = ty_if.map(|ty_if| ty_if.0);
                let ty_if_then = ty_if
                    .map(|ty_if| match &ty_if.1.expression {
                        ty::TyExpressionVariant::CodeBlock(ty_code_block) => Ok(ty_code_block),
                        _ => bail!(invalid_ty_expression_variant(
                            "CodeBlock",
                            "CodeBlockContents"
                        )),
                    })
                    .transpose()?;
                let ty_if_else = ty_if.and_then(|ty_if| ty_if.2);

                visitor.visit_expr(ctx, lexed_condition.__as_ref(), ty_if_condition, output)?;

                Self::visit_block(
                    ctx,
                    __ref([lexed_if.then_block.inner]),
                    ty_if_then,
                    visitor,
                    output,
                )?;

                if let Some((_else_token, lexed_if_else)) = __ref([lexed_if.else_opt]) {
                    match lexed_if_else {
                        LoopControlFlow::Continue(lexed_else_if) => {
                            Self::visit_if(
                                ctx,
                                lexed_else_if.__as_ref(),
                                ty_if_else,
                                visitor,
                                output,
                            )?;
                        }
                        LoopControlFlow::Break(lexed_else_block) => {
                            let ty_if_else = ty_if_else
                                .map(|ty_if_else| match &ty_if_else.expression {
                                    ty::TyExpressionVariant::CodeBlock(ty_code_block) => {
                                        Ok(ty_code_block)
                                    }
                                    _ => bail!(invalid_ty_expression_variant(
                                        "CodeBlock",
                                        "CodeBlockContents"
                                    )),
                                })
                                .transpose()?;
                            Self::visit_block(
                                ctx,
                                __ref([lexed_else_block.inner]),
                                ty_if_else,
                                visitor,
                                output,
                            )?;
                        }
                    }
                }
            }
            IfCondition::Let {
                let_token: _,
                lhs: _,
                eq_token: _,
                rhs: _,
            } => {
                // TODO: Implement visiting `if let`.
                //       Similar to `match` expression, we have a complex
                //       desugaring here and we need to properly locate the
                //       corresponding typed elements.
            }
        }

        Ok(())
    }

    fn visit_args<V, O>(
        ctx: &VisitingContext,
        lexed_expr: __ref_type([Expr]),
        ty_expr: Option<&TyExpression>,
        is_method_call: bool,
        visitor: &mut V,
        output: &mut Vec<O>,
    ) -> Result<()>
    where
        V: __TreesVisitor<O>,
    {
        let ty_args = ty_expr.map(|ty_expr|
            match &ty_expr.expression {
                ty::TyExpressionVariant::FunctionApplication { arguments, .. } => Ok(arguments.iter().map(|(_ident, ty_arg)| ty_arg).collect::<Vec<_>>()),
                ty::TyExpressionVariant::IntrinsicFunction(TyIntrinsicFunctionKind { arguments, .. }) => Ok(arguments.iter().collect::<Vec<_>>()),
                ty::TyExpressionVariant::EnumInstantiation { contents, .. } => Ok(contents.as_ref().map_or(vec![], |arg| vec![arg.as_ref()])),
                _ => bail!(internal_error("Arguments can be visited only on a `ty_expr` of the following `TyExpressionVariant`s: `FunctionApplication`, `IntrinsicFunction`, `EnumInstantiation`.")),
            }
        ).transpose()?;

        let lexed_args = match lexed_expr {
            Expr::FuncApp { args, .. }
            | Expr::MethodCall { args, .. } => args,
            _ => bail!("Arguments can be visited only on a `lexed_expr` of the following `Expr`s: `FuncApp`, `MethodCall`."),
        };

        if let Some(ty_args) = &ty_args {
            let lexed_args_count = lexed_args.inner.iter().count();
            // Ignore the first argument in the typed arguments, which is the `self` argument.
            let ty_args_count = if is_method_call {
                ty_args.len() - 1
            } else {
                ty_args.len()
            };
            if lexed_args_count != ty_args_count {
                bail!(internal_error(format!("Number of arguments in the `lexed_expr` ({lexed_args_count}) must be the same as in the `ty_expr` ({ty_args_count}).")));
            }
        }

        let index_shift = if ty_args.is_some() && is_method_call {
            1
        } else {
            0
        };

        for (i, lexed_arg) in lexed_args.inner.__iter().enumerate() {
            let ty_arg = ty_args.as_ref().map(|ty_args| ty_args[i + index_shift]);
            Self::visit_expr(ctx, lexed_arg, ty_arg, visitor, output)?;
        }

        Ok(())
    }
}

pub(crate) fn invalid_ty_expression_variant(expected_variant: &str, lexed_expr: &str) -> String {
    internal_error(
        format!("`TyExpressionVariant` must be `{expected_variant}`, because the lexed `Expr` was `{lexed_expr}`.")
    )
}

#[duplicate_item(
    __LexedFnCallInfo      __ref_type(type);
    [LexedFnCallInfo]      [&'a type];
    [LexedFnCallInfoMut]   [&'a mut type];
)]
pub(crate) struct __LexedFnCallInfo<'a> {
    pub func: __ref_type([Expr]),
    pub args: Vec<__ref_type([Expr])>,
}

#[duplicate_item(
    __LexedFnCallInfo      __ref_type(type)    __iter      __as_ref;
    [LexedFnCallInfo]      [&'a type]          [iter]      [as_ref];
    [LexedFnCallInfoMut]   [&'a mut type]      [iter_mut]  [as_mut];
)]
impl<'a> __LexedFnCallInfo<'a> {
    pub fn new(lexed_fn_call: __ref_type([Expr])) -> Result<Self> {
        let lexed_fn_call = match lexed_fn_call {
            Expr::FuncApp { func, args } => Ok((func, args)),
            _ => bail!(internal_error(
                "`lexed_fn_call` must be of variant `Expr::FuncApp`."
            )),
        }?;

        Ok(Self {
            func: lexed_fn_call.0.__as_ref(),
            args: lexed_fn_call.1.inner.__iter().collect(),
        })
    }
}

pub(crate) struct TyFnCallInfo<'a> {
    pub call_path: &'a CallPath,
    pub arguments: &'a Vec<(Ident, TyExpression)>,
    pub fn_decl: Arc<TyFunctionDecl>,
}

impl<'a> TyFnCallInfo<'a> {
    pub fn new(decl_engine: &DeclEngine, ty_fn_call: &'a TyExpression) -> Result<Self> {
        let ty_fn_call = match &ty_fn_call.expression {
            TyExpressionVariant::FunctionApplication {
                call_path,
                arguments,
                fn_ref,
                ..
            } => Ok((call_path, arguments, fn_ref)),
            _ => bail!(internal_error(
                "`ty_fn_call` must be of variant `TyExpressionVariant::FunctionApplication`."
            )),
        }?;

        let fn_decl = decl_engine.get_function(ty_fn_call.2.id());

        Ok(Self {
            call_path: ty_fn_call.0,
            arguments: ty_fn_call.1,
            fn_decl,
        })
    }
}

#[duplicate_item(
    __LexedMethodCallInfo      __ref_type(type);
    [LexedMethodCallInfo]      [&'a type];
    [LexedMethodCallInfoMut]   [&'a mut type];
)]
pub(crate) struct __LexedMethodCallInfo<'a> {
    pub target: __ref_type([Expr]),
    pub path_seg: __ref_type([PathExprSegment]),
    pub args: Vec<__ref_type([Expr])>,
}

#[duplicate_item(
    __LexedMethodCallInfo      __ref_type(type)    __iter      __as_ref;
    [LexedMethodCallInfo]      [&'a type]          [iter]      [as_ref];
    [LexedMethodCallInfoMut]   [&'a mut type]      [iter_mut]  [as_mut];
)]
impl<'a> __LexedMethodCallInfo<'a> {
    pub fn new(lexed_method_call: __ref_type([Expr])) -> Result<Self> {
        let lexed_method_call = match lexed_method_call {
            Expr::MethodCall {
                target,
                path_seg,
                args,
                ..
            } => Ok((target, path_seg, args)),
            _ => bail!(internal_error(
                "`lexed_method_call` must be of variant `Expr::MethodCall`."
            )),
        }?;

        Ok(Self {
            target: lexed_method_call.0.__as_ref(),
            path_seg: lexed_method_call.1,
            args: lexed_method_call.2.inner.__iter().collect(),
        })
    }
}

pub(crate) struct TyMethodCallInfo<'a> {
    pub call_path: &'a CallPath,
    pub arguments: &'a Vec<(Ident, TyExpression)>,
    pub fn_decl: Arc<TyFunctionDecl>,
    pub parent_type_id: TypeId,
}

impl<'a> TyMethodCallInfo<'a> {
    pub fn new(decl_engine: &DeclEngine, ty_method_call: &'a TyExpression) -> Result<Self> {
        let ty_method_call = match &ty_method_call.expression {
            TyExpressionVariant::FunctionApplication {
                call_path,
                arguments,
                fn_ref,
                call_path_typeid,
                ..
            } => Ok((call_path, arguments, fn_ref, call_path_typeid)),
            _ => bail!(internal_error(
                "`ty_method_call` must be of variant `TyExpressionVariant::FunctionApplication`."
            )),
        }?;

        let fn_decl = decl_engine.get_function(ty_method_call.2.id());
        let Some(parent_type_id) = ty_method_call.3 else {
            bail!(internal_error("`TyExpressionVariant::FunctionApplication` is a method call and must have `call_path_typeid`."));
        };

        Ok(Self {
            call_path: ty_method_call.0,
            arguments: ty_method_call.1,
            fn_decl,
            parent_type_id: *parent_type_id,
        })
    }
}
