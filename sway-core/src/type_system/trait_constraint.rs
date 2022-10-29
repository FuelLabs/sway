use sway_error::error::CompileError;
use sway_types::Spanned;

use crate::{error::*, language::CallPath, semantic_analysis::TypeCheckContext, TypeArgument};

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub(crate) struct TraitConstraint {
    pub(crate) trait_name: CallPath,
    pub(crate) type_arguments: Vec<TypeArgument>,
}

impl Spanned for TraitConstraint {
    fn span(&self) -> sway_types::Span {
        self.trait_name.span()
    }
}

impl TraitConstraint {
    pub(crate) fn type_check(&mut self, ctx: TypeCheckContext) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        if self.type_arguments.is_empty() {
            errors.push(CompileError::Unimplemented(
                "using generic traits in trait constraints is not implemented yet",
                self.trait_name.span(),
            ));
            return err(warnings, errors);
        }

        // resolve the types of the type arguments
        for type_arg in self.type_arguments.iter_mut() {
            type_arg.type_id = check!(
                ctx.resolve_type_without_self(type_arg.type_id, &type_arg.span, None),
                return err(warnings, errors),
                warnings,
                errors
            );
        }

        ok((), warnings, errors)
    }
}
