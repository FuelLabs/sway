pub mod code_block;
pub mod declaration;
pub mod expression;
pub mod modes;

pub(crate) use expression::*;
pub(crate) use modes::*;

use crate::{
    language::{parsed::*, ty},
    semantic_analysis::*,
    type_system::*,
    Engines, Ident,
};

use sway_error::{
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};
use sway_types::{span::Span, Spanned};

use super::symbol_collection_context::SymbolCollectionContext;

impl ty::TyAstNode {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        node: &AstNode,
    ) -> Result<(), ErrorEmitted> {
        match node.content.clone() {
            AstNodeContent::UseStatement(stmt) => {
                collect_use_statement(handler, engines, ctx, &stmt);
            }
            AstNodeContent::ModStatement(_i) => (),
            AstNodeContent::Declaration(decl) => ty::TyDecl::collect(handler, engines, ctx, decl)?,
            AstNodeContent::Expression(expr) => {
                ty::TyExpression::collect(handler, engines, ctx, &expr)?
            }
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
                    handle_use_statement(&mut ctx, &stmt, handler);
                    ty::TyAstNodeContent::Statement(ty::TyStatement::Use(ty::TyUseStatement {
                        alias: stmt.alias,
                        call_path: stmt.call_path,
                        span: stmt.span,
                        is_relative_to_package_root: stmt.is_relative_to_package_root,
                        import_type: stmt.import_type,
                    }))
                }
                AstNodeContent::ModStatement(i) => ty::TyAstNodeContent::Statement(
                    ty::TyStatement::Mod(ty::TyModStatement {
                        mod_name: i.mod_name,
                        span: i.span,
                        visibility: i.visibility,
                    }),
                ),
                AstNodeContent::Declaration(decl) => ty::TyAstNodeContent::Declaration(
                    ty::TyDecl::type_check(handler, &mut ctx, decl)?,
                ),
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
                                .with_type_annotation(type_engine.new_unknown());
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

fn collect_use_statement(
    handler: &Handler,
    engines: &Engines,
    ctx: &mut SymbolCollectionContext,
    stmt: &UseStatement,
) {
    let path = ctx.namespace.parsed_path_to_full_path(
        engines,
        &stmt.call_path,
        stmt.is_relative_to_package_root,
    );

    let _ = match stmt.import_type {
        ImportType::Star => {
            // try a standard starimport first
            let star_import_handler = Handler::default();
            let import = ctx.star_import(&star_import_handler, engines, &path, stmt.reexport);
            if import.is_ok() {
                handler.append(star_import_handler);
                import
            } else if path.len() >= 2 {
                // if it doesn't work it could be an enum star import
                if let Some((enum_name, path)) = path.split_last() {
                    let variant_import_handler = Handler::default();
                    let variant_import = ctx.variant_star_import(
                        &variant_import_handler,
                        engines,
                        path,
                        enum_name,
                        stmt.reexport,
                    );
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
            } else {
                handler.append(star_import_handler);
                import
            }
        }
        ImportType::SelfImport(_) => {
            ctx.self_import(handler, engines, &path, stmt.alias.clone(), stmt.reexport)
        }
        ImportType::Item(ref s) => {
            // try a standard item import first
            let item_import_handler = Handler::default();
            let import = ctx.item_import(
                &item_import_handler,
                engines,
                &path,
                s,
                stmt.alias.clone(),
                stmt.reexport,
            );

            if import.is_ok() {
                handler.append(item_import_handler);
                import
            } else if path.len() >= 2 {
                // if it doesn't work it could be an enum variant import
                // For this to work the path must have at least 2 elements: The current package name and the enum name
                if let Some((enum_name, path)) = path.split_last() {
                    let variant_import_handler = Handler::default();
                    let variant_import = ctx.variant_import(
                        &variant_import_handler,
                        engines,
                        path,
                        enum_name,
                        s,
                        stmt.alias.clone(),
                        stmt.reexport,
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
            } else {
                handler.append(item_import_handler);
                import
            }
        }
    };
}

// To be removed once TypeCheckContext is ported to use SymbolCollectionContext.
fn handle_use_statement(ctx: &mut TypeCheckContext<'_>, stmt: &UseStatement, handler: &Handler) {
    let path = ctx.namespace.parsed_path_to_full_path(
        ctx.engines,
        &stmt.call_path,
        stmt.is_relative_to_package_root,
    );
    let _ = match stmt.import_type {
        ImportType::Star => {
            // try a standard starimport first
            let star_import_handler = Handler::default();
            let import = ctx.star_import(&star_import_handler, &path, stmt.reexport);
            if import.is_ok() {
                handler.append(star_import_handler);
                import
            } else if path.len() >= 2 {
                // if it doesn't work it could be an enum star import
                if let Some((enum_name, path)) = path.split_last() {
                    let variant_import_handler = Handler::default();
                    let variant_import = ctx.variant_star_import(
                        &variant_import_handler,
                        path,
                        enum_name,
                        stmt.reexport,
                    );
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
            } else {
                handler.append(star_import_handler);
                import
            }
        }
        ImportType::SelfImport(_) => {
            ctx.self_import(handler, &path, stmt.alias.clone(), stmt.reexport)
        }
        ImportType::Item(ref s) => {
            // try a standard item import first
            let item_import_handler = Handler::default();
            let import = ctx.item_import(
                &item_import_handler,
                &path,
                s,
                stmt.alias.clone(),
                stmt.reexport,
            );

            if import.is_ok() {
                handler.append(item_import_handler);
                import
            } else if path.len() >= 2 {
                // if it doesn't work it could be an enum variant import
                // For this to work the path must have at least 2 elements: The current package name and the enum name.
                if let Some((enum_name, path)) = path.split_last() {
                    let variant_import_handler = Handler::default();
                    let variant_import = ctx.variant_import(
                        &variant_import_handler,
                        path,
                        enum_name,
                        s,
                        stmt.alias.clone(),
                        stmt.reexport,
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
            } else {
                handler.append(item_import_handler);
                import
            }
        }
    };
}
