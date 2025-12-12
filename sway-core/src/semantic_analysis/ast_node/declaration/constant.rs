use sway_error::{
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};
use sway_types::{style::is_screaming_snake_case, Spanned};
use symbol_collection_context::SymbolCollectionContext;

use crate::{
    decl_engine::parsed_id::ParsedDeclId,
    language::{
        parsed::{self, *},
        ty::{self, TyConstantDecl, TyExpression},
        CallPath,
    },
    semantic_analysis::*,
    EnforceTypeArguments, Engines, SubstTypes, TypeInfo,
};

impl ty::TyConstantDecl {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        decl_id: &ParsedDeclId<ConstantDeclaration>,
    ) -> Result<(), ErrorEmitted> {
        let constant_decl = engines.pe().get_constant(decl_id);
        ctx.insert_parsed_symbol(
            handler,
            engines,
            constant_decl.name.clone(),
            Declaration::ConstantDeclaration(*decl_id),
        )?;
        if let Some(value) = &constant_decl.value {
            TyExpression::collect(handler, engines, ctx, value)?;
        }
        Ok(())
    }

    pub fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        decl: ConstantDeclaration,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let ConstantDeclaration {
            name,
            span,
            mut type_ascription,
            value,
            attributes,
            visibility,
        } = decl.clone();

        type_ascription.type_id = ctx
            .resolve_type(
                handler,
                type_ascription.type_id,
                &type_ascription.span(),
                EnforceTypeArguments::No,
                None,
            )
            .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));

        // this subst is required to replace associated types, namely TypeInfo::TraitType.
        type_ascription.type_id.subst(&ctx.subst_ctx(handler));

        if !is_screaming_snake_case(name.as_str()) {
            handler.emit_warn(CompileWarning {
                span: name.span(),
                warning_content: Warning::NonScreamingSnakeCaseConstName { name: name.clone() },
            })
        }

        let mut ctx = ctx
            .by_ref()
            .with_type_annotation(type_ascription.type_id)
            .with_help_text(
                "This declaration's type annotation does not match up with the assigned \
        expression's type.",
            );

        let value = value.map(|value| {
            ty::TyExpression::type_check(handler, ctx.by_ref(), &value)
                .unwrap_or_else(|err| ty::TyExpression::error(err, name.span(), engines))
        });

        // Integers are special in the sense that we can't only rely on the type of `expression`
        // to get the type of the variable. The type of the variable *has* to follow
        // `type_ascription` if `type_ascription` is a concrete integer type that does not
        // conflict with the type of `expression` (i.e. passes the type checking above).
        let return_type = match &*type_engine.get(type_ascription.type_id) {
            TypeInfo::UnsignedInteger(_) => type_ascription.type_id,
            _ => match &value {
                Some(value) => value.return_type,
                None => type_ascription.type_id,
            },
        };

        let mut call_path: CallPath = name.into();
        call_path = call_path.to_fullpath(engines, ctx.namespace());

        Ok(ty::TyConstantDecl {
            call_path,
            attributes,
            return_type,
            type_ascription,
            span,
            value,
            visibility,
        })
    }

    /// Used to create a stubbed out constant when the constant fails to
    /// compile, preventing cascading namespace errors.
    pub(crate) fn error(engines: &Engines, decl: parsed::ConstantDeclaration) -> TyConstantDecl {
        let type_engine = engines.te();
        let parsed::ConstantDeclaration {
            name,
            span,
            visibility,
            type_ascription,
            ..
        } = decl;
        let call_path: CallPath = name.into();
        TyConstantDecl {
            call_path,
            span,
            attributes: Default::default(),
            return_type: type_engine.new_unknown(),
            type_ascription,
            value: None,
            visibility,
        }
    }
}

impl TypeCheckAnalysis for TyConstantDecl {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        if let Some(value) = self.value.as_ref() {
            value.type_check_analyze(handler, ctx)?;
        }
        Ok(())
    }
}

impl TypeCheckFinalization for TyConstantDecl {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        if let Some(value) = self.value.as_mut() {
            value.type_check_finalize(handler, ctx)?;
        }
        Ok(())
    }
}
