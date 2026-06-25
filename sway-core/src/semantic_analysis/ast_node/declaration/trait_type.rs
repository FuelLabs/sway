use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Span, Spanned};

use crate::{
    decl_engine::parsed_id::ParsedDeclId,
    language::{
        parsed::{Declaration, TraitTypeDeclaration},
        ty::{self, TyTraitType},
    },
    semantic_analysis::{
        symbol_collection_context::SymbolCollectionContext, TypeCheckAnalysis,
        TypeCheckAnalysisContext, TypeCheckContext,
    },
    EnforceTypeArguments, Engines,
};

impl ty::TyTraitType {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        decl_id: &ParsedDeclId<TraitTypeDeclaration>,
    ) -> Result<(), ErrorEmitted> {
        let trait_type_decl = engines.pe().get_trait_type(decl_id);
        ctx.insert_parsed_symbol(
            handler,
            engines,
            trait_type_decl.name.clone(),
            Declaration::TraitTypeDeclaration(*decl_id),
        )
    }

    pub(crate) fn type_check(
        handler: &Handler,
        ctx: TypeCheckContext,
        trait_type: &TraitTypeDeclaration,
    ) -> Result<Self, ErrorEmitted> {
        let TraitTypeDeclaration {
            name,
            attributes,
            ty_opt,
            span,
        } = trait_type;

        let engines = ctx.engines();
        let type_engine = engines.te();

        let ty_opt = ty_opt.as_ref().cloned();
        let ty = ty_opt.map(|mut ty| {
            *ty.type_id_mut() = ctx
                .resolve_type(
                    handler,
                    ty.type_id(),
                    &ty.span(),
                    EnforceTypeArguments::No,
                    None,
                )
                .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));
            ty
        });

        if let Some(implementing_type) = ctx.self_type() {
            Ok(ty::TyTraitType {
                name: name.clone(),
                attributes: attributes.clone(),
                ty,
                implementing_type,
                span: span.clone(),
            })
        } else {
            Err(handler.emit_err(CompileError::Internal(
                "Self type not provided.",
                span.clone(),
            )))
        }
    }

    /// Used to create a stubbed out constant when the constant fails to
    /// compile, preventing cascading namespace errors.
    pub(crate) fn error(engines: &Engines, decl: TraitTypeDeclaration) -> TyTraitType {
        let TraitTypeDeclaration {
            name,
            attributes,
            ty_opt,
            span,
        } = decl;
        TyTraitType {
            name,
            attributes,
            ty: ty_opt,
            implementing_type: engines.te().new_self_type(engines, Span::dummy()),
            span,
        }
    }
}

impl TypeCheckAnalysis for ty::TyTraitType {
    fn type_check_analyze(
        &self,
        _handler: &Handler,
        _ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        Ok(())
    }
}
