use crate::{
    decl_engine::{parsed_engine::ParsedDeclEngineInsert, DeclEngineGet},
    language::{
        parsed::{
            AstNode, AstNodeContent, CodeBlock, Declaration, Expression, ExpressionKind,
            FunctionApplicationExpression, FunctionDeclarationKind, IfExpression,
            IntrinsicFunctionExpression, MethodApplicationExpression, MethodName, ParseProgram,
            TreeType, TupleIndexExpression, VariableDeclaration,
        },
        ty::{self, TyAstNode, TyFunctionDecl, TyModule, TyProgram},
        CallPath, Literal, Purity,
    },
    metadata::MetadataManager,
    semantic_analysis::{
        namespace::{self, Namespace},
        TypeCheckContext,
    },
    transform::AttributesMap,
    BuildConfig, Engines, TypeArgs, TypeArgument, TypeBinding, TypeId, TypeInfo,
};
use sway_ast::{Intrinsic, ItemFn};
use sway_error::{
    diagnostic::ToDiagnostic,
    handler::{ErrorEmitted, Handler},
};
use sway_ir::{Context, Module};
use sway_types::{BaseIdent, Ident, Span, Spanned};

use super::{
    declaration::auto_impl, TypeCheckAnalysis, TypeCheckAnalysisContext, TypeCheckFinalization,
    TypeCheckFinalizationContext,
};

fn call_encode(_engines: &Engines, arg: Expression) -> Expression {
    Expression {
        kind: ExpressionKind::FunctionApplication(Box::new(FunctionApplicationExpression {
            call_path_binding: TypeBinding {
                inner: CallPath {
                    prefixes: vec![],
                    suffix: Ident::new_no_span("encode".into()),
                    is_absolute: false,
                },
                type_arguments: TypeArgs::Regular(vec![]),
                span: Span::dummy(),
            },
            arguments: vec![arg],
        })),
        span: Span::dummy(),
    }
}

fn call_decode_first_param(engines: &Engines) -> Expression {
    let string_slice_type_id = engines.te().insert(engines, TypeInfo::StringSlice, None);
    Expression {
        kind: ExpressionKind::FunctionApplication(Box::new(FunctionApplicationExpression {
            call_path_binding: TypeBinding {
                inner: CallPath {
                    prefixes: vec![],
                    suffix: Ident::new_no_span("decode_first_param".into()),
                    is_absolute: false,
                },
                type_arguments: TypeArgs::Regular(vec![TypeArgument {
                    type_id: string_slice_type_id,
                    initial_type_id: string_slice_type_id,
                    span: Span::dummy(),
                    call_path_tree: None,
                }]),
                span: Span::dummy(),
            },
            arguments: vec![],
        })),
        span: Span::dummy(),
    }
}

fn call_decode_second_param(_engines: &Engines, args_type: TypeArgument) -> Expression {
    Expression {
        kind: ExpressionKind::FunctionApplication(Box::new(FunctionApplicationExpression {
            call_path_binding: TypeBinding {
                inner: CallPath {
                    prefixes: vec![],
                    suffix: Ident::new_no_span("decode_second_param".into()),
                    is_absolute: false,
                },
                type_arguments: TypeArgs::Regular(vec![args_type]),
                span: Span::dummy(),
            },
            arguments: vec![],
        })),
        span: Span::dummy(),
    }
}

fn call_eq(_engines: &Engines, l: Expression, r: Expression) -> Expression {
    Expression {
        kind: ExpressionKind::MethodApplication(Box::new(MethodApplicationExpression {
            method_name_binding: TypeBinding {
                inner: MethodName::FromModule {
                    method_name: Ident::new_no_span("eq".to_string()),
                },
                type_arguments: TypeArgs::Regular(vec![]),
                span: Span::dummy(),
            },
            contract_call_params: vec![],
            arguments: vec![l, r],
        })),
        span: Span::dummy(),
    }
}

fn call_fn(expr: Expression, name: &str) -> Expression {
    Expression {
        kind: ExpressionKind::MethodApplication(Box::new(MethodApplicationExpression {
            method_name_binding: TypeBinding {
                inner: MethodName::FromModule {
                    method_name: Ident::new_no_span(name.to_string()),
                },
                type_arguments: TypeArgs::Regular(vec![]),
                span: Span::dummy(),
            },
            contract_call_params: vec![],
            arguments: vec![expr],
        })),
        span: Span::dummy(),
    }
}

fn arguments_type(engines: &Engines, decl: &TyFunctionDecl) -> Option<TypeArgument> {
    if decl.parameters.is_empty() {
        return None;
    }

    // if decl.parameters.len() == 1 {
    //     return Some(decl.parameters[0].type_argument.clone());
    // }

    let types = decl
        .parameters
        .iter()
        .map(|p| p.type_argument.clone())
        .collect();
    let type_id = engines.te().insert(engines, TypeInfo::Tuple(types), None);
    Some(TypeArgument {
        type_id,
        initial_type_id: type_id,
        span: Span::dummy(),
        call_path_tree: None,
    })
}

fn arguments_as_expressions(name: BaseIdent, decl: &TyFunctionDecl) -> Vec<Expression> {
    decl.parameters
        .iter()
        .enumerate()
        .map(|(idx, _)| Expression {
            kind: ExpressionKind::TupleIndex(TupleIndexExpression {
                prefix: Box::new(Expression {
                    kind: ExpressionKind::AmbiguousVariableExpression(name.clone()),
                    span: Span::dummy(),
                }),
                index: idx,
                index_span: Span::dummy(),
            }),
            span: Span::dummy(),
        })
        .collect()
}

