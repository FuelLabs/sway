use std::collections::VecDeque;

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};
use sway_types::{style::is_screaming_snake_case, Spanned};

use crate::{
    decl_engine::{DeclEngineInsert, ReplaceDecls},
    language::{
        parsed::*,
        ty::{self, TyConfigurableDecl},
        CallPath,
    },
    semantic_analysis::{type_check_context::EnforceTypeArguments, *},
    SubstTypes, TypeArgument, TypeBinding, TypeCheckTypeBinding, TypeInfo,
};

impl ty::TyConfigurableDecl {
    pub fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        decl: ConfigurableDeclaration,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let ConfigurableDeclaration {
            name,
            span,
            mut type_ascription,
            value,
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

        // Configurables using encoding v1 will be encoded and must be type_checked into "slice"
        let (value, decode_fn) = if ctx.experimental.new_encoding {
            let mut ctx = ctx
                .by_ref()
                .with_type_annotation(type_engine.insert(engines, TypeInfo::RawUntypedSlice, None))
                .with_help_text("Configurables must evaluate to slices.");

            let value = value.map(|value| {
                ty::TyExpression::type_check(handler, ctx.by_ref(), &value)
                    .unwrap_or_else(|err| ty::TyExpression::error(err, name.span(), engines))
            });

            let mut arguments = VecDeque::default();
            arguments.push_back(
                engines
                    .te()
                    .insert(engines, TypeInfo::RawUntypedSlice, None),
            );
            arguments.push_back(engines.te().insert(
                engines,
                TypeInfo::UnsignedInteger(sway_types::integer_bits::IntegerBits::SixtyFour),
                None,
            ));
            arguments.push_back(
                engines
                    .te()
                    .insert(engines, TypeInfo::RawUntypedSlice, None),
            );

            let value_span = value
                .as_ref()
                .map(|x| x.span.clone())
                .unwrap_or_else(|| span.clone());
            let abi_decode_in_place_handler = Handler::default();
            let r = crate::TypeBinding::type_check(
                &mut TypeBinding::<CallPath> {
                    inner: CallPath {
                        prefixes: vec![],
                        suffix: sway_types::Ident::new_with_override(
                            "abi_decode_in_place".into(),
                            value_span.clone(),
                        ),
                        is_absolute: false,
                    },
                    type_arguments: crate::TypeArgs::Regular(vec![TypeArgument {
                        type_id: type_ascription.type_id,
                        initial_type_id: type_ascription.type_id,
                        span: sway_types::Span::dummy(),
                        call_path_tree: None,
                    }]),
                    span: value_span.clone(),
                },
                &abi_decode_in_place_handler,
                ctx.by_ref(),
            );

            // Map expected errors to more understandable ones
            handler.map_and_emit_errors_from(abi_decode_in_place_handler, |e| match e {
                CompileError::SymbolNotFound { span, .. } => {
                    Some(CompileError::ConfigurableMissingAbiDecodeInPlace { span })
                }
                e => Some(e),
            })?;
            let (decode_fn_ref, _, _): (crate::decl_engine::DeclRefFunction, _, _) = r?;

            let mut decode_fn_decl = (*engines.de().get_function(&decode_fn_ref)).clone();
            let decl_mapping = crate::TypeParameter::gather_decl_mapping_from_trait_constraints(
                handler,
                ctx.by_ref(),
                &decode_fn_decl.type_parameters,
                decode_fn_decl.name.as_str(),
                &span,
            )?;
            decode_fn_decl.replace_decls(&decl_mapping, handler, &mut ctx)?;
            let decode_fn_ref = engines
                .de()
                .insert(decode_fn_decl)
                .with_parent(engines.de(), (*decode_fn_ref.id()).into());

            (value, Some(decode_fn_ref))
        } else {
            // while configurables using encoding v0 will typed as the configurable type itself
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

            (value, None)
        };

        let mut call_path: CallPath = name.into();
        call_path = call_path.to_fullpath(engines, ctx.namespace());

        Ok(ty::TyConfigurableDecl {
            call_path,
            attributes,
            return_type: type_ascription.type_id,
            type_ascription,
            span,
            value,
            decode_fn,
            visibility,
        })
    }
}

impl TypeCheckAnalysis for TyConfigurableDecl {
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

impl TypeCheckFinalization for TyConfigurableDecl {
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
