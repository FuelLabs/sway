use crate::{
    decl_engine::parsed_id::ParsedDeclId,
    language::{parsed::*, ty, CallPath},
    semantic_analysis::*,
    type_system::*,
    Engines,
};
use ast_elements::type_parameter::GenericTypeParameter;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Spanned;
use symbol_collection_context::SymbolCollectionContext;

impl ty::TyEnumDecl {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        decl_id: &ParsedDeclId<EnumDeclaration>,
    ) -> Result<(), ErrorEmitted> {
        let enum_decl = engines.pe().get_enum(decl_id);
        let decl = Declaration::EnumDeclaration(*decl_id);
        ctx.insert_parsed_symbol(handler, engines, enum_decl.name.clone(), decl.clone())?;

        // create a namespace for the decl, used to create a scope for generics
        let _ = ctx.scoped(engines, enum_decl.span.clone(), Some(decl), |mut _ctx| {
            Ok(())
        });
        Ok(())
    }

    pub fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        decl: EnumDeclaration,
    ) -> Result<Self, ErrorEmitted> {
        let EnumDeclaration {
            name,
            type_parameters,
            variants,
            span,
            attributes,
            visibility,
            ..
        } = decl;

        // create a namespace for the decl, used to create a scope for generics
        ctx.scoped(handler, Some(span.clone()), |ctx| {
            // Type check the type parameters.
            let new_type_parameters = GenericTypeParameter::type_check_type_params(
                handler,
                ctx.by_ref(),
                type_parameters,
                None,
            )?;

            // type check the variants
            let mut variants_buf = vec![];
            for variant in variants {
                variants_buf.push(
                    match ty::TyEnumVariant::type_check(handler, ctx.by_ref(), variant.clone()) {
                        Ok(res) => res,
                        Err(_) => continue,
                    },
                );
            }

            let call_path = CallPath::ident_to_fullpath(name, ctx.namespace());

            // create the enum decl
            let decl = ty::TyEnumDecl {
                call_path,
                generic_parameters: new_type_parameters,
                variants: variants_buf,
                span,
                attributes,
                visibility,
            };
            Ok(decl)
        })
    }
}

impl ty::TyEnumVariant {
    pub(crate) fn type_check(
        handler: &Handler,
        ctx: TypeCheckContext,
        variant: EnumVariant,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let mut type_argument = variant.type_argument;
        *type_argument.type_id_mut() = ctx
            .resolve_type(
                handler,
                type_argument.type_id(),
                &type_argument.span(),
                EnforceTypeArguments::Yes,
                None,
            )
            .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));
        Ok(ty::TyEnumVariant {
            name: variant.name.clone(),
            type_argument,
            tag: variant.tag,
            span: variant.span,
            attributes: variant.attributes,
        })
    }
}

impl TypeCheckAnalysis for ty::TyEnumDecl {
    fn type_check_analyze(
        &self,
        _handler: &Handler,
        _ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        Ok(())
    }
}

impl TypeCheckFinalization for ty::TyEnumDecl {
    fn type_check_finalize(
        &mut self,
        _handler: &Handler,
        _ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        Ok(())
    }
}
