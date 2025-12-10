use std::collections::VecDeque;

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};
use sway_types::{style::is_screaming_snake_case, Spanned};
use symbol_collection_context::SymbolCollectionContext;

use crate::{
    ast_elements::{type_argument::GenericTypeArgument, type_parameter::GenericTypeParameter},
    decl_engine::{
        parsed_id::ParsedDeclId, DeclEngineGetParsedDeclId, DeclEngineInsert, ReplaceDecls,
    },
    language::{
        parsed::*,
        ty::{self, TyConfigurableDecl, TyExpression},
        CallPath, CallPathType,
    },
    semantic_analysis::*,
    EnforceTypeArguments, Engines, GenericArgument, SubstTypes, TypeBinding, TypeCheckTypeBinding,
};

impl ty::TyConfigurableDecl {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        decl_id: &ParsedDeclId<ConfigurableDeclaration>,
    ) -> Result<(), ErrorEmitted> {
        let configurable_decl = engines.pe().get_configurable(decl_id);
        ctx.insert_parsed_symbol(
            handler,
            engines,
            configurable_decl.name.clone(),
            Declaration::ConfigurableDeclaration(*decl_id),
        )?;
        if let Some(value) = &configurable_decl.value {
            TyExpression::collect(handler, engines, ctx, value)?;
        }
        Ok(())
    }

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
            block_keyword_span,
        } = decl;

        type_ascription.type_id = ctx
            .resolve_type(
                handler,
                type_ascription.type_id,
                &type_ascription.span,
                EnforceTypeArguments::No,
                None,
            )
            .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));

        // this subst is required to replace associated types, namely TypeInfo::TraitType.
        type_ascription.type_id.subst(&ctx.subst_ctx());

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
                .with_type_annotation(type_engine.id_of_raw_slice())
                .with_help_text("Configurables must evaluate to slices.");

            let value = value.map(|value| {
                ty::TyExpression::type_check(handler, ctx.by_ref(), &value)
                    .unwrap_or_else(|err| ty::TyExpression::error(err, name.span(), engines))
            });

            let mut arguments = VecDeque::default();
            arguments.push_back(engines.te().id_of_raw_slice());
            arguments.push_back(engines.te().id_of_u64());
            arguments.push_back(engines.te().id_of_raw_slice());

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
                        callpath_type: CallPathType::Ambiguous,
                    },
                    type_arguments: crate::TypeArgs::Regular(vec![GenericArgument::Type(
                        GenericTypeArgument {
                            type_id: type_ascription.type_id,
                            initial_type_id: type_ascription.type_id,
                            span: sway_types::Span::dummy(),
                            call_path_tree: None,
                        },
                    )]),
                    span: value_span.clone(),
                },
                &abi_decode_in_place_handler,
                ctx.by_ref(),
            );

            // Map expected errors to more understandable ones
            handler.map_and_emit_errors_from(abi_decode_in_place_handler, |e| match e {
                CompileError::SymbolNotFound { .. } => {
                    Some(CompileError::ConfigurableMissingAbiDecodeInPlace {
                        span: block_keyword_span.clone(),
                    })
                }
                e => Some(e),
            })?;
            let (decode_fn_ref, _, _): (crate::decl_engine::DeclRefFunction, _, _) = r?;

            let decode_fn_id = *decode_fn_ref.id();
            let mut decode_fn_decl = (*engines.de().get_function(&decode_fn_id)).clone();
            let decl_mapping = GenericTypeParameter::gather_decl_mapping_from_trait_constraints(
                handler,
                ctx.by_ref(),
                &decode_fn_decl.type_parameters,
                decode_fn_decl.name.as_str(),
                &span,
            )?;
            decode_fn_decl.replace_decls(&decl_mapping, handler, &mut ctx)?;
            let decode_fn_ref = engines
                .de()
                .insert(
                    decode_fn_decl,
                    engines.de().get_parsed_decl_id(&decode_fn_id).as_ref(),
                )
                .with_parent(engines.de(), decode_fn_id.into());

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

    pub(crate) fn forbid_const_generics(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<(), ErrorEmitted> {
        if self.type_ascription.type_id.has_const_generics(engines) {
            Err(
                handler.emit_err(CompileError::ConstGenericNotSupportedHere {
                    span: self.type_ascription.span.clone(),
                }),
            )
        } else {
            Ok(())
        }
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
