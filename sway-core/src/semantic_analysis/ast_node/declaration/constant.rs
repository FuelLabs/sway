use sway_error::warning::{CompileWarning, Warning};
use sway_types::{style::is_screaming_snake_case, Spanned};

use crate::{
    error::*,
    language::{
        parsed::{self, *},
        ty::{self, TyConstantDecl},
        CallPath,
    },
    semantic_analysis::*,
    EnforceTypeArguments, Engines, TypeInfo,
};

impl ty::TyConstantDecl {
    pub fn type_check(mut ctx: TypeCheckContext, decl: ConstantDeclaration) -> CompileResult<Self> {
        let mut errors = vec![];
        let mut warnings = vec![];

        let engines = &ctx.engines.clone();
        let type_engine = engines.te();

        let ConstantDeclaration {
            name,
            span,
            mut type_ascription,
            value,
            is_configurable,
            attributes,
            visibility,
        } = decl;

        type_ascription.type_id = check!(
            ctx.resolve_type_with_self(
                type_ascription.type_id,
                &type_ascription.span,
                EnforceTypeArguments::No,
                None
            ),
            type_engine.insert(&ctx.engines, TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        let mut ctx = ctx
            .by_ref()
            .with_type_annotation(type_ascription.type_id)
            .with_help_text(
                "This declaration's type annotation does not match up with the assigned \
            expression's type.",
            );

        let value = match value {
            Some(value) => {
                let result = ty::TyExpression::type_check(ctx.by_ref(), value);

                if !is_screaming_snake_case(name.as_str()) {
                    warnings.push(CompileWarning {
                        span: name.span(),
                        warning_content: Warning::NonScreamingSnakeCaseConstName {
                            name: name.clone(),
                        },
                    })
                }

                let value = check!(
                    result,
                    ty::TyExpression::error(name.span(), &ctx.engines),
                    warnings,
                    errors
                );

                Some(value)
            }
            None => None,
        };

        // Integers are special in the sense that we can't only rely on the type of `expression`
        // to get the type of the variable. The type of the variable *has* to follow
        // `type_ascription` if `type_ascription` is a concrete integer type that does not
        // conflict with the type of `expression` (i.e. passes the type checking above).
        let return_type = match type_engine.get(type_ascription.type_id) {
            TypeInfo::UnsignedInteger(_) => type_ascription.type_id,
            _ => match &value {
                Some(value) => value.return_type,
                None => type_ascription.type_id,
            },
        };

        let mut call_path: CallPath = name.into();
        call_path = call_path.to_fullpath(ctx.namespace);

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
        ok(decl, warnings, errors)
    }

    /// Used to create a stubbed out constant when the constant fails to
    /// compile, preventing cascading namespace errors.
    pub(crate) fn error(engines: &Engines, decl: parsed::ConstantDeclaration) -> TyConstantDecl {
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
            return_type: engines.te().insert(engines, TypeInfo::Unknown),
            type_ascription,
            is_configurable: false,
            value: None,
            visibility,
            implementing_type: None,
        }
    }
}
