use crate::language::ty;

pub(crate) struct ReachableReport {
    pub(crate) reachable: bool,
    pub(crate) scrutinee: ty::TyScrutinee,
}

impl ReachableReport {
    pub(super) fn new(reachable: bool, scrutinee: ty::TyScrutinee) -> ReachableReport {
        ReachableReport {
            reachable,
            scrutinee,
        }
    }
}
