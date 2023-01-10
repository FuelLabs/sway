use crate::{engine_threading::*, type_system::*};

use sway_types::Span;

/// Helper struct to perform the occurs check.
///
/// ---
///
/// "causes unification of a variable V and a structure S to fail if S
/// contains V"
/// https://en.wikipedia.org/wiki/Occurs_check
///
/// "occurs check: a check for whether the same variable occurs on both
/// sides and, if it does, decline to unify"
/// https://papl.cs.brown.edu/2016/Type_Inference.html
pub(super) struct OccursCheck<'a> {
    engines: Engines<'a>,
}

impl<'a> OccursCheck<'a> {
    /// Creates a new [OccursCheck].
    pub(super) fn new(engines: Engines<'a>) -> OccursCheck<'a> {
        OccursCheck { engines }
    }

    /// Checks whether `generic` occurs in `other` and returns true if so.
    ///
    /// NOTE: This first-cut implementation takes the most simple approach---
    /// does `other` contain `generic`? If so, return true.
    /// TODO: In the future, we may need to expand this definition.
    ///
    /// NOTE: This implementation assumes that `other` =/ `generic`, in which
    /// case the occurs check would return `false`, as this is a valid
    /// unification.
    pub(super) fn check(
        &self,
        generic: TypeInfo,
        other: &TypeInfo,
        span: &Span,
    ) -> CompileResult<bool> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let other_generics = check!(
            other.extract_nested_generics(self.engines, span),
            return err(warnings, errors),
            warnings,
            errors
        );
        let occurs = other_generics.contains(&self.engines.help_out(generic));
        ok(occurs, warnings, errors)
    }
}
