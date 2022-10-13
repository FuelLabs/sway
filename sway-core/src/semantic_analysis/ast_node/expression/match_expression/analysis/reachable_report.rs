use sway_types::Span;

use crate::language::ty;

pub(crate) struct ReachableReport {
    pub(crate) reachable: bool,
    pub(crate) span: Span,
}

impl ReachableReport {
    pub(super) fn new(reachable: bool, scrutinee: ty::TyScrutinee) -> ReachableReport {
        ReachableReport {
            reachable,
            span: scrutinee.span,
        }
    }
}
