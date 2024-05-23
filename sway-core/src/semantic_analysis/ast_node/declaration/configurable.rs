use std::collections::VecDeque;

use sway_error::{
    handler::{ErrorEmitted, Handler},
    warning::{CompileWarning, Warning},
};
use sway_types::{style::is_screaming_snake_case, Spanned};

use crate::{
    decl_engine::{DeclEngineInsert, ReplaceDecls},
    language::{
        parsed::{self, *},
        ty::{self, TyConfigurableDecl, TyExpression},
        CallPath,
    },
    semantic_analysis::{type_check_context::EnforceTypeArguments, *},
    Engines, SubstTypes, TypeArgument, TypeBinding, TypeCheckTypeBinding, TypeInfo,
};

use self::ast_node::typed_expression::{monomorphize_method, resolve_method_name};

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

        // Configurables will be encoded and must be type_checked into "slice"
        let value = if ctx.experimental.new_encoding {
            let mut ctx = ctx
                .by_ref()
                .with_type_annotation(type_engine.insert(engines, TypeInfo::RawUntypedSlice, None))
                .with_help_text("Configurables must evaluate to slices.");

            let value = value.map(|value| {
                ty::TyExpression::type_check(handler, ctx.by_ref(), &value)
                    .unwrap_or_else(|err| ty::TyExpression::error(err, name.span(), engines))
            });

            value
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

            value
        };

        let mut call_path: CallPath = name.into();
        call_path = call_path.to_fullpath(engines, ctx.namespace());

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

        let (decode_fn_ref, _, _): (crate::decl_engine::DeclRefFunction, _, _) =
            crate::TypeBinding::type_check(
                &mut TypeBinding::<CallPath> {
                    inner: CallPath {
                        prefixes: vec![],
                        suffix: sway_types::Ident::new_no_span("abi_decode_in_place".into()),
                        is_absolute: false,
                    },
                    type_arguments: crate::TypeArgs::Regular(vec![TypeArgument {
                        type_id: type_ascription.type_id,
                        initial_type_id: type_ascription.type_id,
                        span: sway_types::Span::dummy(),
                        call_path_tree: None,
                    }]),
                    span: sway_types::Span::dummy(),
                },
                handler,
                ctx.by_ref(),
            )
            .unwrap();

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

        // let (decode_fn, _) = resolve_method_name(
        //     handler,
        //     ctx,
        //     &crate::TypeBinding {
        //         inner: MethodName::FromModule {
        //             method_name: sway_types::Ident::new_no_span("abi_decode".into()),
        //         },
        //         type_arguments: crate::TypeArgs::Regular(vec![TypeArgument {
        //             type_id: type_ascription.type_id,
        //             initial_type_id: type_ascription.type_id,
        //             span: sway_types::Span::dummy(),
        //             call_path_tree: None,
        //         }]),
        //         span: sway_types::Span::dummy(),
        //     },
        //     arguments,
        // )
        // .unwrap();

        Ok(ty::TyConfigurableDecl {
            call_path,
            attributes,
            return_type: type_ascription.type_id,
            type_ascription,
            span,
            value,
            decode_fn: Some(decode_fn_ref),
            visibility,
            implementing_type: None,
        })
    }

    /// Used to create a stubbed out constant when the constant fails to
    /// compile, preventing cascading namespace errors.
    pub(crate) fn error(
        engines: &Engines,
        decl: parsed::ConfigurableDeclaration,
    ) -> TyConfigurableDecl {
        let type_engine = engines.te();
        let parsed::ConfigurableDeclaration {
            name,
            span,
            visibility,
            type_ascription,
            ..
        } = decl;
        let call_path: CallPath = name.into();
        TyConfigurableDecl {
            call_path,
            span,
            attributes: Default::default(),
            return_type: type_engine.insert(engines, TypeInfo::Unknown, None),
            type_ascription,
            value: None,
            visibility,
            implementing_type: None,
            decode_fn: None,
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
