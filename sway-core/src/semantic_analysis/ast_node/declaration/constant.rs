use crate::{
    error::*,
    language::{
        parsed::{self, *},
        ty::{self, TyConstantDeclaration},
    },
    semantic_analysis::*,
};

impl ty::TyConstantDeclaration {
    pub fn type_check(mut ctx: TypeCheckContext, decl: ConstantDeclaration) -> CompileResult<Self> {
        let mut errors = vec![];
        let mut warnings = vec![];

        let value = match decl.value {
            Some(value) => Some(check!(
                ty::TyExpression::type_check(ctx.by_ref(), value),
                return err(warnings, errors),
                warnings,
                errors
            )),
            None => None,
        };

        // create the const decl
        let decl = ty::TyConstantDeclaration {
            name: decl.name,
            attributes: decl.attributes,
            is_configurable: false,
            type_ascription: decl.type_ascription,
            span: decl.span,
            value,
            visibility: decl.visibility,
            implementing_type: None,
        };
        ok(decl, warnings, errors)
    }

    /// Used to create a stubbed out constant when the constant fails to
    /// compile, preventing cascading namespace errors.
    pub(crate) fn error(decl: parsed::ConstantDeclaration) -> TyConstantDeclaration {
        let parsed::ConstantDeclaration {
            name,
            span,
            visibility,
            type_ascription,
            ..
        } = decl;
        TyConstantDeclaration {
            name,
            span,
            attributes: Default::default(),
            type_ascription,
            is_configurable: false,
            value: None,
            visibility,
            implementing_type: None,
        }
    }
}
