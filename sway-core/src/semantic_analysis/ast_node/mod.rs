pub mod code_block;
pub mod declaration;
pub mod expression;
pub mod modes;

pub(crate) use expression::*;
pub(crate) use modes::*;

use crate::{
    engine_threading::SpannedWithEngines,
    language::{
        parsed::*,
        ty::{self, TyDecl},
    },
    namespace::TraitMap,
    semantic_analysis::*,
    type_system::*,
    Engines, Ident,
};

use sway_error::{
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};
use sway_types::{span::Span, Spanned};

use super::collection_context::SymbolCollectionContext;

impl ty::TyAstNode {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        node: &AstNode,
    ) -> Result<(), ErrorEmitted> {
        match node.content.clone() {
            AstNodeContent::UseStatement(_stmt) => {}
            AstNodeContent::IncludeStatement(_i) => (),
            AstNodeContent::Declaration(decl) => ty::TyDecl::collect(handler, engines, ctx, decl)?,
            AstNodeContent::Expression(_expr) => (),
            AstNodeContent::Error(_spans, _err) => (),
        };

        Ok(())
    }

    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        node: &AstNode,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        let node = ty::TyAstNode {
            content: match node.content.clone() {
                AstNodeContent::UseStatement(stmt) => {
                    handle_use_statement(&mut ctx, engines, &stmt, handler);
                    handle_item_trait_imports(&mut ctx, engines, handler)?;
                    ty::TyAstNodeContent::SideEffect(ty::TySideEffect {
                        side_effect: ty::TySideEffectVariant::UseStatement(ty::TyUseStatement {
                            alias: stmt.alias,
                            call_path: stmt.call_path,
                            span: stmt.span,
                            is_absolute: stmt.is_absolute,
                            import_type: stmt.import_type,
                        }),
                    })
                }
                AstNodeContent::IncludeStatement(i) => {
                    ty::TyAstNodeContent::SideEffect(ty::TySideEffect {
                        side_effect: ty::TySideEffectVariant::IncludeStatement(
                            ty::TyIncludeStatement {
                                mod_name: i.mod_name,
                                span: i.span,
                                visibility: i.visibility,
                            },
                        ),
                    })
                }
                AstNodeContent::Declaration(decl) => {
                    ty::TyAstNodeContent::Declaration(ty::TyDecl::type_check(handler, ctx, decl)?)
                }
                AstNodeContent::Expression(expr) => {
                    let mut ctx = ctx;
                    match expr.kind {
                        ExpressionKind::ImplicitReturn(_) => {
                            // Do not use any type annotation with implicit returns as that
                            // will later cause type inference errors when matching implicit block
                            // types.
                        }
                        _ => {
                            ctx = ctx
                                .with_help_text("")
                                .with_type_annotation(type_engine.insert(
                                    engines,
                                    TypeInfo::Unknown,
                                    None,
                                ));
                        }
                    }
                    let inner = ty::TyExpression::type_check(handler, ctx, &expr)
                        .unwrap_or_else(|err| ty::TyExpression::error(err, expr.span(), engines));
                    ty::TyAstNodeContent::Expression(inner)
                }
                AstNodeContent::Error(spans, err) => ty::TyAstNodeContent::Error(spans, err),
            },
            span: node.span.clone(),
        };

        if let ty::TyAstNode {
            content: ty::TyAstNodeContent::Expression(ty::TyExpression { expression, .. }),
            ..
        } = &node
        {
            match expression {
                ty::TyExpressionVariant::ImplicitReturn(_) => {}
                _ => {
                    if !node
                        .type_info(type_engine)
                        .can_safely_ignore(type_engine, decl_engine)
                    {
                        handler.emit_warn(CompileWarning {
                            warning_content: Warning::UnusedReturnValue {
                                r#type: engines.help_out(node.type_info(type_engine)).to_string(),
                            },
                            span: node.span.clone(),
                        })
                    };
                }
            }
        }

        Ok(node)
    }
}