fn gen_entry_fn(
    ctx: &mut TypeCheckContext,
    root: &mut TyModule,
    purity: Purity,
    contents: Vec<AstNode>,
    return_type_id: TypeId,
) -> Result<(), ErrorEmitted> {
    let entry_fn_decl = crate::language::parsed::function::FunctionDeclaration {
        purity,
        attributes: AttributesMap::default(),
        name: Ident::new_no_span("__entry".to_string()),
        visibility: crate::language::Visibility::Public,
        body: CodeBlock {
            contents: contents.clone(),
            whole_block_span: Span::dummy(),
        },
        parameters: vec![],
        span: Span::dummy(),
        return_type: TypeArgument {
            type_id: return_type_id,
            initial_type_id: return_type_id,
            span: Span::dummy(),
            call_path_tree: None,
        },
        type_parameters: vec![],
        where_clause: vec![],
        kind: FunctionDeclarationKind::Entry,
    };
    let entry_fn_decl = ctx.engines.pe().insert(entry_fn_decl);

    let handler = Handler::default();
    root.all_nodes.push(TyAstNode::type_check(
        &handler,
        ctx.by_ref(),
        AstNode {
            content: AstNodeContent::Declaration(Declaration::FunctionDeclaration(entry_fn_decl)),
            span: Span::dummy(),
        },
    )?);

    if handler.has_errors() {
        println!("{}", ctx.engines().de().pretty_print(&ctx.engines));
        let (a, b) = handler.consume();
        for a in a {
            println!(
                "gen_entry_fn: {:?} {:?}",
                a,
                a.to_diagnostic(ctx.engines.se())
            );
        }
    } else {
        assert!(!handler.has_errors(), "{:?}", handler);
        assert!(!handler.has_warnings(), "{:?}", handler);
    }

    Ok(())
}

impl TyProgram {
    /// Type-check the given parsed program to produce a typed program.
    ///
    /// The given `initial_namespace` acts as an initial state for each module within this program.
    /// It should contain a submodule for each library package dependency.
    pub fn type_check(
        handler: &Handler,
        engines: &Engines,
        parsed: &ParseProgram,
        initial_namespace: namespace::Module,
        package_name: &str,
        build_config: Option<&BuildConfig>,
    ) -> Result<Self, ErrorEmitted> {
        let experimental = build_config.map(|x| x.experimental).unwrap_or_default();

        let mut namespace = Namespace::init_root(initial_namespace);
        let mut ctx = TypeCheckContext::from_root(&mut namespace, engines, experimental)
            .with_kind(parsed.kind);

        let ParseProgram { root, kind } = parsed;

        // Analyze the dependency order for the submodules.
        let modules_dep_graph = ty::TyModule::analyze(handler, root)?;
        let module_eval_order: Vec<sway_types::BaseIdent> =
            modules_dep_graph.compute_order(handler)?;

        let mut root = ty::TyModule::type_check(
            handler,
            ctx.by_ref(),
            engines,
            parsed.kind,
            root,
            module_eval_order,
        )?;

        let (kind, declarations, configurables) = Self::validate_root(
            handler,
            engines,
            &root,
            *kind,
            package_name,
            ctx.experimental,
        )?;

        let program = TyProgram {
            kind,
            root,
            declarations,
            configurables,
            storage_slots: vec![],
            logged_types: vec![],
            messages_types: vec![],
        };

        Ok(program)
    }

    pub(crate) fn get_typed_program_with_initialized_storage_slots(
        self,
        handler: &Handler,
        engines: &Engines,
        context: &mut Context,
        md_mgr: &mut MetadataManager,
        module: Module,
    ) -> Result<Self, ErrorEmitted> {
        let decl_engine = engines.de();
        match &self.kind {
            ty::TyProgramKind::Contract { .. } => {
                let storage_decl = self
                    .declarations
                    .iter()
                    .find(|decl| matches!(decl, ty::TyDecl::StorageDecl { .. }));

                // Expecting at most a single storage declaration
                match storage_decl {
                    Some(ty::TyDecl::StorageDecl(ty::StorageDecl {
                        decl_id,
                        decl_span: _,
                        ..
                    })) => {
                        let decl = decl_engine.get_storage(decl_id);
                        let mut storage_slots = decl.get_initialized_storage_slots(
                            handler, engines, context, md_mgr, module,
                        )?;
                        // Sort the slots to standardize the output. Not strictly required by the
                        // spec.
                        storage_slots.sort();
                        Ok(Self {
                            storage_slots,
                            ..self
                        })
                    }
                    _ => Ok(Self {
                        storage_slots: vec![],
                        ..self
                    }),
                }
            }
            _ => Ok(Self {
                storage_slots: vec![],
                ..self
            }),
        }
    }
}

impl TypeCheckAnalysis for TyProgram {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        for node in self.root.all_nodes.iter() {
            node.type_check_analyze(handler, ctx)?;
        }
        Ok(())
    }
}

impl TypeCheckFinalization for TyProgram {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        handler.scope(|handler| {
            for node in self.root.all_nodes.iter_mut() {
                let _ = node.type_check_finalize(handler, ctx);
            }
            Ok(())
        })
    }
}
