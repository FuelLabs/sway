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
    types::DeterministicallyAborts,
    Ident,
};

use sway_error::{
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};
use sway_types::{span::Span, Spanned};

impl ty::TyAstNode {
    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        node: AstNode,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        let node = ty::TyAstNode {
            content: match node.content.clone() {
                AstNodeContent::UseStatement(a) => {
                    // dbg!(&a);
                    let mut is_external = false;
                    // dbg!(is_external, &ctx.namespace.name);
                    if let Some(submodule) = ctx.namespace.submodule(&[a.call_path[0].clone()]) {
                        is_external = submodule.is_external;
                    }
                    // dbg!(is_external, &ctx.namespace.name);
                    let path = if is_external || a.is_absolute {
                        a.call_path.clone()
                    } else {
                        ctx.namespace.find_module_path(&a.call_path)
                    };
                    // dbg!(&path);
                    let _ = match a.import_type {
                        ImportType::Star => {
                            // try a standard starimport first
                            let star_import_handler = Handler::default();
                            // dbg!(&ctx.namespace.name);
                            let import =
                                ctx.star_import(&star_import_handler, &path, a.is_absolute);

                            if import.is_ok() {
                                handler.append(star_import_handler);
                                import
                            } else {
                                // if it doesn't work it could be an enum star import
                                if let Some((enum_name, path)) = path.split_last() {
                                    let variant_import_handler = Handler::default();
                                    let variant_import = ctx.variant_star_import(
                                        &variant_import_handler,
                                        path,
                                        enum_name,
                                        a.is_absolute,
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
                            }
                        }
                        ImportType::SelfImport(_) => {
                            ctx.self_import(handler, &path, a.alias.clone(), a.is_absolute)
                        }
                        ImportType::Item(ref s) => {
                            // try a standard item import first
                            let item_import_handler = Handler::default();
                            let import = ctx.item_import(
                                &item_import_handler,
                                &path,
                                s,
                                a.alias.clone(),
                                a.is_absolute,
                            );

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
                                        a.alias.clone(),
                                        a.is_absolute,
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
                    ty::TyAstNodeContent::SideEffect(ty::TySideEffect {
                        side_effect: ty::TySideEffectVariant::UseStatement(ty::TyUseStatement {
                            alias: a.alias,
                            call_path: a.call_path,
                            span: a.span,
                            is_absolute: a.is_absolute,
                            import_type: a.import_type,
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
                    let ctx = ctx
                        .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None))
                        .with_help_text("");
                    let inner = ty::TyExpression::type_check(handler, ctx, expr.clone())
                        .unwrap_or_else(|err| ty::TyExpression::error(err, expr.span(), engines));
                    ty::TyAstNodeContent::Expression(inner)
                }
                AstNodeContent::ImplicitReturnExpression(expr) => {
                    let ctx =
                        ctx.with_help_text("Implicit return must match up with block's type.");
                    let typed_expr = ty::TyExpression::type_check(handler, ctx, expr.clone())
                        .unwrap_or_else(|err| ty::TyExpression::error(err, expr.span(), engines));
                    ty::TyAstNodeContent::ImplicitReturnExpression(typed_expr)
                }
                AstNodeContent::Error(spans, err) => ty::TyAstNodeContent::Error(spans, err),
            },
            span: node.span,
        };

        if let ty::TyAstNode {
            content: ty::TyAstNodeContent::Expression(ty::TyExpression { .. }),
            ..
        } = node
        {
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

        Ok(node)
    }
}
