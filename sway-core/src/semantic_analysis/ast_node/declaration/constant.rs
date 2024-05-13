use sway_error::{
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};
use sway_types::{style::is_screaming_snake_case, Spanned};

use crate::{
    language::{
        parsed::{self, *},
        ty::{self, TyConstantDecl},
        CallPath,
    },
    semantic_analysis::{type_check_context::EnforceTypeArguments, *},
    Engines, SubstTypes, TypeInfo,
};

impl ty::TyConstantDecl {
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
            is_configurable,
            attributes,
            visibility,
        } = decl;

        type_ascription.type_id = ctx
            .resolve_type(
                handler,
                type_ascription.type_id,
                &type_ascription.span,
                EnforceTypeArguments::No,
                None,
            )
            .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None));

        // this subst is required to replace associated types, namely TypeInfo::TraitType.
        type_ascription.type_id.subst(&ctx.type_subst(), engines);

        if !is_screaming_snake_case(name.as_str()) {
            handler.emit_warn(CompileWarning {
                span: name.span(),
                warning_content: Warning::NonScreamingSnakeCaseConstName { name: name.clone() },
            })
        }

        // Configurables will be encoded and must be type_checked into "slice"
        let (value, return_type) = if is_configurable && ctx.experimental.new_encoding {
            let mut ctx = ctx
                .by_ref()
                .with_type_annotation(type_engine.insert(engines, TypeInfo::RawUntypedSlice, None))
                .with_help_text("Configurables must evaluate to slices.");

            let value = value.map(|value| {
                ty::TyExpression::type_check(handler, ctx.by_ref(), &value)
                    .unwrap_or_else(|err| ty::TyExpression::error(err, name.span(), engines))
            });

            (
                value,
                type_engine.insert(engines, TypeInfo::RawUntypedSlice, None),
            )
        } else {
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

            (value, return_type)
        };

        let mut call_path: CallPath = name.into();
        call_path = call_path.to_fullpath(engines, ctx.namespace());

        // create the const decl
        let decl = ty::TyConstantDecl {
            call_path,
            attributes,
            is_configurable,
            return_type,
            type_ascription,
            span,
            value,
            visibility,
            implementing_type: None,
        };
        Ok(decl)
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
            return_type: type_engine.insert(engines, TypeInfo::Unknown, None),
            type_ascription,
            is_configurable: false,
            value: None,
            visibility,
            implementing_type: None,
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