fn handle_item_trait_imports(
    ctx: &mut TypeCheckContext<'_>,
    engines: &Engines,
    handler: &Handler,
) -> Result<(), ErrorEmitted> {
    let mut impls_to_insert = TraitMap::default();

    let root_mod = &ctx.namespace().root().module;
    let dst_mod = ctx.namespace.module(engines);

    for (_, (_, src, decl)) in dst_mod.current_items().use_item_synonyms.iter() {
        let src_mod = root_mod.lookup_submodule(handler, engines, src)?;

        //  if this is an enum or struct or function, import its implementations
        if let Ok(type_id) = decl.return_type(&Handler::default(), engines) {
            impls_to_insert.extend(
                src_mod
                    .current_items()
                    .implemented_traits
                    .filter_by_type_item_import(type_id, engines),
                engines,
            );
        }
        // if this is a trait, import its implementations
        let decl_span = decl.span(engines);
        if matches!(decl, TyDecl::TraitDecl(_)) {
            // TODO: we only import local impls from the source namespace
            // this is okay for now but we'll need to device some mechanism to collect all
            // available trait impls
            impls_to_insert.extend(
                src_mod
                    .current_items()
                    .implemented_traits
                    .filter_by_trait_decl_span(decl_span),
                engines,
            );
        }
    }

    let dst_mod = ctx.namespace_mut().module_mut(engines);
    dst_mod
        .current_items_mut()
        .implemented_traits
        .extend(impls_to_insert, engines);

    Ok(())
}

fn handle_use_statement(
    ctx: &mut TypeCheckContext<'_>,
    engines: &Engines,
    stmt: &UseStatement,
    handler: &Handler,
) {
    let mut is_external = false;
    if let Some(submodule) = ctx
        .namespace()
        .module(engines)
        .submodule(engines, &[stmt.call_path[0].clone()])
    {
        is_external = submodule.read(engines, |m| m.is_external);
    }
    let path = if is_external || stmt.is_absolute {
        stmt.call_path.clone()
    } else {
        ctx.namespace().prepend_module_path(&stmt.call_path)
    };
    let _ = match stmt.import_type {
        ImportType::Star => {
            // try a standard starimport first
            let star_import_handler = Handler::default();
            let import = ctx.star_import(&star_import_handler, &path);
            if import.is_ok() {
                handler.append(star_import_handler);
                import
            } else {
                // if it doesn't work it could be an enum star import
                if let Some((enum_name, path)) = path.split_last() {
                    let variant_import_handler = Handler::default();
                    let variant_import =
                        ctx.variant_star_import(&variant_import_handler, path, enum_name);
                    if variant_import.is_ok() {
                        handler.append(variant_import_handler);
                        variant_import
                    } else {
                        handler.append(star_import_handler);
                        import
                    }
                } else {
                    handler.append(star_import_handler);
                    import
                }
            }
        }
        ImportType::SelfImport(_) => ctx.self_import(handler, &path, stmt.alias.clone()),
        ImportType::Item(ref s) => {
            // try a standard item import first
            let item_import_handler = Handler::default();
            let import = ctx.item_import(&item_import_handler, &path, s, stmt.alias.clone());

            if import.is_ok() {
                handler.append(item_import_handler);
                import
            } else {
                // if it doesn't work it could be an enum variant import
                if let Some((enum_name, path)) = path.split_last() {
                    let variant_import_handler = Handler::default();
                    let variant_import = ctx.variant_import(
                        &variant_import_handler,
                        path,
                        enum_name,
                        s,
                        stmt.alias.clone(),
                    );
                    if variant_import.is_ok() {
                        handler.append(variant_import_handler);
                        variant_import
                    } else {
                        handler.append(item_import_handler);
                        import
                    }
                } else {
                    handler.append(item_import_handler);
                    import
                }
            }
        }
    };
}
