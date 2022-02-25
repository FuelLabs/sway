use std::fmt;

use itertools::Itertools;
use sway_types::Span;

use crate::{
    error::{err, ok},
    CompileError, CompileResult,
};

use super::{patstack::PatStack, pattern::Pattern};

/// A `WitnessReport` is a report of the witnesses to a `Pattern` being useful
/// and is used in the match expression exhaustivity checking algorithm.
#[derive(Debug)]
pub(crate) enum WitnessReport {
    NoWitnesses,
    Witnesses(PatStack),
}

impl WitnessReport {
    /// Joins two `WitnessReport`s together.
    pub(crate) fn join_witness_reports(a: WitnessReport, b: WitnessReport) -> Self {
        match (a, b) {
            (WitnessReport::NoWitnesses, WitnessReport::NoWitnesses) => WitnessReport::NoWitnesses,
            (WitnessReport::NoWitnesses, WitnessReport::Witnesses(wits)) => {
                WitnessReport::Witnesses(wits)
            }
            (WitnessReport::Witnesses(wits), WitnessReport::NoWitnesses) => {
                WitnessReport::Witnesses(wits)
            }
            (WitnessReport::Witnesses(wits1), WitnessReport::Witnesses(mut wits2)) => {
                let mut wits = wits1;
                wits.append(&mut wits2);
                WitnessReport::Witnesses(wits)
            }
        }
    }

    /// Given a `WitnessReport` *wr* and a constructor *c* with *a* number of
    /// sub-patterns, creates a new `Pattern` *p* and a new `WitnessReport`
    /// *wr'*. *p* is created by applying *c* to the first *a* elements of *wr*.
    /// *wr'* is created by taking the remaining elements of *wr* after *a*
    /// elements have been removed from the front of *wr*.
    pub(crate) fn split_into_leading_constructor(
        witness_report: WitnessReport,
        c: &Pattern,
        span: &Span,
    ) -> CompileResult<(Pattern, Self)> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match witness_report {
            WitnessReport::NoWitnesses => {
                errors.push(CompileError::Internal(
                    "expected to find witnesses to use as arguments to a constructor",
                    span.clone(),
                ));
                err(warnings, errors)
            }
            WitnessReport::Witnesses(witnesses) => {
                let (rs, ps) = check!(
                    witnesses.split_at(c.a(), span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let pat = check!(
                    Pattern::from_constructor_and_arguments(c, rs, span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                ok((pat, WitnessReport::Witnesses(ps)), warnings, errors)
            }
        }
    }

    /// Prepends a witness `Pattern` onto the `WitnessReport`.
    pub(crate) fn add_witness(&mut self, witness: Pattern, span: &Span) -> CompileResult<()> {
        let warnings = vec![];
        let mut errors = vec![];
        match self {
            WitnessReport::NoWitnesses => {
                errors.push(CompileError::Internal(
                    "expected to find witnesses",
                    span.clone(),
                ));
                err(warnings, errors)
            }
            WitnessReport::Witnesses(witnesses) => {
                witnesses.prepend(witness);
                ok((), warnings, errors)
            }
        }
    }

    /// Reports if this `WitnessReport` has witnesses.
    pub(crate) fn has_witnesses(&self) -> bool {
        match self {
            WitnessReport::NoWitnesses => false,
            WitnessReport::Witnesses(_) => true, // !witnesses.is_empty()
        }
    }
}

impl fmt::Display for WitnessReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let witnesses = match self {
            WitnessReport::NoWitnesses => PatStack::empty(),
            WitnessReport::Witnesses(witnesses) => witnesses.clone(),
        };
        let s = witnesses
            .flatten()
            .into_iter()
            .map(|x| format!("`{}`", x))
            .join(", ");
        write!(f, "{}", s)
    }
}
