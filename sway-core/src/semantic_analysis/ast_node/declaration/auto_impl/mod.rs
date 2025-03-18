//! This module contains common infrastructure for generating and parsing auto-generated code.
pub mod abi_encoding;
pub mod debug;
pub mod marker_traits;

use std::ops::Deref;

use crate::{
    build_config::DbgGeneration,
    engine_threading::SpannedWithEngines,
    language::{
        parsed::{self, AstNodeContent, Declaration, FunctionDeclarationKind},
        ty::{self, TyAstNode, TyDecl},
    },
    semantic_analysis::TypeCheckContext,
    BuildTarget, Engines, GenericArgument, TypeInfo, TypeParameter,
};
use sway_error::handler::Handler;
use sway_parse::Parse;
use sway_types::{Named, SourceId, Spanned};

/// Contains all information needed to auto-implement code for a certain feature.
pub struct AutoImplContext<'a, 'b, I>
where
    'a: 'b,
{
    ctx: &'b mut TypeCheckContext<'a>,
    /// Additional information, aside from `ctx`, needed to auto-implement a concrete feature.
    #[allow(dead_code)]
    info: I,
}

impl<'a, 'b, I> AutoImplContext<'a, 'b, I>
where
    'a: 'b,
{
    pub fn new(ctx: &'b mut TypeCheckContext<'a>) -> Self
    where
        I: Default,
    {
        Self {
            ctx,
            info: I::default(),
        }
    }

    /// Parses `input` into the expected [Parse] type.
    /// The resulted [Parse] has source id set to autogenerated source id
    /// within the program represented by the `program_id`.
    fn parse<T>(engines: &Engines, original_source_id: Option<SourceId>, input: &str) -> T
    where
        T: Parse,
    {
        // Uncomment this to see what is being generated
        // println!("{}", input);

        let handler = <_>::default();
        let autogenerated_source_id = original_source_id.map(|source_id| {
            engines
                .se()
                .get_associated_autogenerated_source_id(&source_id)
        });

        let token_stream = sway_parse::lex(
            &handler,
            &std::sync::Arc::from(input),
            0,
            input.len(),
            autogenerated_source_id,
        )
        .unwrap();
        let mut p = sway_parse::Parser::new(&handler, &token_stream);
        p.check_double_underscore = false;

        let r = p.parse();

        assert!(!handler.has_errors(), "{:?}", handler);
        assert!(!handler.has_warnings(), "{:?}", handler);

        assert!(!p.has_errors());
        assert!(!p.has_warnings());

        r.unwrap()
    }

    /// Generates code like: `<A, B, u64>`.
    fn generate_type_parameters_declaration_code(
        &self,
        type_parameters: &[TypeParameter],
    ) -> String {
        if type_parameters.is_empty() {
            String::new()
        } else {
            format!(
                "<{}>",
                itertools::intersperse(type_parameters.iter().map(|p| { p.name().as_str() }), ", ")
                    .collect::<String>()
            )
        }
    }

    /// Generates code like: `T: Eq + Hash,\n`.
    fn generate_type_parameters_constraints_code(
        &self,
        type_parameters: &[TypeParameter],
        extra_constraint: &str,
    ) -> String {
        let mut code = String::new();

        for p in type_parameters.iter() {
            let p = p
                .as_type_parameter()
                .expect("only works with type parameters");
            code.push_str(&format!(
                "{}: {},\n",
                p.name.as_str(),
                itertools::intersperse(
                    [extra_constraint].into_iter().chain(
                        p.trait_constraints
                            .iter()
                            .map(|x| x.trait_name.suffix.as_str())
                    ),
                    " + "
                )
                .collect::<String>()
            ));
        }

        if !code.is_empty() {
            code = format!(" where {code}\n");
        }

        code
    }

    /// Parses `code` that contains [Declaration::FunctionDeclaration] into the
    /// corresponding [TyAstNode].
    pub fn parse_fn_to_ty_ast_node(
        &mut self,
        engines: &Engines,
        original_source_id: Option<SourceId>,
        kind: FunctionDeclarationKind,
        code: &str,
        dbg_generation: DbgGeneration,
    ) -> Result<TyAstNode, Handler> {
        let mut ctx = crate::transform::to_parsed_lang::Context::new(
            crate::BuildTarget::Fuel,
            dbg_generation,
            self.ctx.experimental,
        );

        let handler = Handler::default();

        let item = Self::parse(engines, original_source_id, code);
        let nodes = crate::transform::to_parsed_lang::item_to_ast_nodes(
            &mut ctx,
            &handler,
            engines,
            item,
            false,
            Some(kind),
        )
        .unwrap();

        let decl = match nodes[0].content {
            AstNodeContent::Declaration(Declaration::FunctionDeclaration(f)) => f,
            _ => unreachable!("unexpected node; expected `Declaration::FunctionDeclaration`"),
        };

        if handler.has_errors() {
            panic!(
                "{:?} {:?}",
                handler,
                original_source_id.map(|x| engines.se().get_file_name(&x))
            );
        }
        assert!(!handler.has_warnings(), "{:?}", handler);

        let mut ctx = self.ctx.by_ref();
        let _r = TyDecl::collect(
            &handler,
            engines,
            ctx.collection_ctx,
            Declaration::FunctionDeclaration(decl),
        );
        if handler.has_errors() {
            return Err(handler);
        }

        let r = ctx.scoped(&handler, None, |ctx| {
            TyDecl::type_check(
                &handler,
                &mut ctx.by_ref(),
                parsed::Declaration::FunctionDeclaration(decl),
            )
        });

        // Uncomment this to understand why an entry function was not generated
        // println!("{}, {:#?}", r.is_ok(), handler);

        let decl = r.map_err(|_| handler.clone())?;

        if handler.has_errors() || matches!(decl, TyDecl::ErrorRecovery(_, _)) {
            Err(handler)
        } else {
            Ok(TyAstNode {
                span: decl.span(engines),
                content: ty::TyAstNodeContent::Declaration(decl),
            })
        }
    }

    /// Parses `code` that contains [Declaration::ImplSelfOrTrait] into the
    /// corresponding [TyAstNode].
    fn parse_impl_trait_to_ty_ast_node(
        &mut self,
        engines: &Engines,
        original_source_id: Option<SourceId>,
        code: &str,
        dbg_generation: DbgGeneration,
    ) -> Result<TyAstNode, Handler> {
        let mut ctx = crate::transform::to_parsed_lang::Context::new(
            BuildTarget::Fuel,
            dbg_generation,
            self.ctx.experimental,
        );

        let handler = Handler::default();

        let item = Self::parse(engines, original_source_id, code);
        let nodes = crate::transform::to_parsed_lang::item_to_ast_nodes(
            &mut ctx, &handler, engines, item, false, None,
        )
        .unwrap();

        let decl = match nodes[0].content {
            AstNodeContent::Declaration(Declaration::ImplSelfOrTrait(f)) => f,
            _ => unreachable!("unexpected node; expected `Declaration::ImplSelfOrTrait`"),
        };

        assert!(!handler.has_errors(), "{:?}", handler);

        let mut ctx = self.ctx.by_ref();
        let _r = TyDecl::collect(
            &handler,
            engines,
            ctx.collection_ctx,
            Declaration::ImplSelfOrTrait(decl),
        );
        if handler.has_errors() {
            return Err(handler);
        }

        let r = ctx.scoped(&handler, None, |ctx| {
            TyDecl::type_check(&handler, ctx, Declaration::ImplSelfOrTrait(decl))
        });

        // Uncomment this to understand why auto impl failed for a type.
        // println!("{:#?}", handler);

        let decl = r.map_err(|_| handler.clone())?;

        if handler.has_errors() || matches!(decl, TyDecl::ErrorRecovery(_, _)) {
            Err(handler)
        } else {
            let impl_trait = if let TyDecl::ImplSelfOrTrait(impl_trait_id) = &decl {
                engines.de().get_impl_self_or_trait(&impl_trait_id.decl_id)
            } else {
                unreachable!();
            };

            // Insert trait implementation generated in the previous scope into the current scope.
            ctx.insert_trait_implementation(
                &handler,
                impl_trait.trait_name.clone(),
                impl_trait.trait_type_arguments.clone(),
                impl_trait.implementing_for.type_id(),
                impl_trait.impl_type_parameters.clone(),
                &impl_trait.items,
                &impl_trait.span,
                impl_trait
                    .trait_decl_ref
                    .as_ref()
                    .map(|decl_ref| decl_ref.decl_span().clone()),
                crate::namespace::IsImplSelf::No,
                crate::namespace::IsExtendingExistingImpl::No,
            )
            .ok();

            Ok(TyAstNode {
                span: decl.span(engines),
                content: ty::TyAstNodeContent::Declaration(decl),
            })
        }
    }

    /// Returns the string representation of the type given by `ta`, as given in code
    /// by the `ta`'s span.
    ///
    /// The safest way would be to return a canonical fully qualified type path.
    /// We do not have a way to do this at the moment, so the best way is to use
    /// exactly what was typed by the user, to accommodate aliased imports.
    fn generate_type(engines: &Engines, ta: &GenericArgument) -> Option<String> {
        match &*engines.te().get(ta.type_id()) {
            // A special case for function return type.
            // When a function does not define a return type, the span points to the whole signature.
            TypeInfo::Tuple(v) if v.is_empty() => Some("()".into()),
            // Otherwise, take the type from the span.
            _ => Some(ta.span().as_str().to_string()),
        }
    }
}

impl<'a, 'b, I> Deref for AutoImplContext<'a, 'b, I>
where
    'a: 'b,
{
    type Target = TypeCheckContext<'a>;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}
