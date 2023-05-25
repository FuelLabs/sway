pub mod code_block;
pub mod declaration;
pub mod expression;
pub mod mode;

pub(crate) use expression::*;
pub(crate) use mode::*;

use crate::{
    error::*,
    language::{parsed::*, ty},
    semantic_analysis::*,
    type_system::*,
    types::DeterministicallyAborts,
    Ident,
};

use sway_error::warning::Warning;
use sway_types::{span::Span, Spanned};

impl ty::TyAstNode {
    pub(crate) fn type_check(ctx: TypeCheckContext, node: AstNode) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;
        let engines = ctx.engines();

        let node = ty::TyAstNode {
            content: match node.content.clone() {
                AstNodeContent::UseStatement(a) => {
                    let path = if a.is_absolute {
                        a.call_path.clone()
                    } else {
                        ctx.namespace.find_module_path(&a.call_path)
                    };
                    let mut res = match a.import_type {
                        ImportType::Star => {
                            // try a standard starimport first
                            let import = ctx.namespace.star_import(&path, engines);
                            if import.is_ok() {
                                import
                            } else {
                                // if it doesn't work it could be an enum star import
                                if let Some((enum_name, path)) = path.split_last() {
                                    let variant_import =
                                        ctx.namespace.variant_star_import(path, engines, enum_name);
                                    if variant_import.is_ok() {
                                        variant_import
                                    } else {
                                        import
                                    }
                                } else {
                                    import
                                }
                            }
                        }
                        ImportType::SelfImport(_) => {
                            ctx.namespace.self_import(engines, &path, a.alias.clone())
                        }
                        ImportType::Item(ref s) => {
                            // try a standard item import first
                            let import =
                                ctx.namespace
                                    .item_import(engines, &path, s, a.alias.clone());

                            if import.is_ok() {
                                import
                            } else {
                                // if it doesn't work it could be an enum variant import
                                if let Some((enum_name, path)) = path.split_last() {
                                    let variant_import = ctx.namespace.variant_import(
                                        engines,
                                        path,
                                        enum_name,
                                        s,
                                        a.alias.clone(),
                                    );
                                    if variant_import.is_ok() {
                                        variant_import
                                    } else {
                                        import
                                    }
                                } else {
                                    import
                                }
                            }
                        }
                    };
                    warnings.append(&mut res.warnings);
                    errors.append(&mut res.errors);
                    ty::TyAstNodeContent::SideEffect(ty::TySideEffect {
                        side_effect: ty::TySideEffectVariant::UseStatement(ty::TyUseStatement {
                            alias: a.alias,
                            call_path: a.call_path,
                            is_absolute: a.is_absolute,
                            import_type: a.import_type,
                        }),
                    })
                }
                AstNodeContent::IncludeStatement(_) => {
                    ty::TyAstNodeContent::SideEffect(ty::TySideEffect {
                        side_effect: ty::TySideEffectVariant::IncludeStatement,
                    })
                }
                AstNodeContent::Declaration(decl) => ty::TyAstNodeContent::Declaration(check!(
                    ty::TyDecl::type_check(ctx, decl),
                    return err(warnings, errors),
                    warnings,
                    errors
                )),
                AstNodeContent::Expression(expr) => {
                    let ctx = ctx
                        .with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown))
                        .with_help_text("");
                    let inner = check!(
                        ty::TyExpression::type_check(ctx, expr.clone()),
                        ty::TyExpression::error(expr.span(), engines),
                        warnings,
                        errors
                    );
                    ty::TyAstNodeContent::Expression(inner)
                }
                AstNodeContent::ImplicitReturnExpression(expr) => {
                    let ctx =
                        ctx.with_help_text("Implicit return must match up with block's type.");
                    let typed_expr = check!(
                        ty::TyExpression::type_check(ctx, expr.clone()),
                        ty::TyExpression::error(expr.span(), engines),
                        warnings,
                        errors
                    );
                    ty::TyAstNodeContent::ImplicitReturnExpression(typed_expr)
                }
            },
            span: node.span,
        };

        if let ty::TyAstNode {
            content: ty::TyAstNodeContent::Expression(ty::TyExpression { .. }),
            ..
        } = node
        {
            let warning = Warning::UnusedReturnValue {
                r#type: engines.help_out(node.type_info(type_engine)).to_string(),
            };
            assert_or_warn!(
                node.type_info(type_engine)
                    .can_safely_ignore(type_engine, decl_engine),
                warnings,
                node.span.clone(),
                warning
            );
        }

        ok(node, warnings, errors)
    }
}
